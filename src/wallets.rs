use actix_web::{web, HttpResponse};
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{ApiResponse, CreateWalletRequest, Wallet, UpdateWalletRequest};
use crate::cache::{get_or_set_cache, invalidate_cache_pattern};

// ==================== CRUD Handlers ====================

/// Get all wallets for a user (with caching)
pub async fn get_user_wallets(
    user_id: web::Path<String>,
    db: web::Data<PgPool>,
    cache: web::Data<ConnectionManager>,
) -> HttpResponse {
    let user_id = user_id.into_inner();
    let cache_key = format!("wallets:{}", user_id);

    let result = get_or_set_cache(
        &cache.get_ref(),
        &cache_key,
        fetch_wallets_from_db(db.get_ref(), &user_id),
    )
    .await;

    match result {
        Ok(wallets) => HttpResponse::Ok().json(ApiResponse::success(wallets)),
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<Vec<Wallet>>::error(e.to_string())),
    }
}

/// Get a single wallet by ID
pub async fn get_wallet(
    path: web::Path<(String, String)>,
    db: web::Data<PgPool>,
    cache: web::Data<ConnectionManager>,
) -> HttpResponse {
    let (user_id, wallet_id) = path.into_inner();
    let cache_key = format!("wallet:{}:{}", user_id, wallet_id);

    let result = get_or_set_cache(
        &cache.get_ref(),
        &cache_key,
        fetch_wallet_by_id(db.get_ref(), &wallet_id, &user_id),
    )
    .await;

    match result {
        Ok(wallet) => HttpResponse::Ok().json(ApiResponse::success(wallet)),
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<Wallet>::error(e.to_string())),
    }
}

/// Create a new wallet
pub async fn create_wallet(
    req: web::Json<CreateWalletRequest>,
    db: web::Data<PgPool>,
    cache: web::Data<ConnectionManager>,
) -> HttpResponse {
    let wallet_id = Uuid::new_v4().to_string();
    let wallet_type_str = req.wallet_type.as_str();

    let query_result = sqlx::query_as::<_, Wallet>(
        r#"
        INSERT INTO wallets (id, user_id, name, balance, credit_limit, wallet_type)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, user_id, name, balance, credit_limit, wallet_type, created_at, updated_at
        "#,
    )
    .bind(&wallet_id)
    .bind(&req.user_id)
    .bind(&req.name)
    .bind(&req.balance)
    .bind(&req.credit_limit)
    .bind(wallet_type_str)
    .fetch_one(db.get_ref())
    .await;

    match query_result {
        Ok(wallet) => {
            // Invalidate user's wallets cache
            let mut cache_clone = cache.get_ref().clone();
            let pattern = format!("wallets:{}", req.user_id);
            let _ = invalidate_cache_pattern(&mut cache_clone, &pattern).await;

            HttpResponse::Created().json(ApiResponse::success(wallet))
        }
        Err(e) => {
            log::error!("Failed to create wallet: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<Wallet>::error("Failed to create wallet".to_string()))
        }
    }
}

/// Update a wallet
pub async fn update_wallet(
    path: web::Path<(String, String)>,
    req: web::Json<UpdateWalletRequest>,
    db: web::Data<PgPool>,
    cache: web::Data<ConnectionManager>,
) -> HttpResponse {
    let (user_id, wallet_id) = path.into_inner();

    let query_result = sqlx::query_as::<_, Wallet>(
        r#"
        UPDATE wallets
        SET name = COALESCE($1, name), balance = COALESCE($2, balance), credit_limit = COALESCE($3, credit_limit)
        WHERE id = $4 AND user_id = $5
        RETURNING id, user_id, name, balance, credit_limit, wallet_type, created_at, updated_at
        "#,
    )
    .bind(&req.name)
    .bind(&req.balance)
    .bind(&req.credit_limit)
    .bind(&wallet_id)
    .bind(&user_id)
    .fetch_optional(db.get_ref())
    .await;

    match query_result {
        Ok(Some(wallet)) => {
            // Invalidate relevant caches
            let mut cache_clone = cache.get_ref().clone();
            let pattern = format!("wallet{}:*", user_id);
            let _ = invalidate_cache_pattern(&mut cache_clone, &pattern).await;

            HttpResponse::Ok().json(ApiResponse::success(wallet))
        }
        Ok(None) => {
            HttpResponse::NotFound()
                .json(ApiResponse::<Wallet>::error("Wallet not found".to_string()))
        }
        Err(e) => {
            log::error!("Failed to update wallet: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<Wallet>::error("Failed to update wallet".to_string()))
        }
    }
}

/// Delete a wallet
pub async fn delete_wallet(
    path: web::Path<(String, String)>,
    db: web::Data<PgPool>,
    cache: web::Data<ConnectionManager>,
) -> HttpResponse {
    let (user_id, wallet_id) = path.into_inner();

    let delete_result = sqlx::query("DELETE FROM wallets WHERE id = $1 AND user_id = $2")
        .bind(&wallet_id)
        .bind(&user_id)
        .execute(db.get_ref())
        .await;

    match delete_result {
        Ok(result) => {
            if result.rows_affected() > 0 {
                // Invalidate relevant caches
                let mut cache_clone = cache.get_ref().clone();
                let pattern = format!("wallet{}:*", user_id);
                let _ = invalidate_cache_pattern(&mut cache_clone, &pattern).await;

                HttpResponse::NoContent().finish()
            } else {
                HttpResponse::NotFound()
                    .json(ApiResponse::<String>::error("Wallet not found".to_string()))
            }
        }
        Err(e) => {
            log::error!("Failed to delete wallet: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error("Failed to delete wallet".to_string()))
        }
    }
}

// ==================== Database Functions ====================

async fn fetch_wallets_from_db(pool: &PgPool, user_id: &str) -> Result<Vec<Wallet>, sqlx::Error> {
    sqlx::query_as::<_, Wallet>(
        "SELECT id, user_id, name, balance, credit_limit, wallet_type, created_at, updated_at FROM wallets WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

async fn fetch_wallet_by_id(
    pool: &PgPool,
    wallet_id: &str,
    user_id: &str,
) -> Result<Wallet, sqlx::Error> {
    sqlx::query_as::<_, Wallet>(
        "SELECT id, user_id, name, balance, credit_limit, wallet_type, created_at, updated_at FROM wallets WHERE id = $1 AND user_id = $2",
    )
    .bind(wallet_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
}

// Update wallet balance (internal helper)
pub async fn update_wallet_balance(
    pool: &PgPool,
    wallet_id: &str,
    amount_delta: f64,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE wallets SET balance = balance + $1 WHERE id = $2")
        .bind(amount_delta)
        .bind(wallet_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ==================== Route Configuration ====================

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/wallets")
            .route("/user/{user_id}", web::get().to(get_user_wallets))
            .route("/{user_id}/{wallet_id}", web::get().to(get_wallet))
            .route("", web::post().to(create_wallet))
            .route("/{user_id}/{wallet_id}", web::put().to(update_wallet))
            .route("/{user_id}/{wallet_id}", web::delete().to(delete_wallet)),
    );
}
