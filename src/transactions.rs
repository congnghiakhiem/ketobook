use actix_web::{web, HttpResponse};
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

use crate::models::{ApiResponse, CreateTransactionRequest, Transaction, UpdateTransactionRequest};
use crate::cache::{get_or_set_cache, invalidate_cache_pattern};

// ==================== CRUD Handlers ====================

/// Get all transactions for a user (with caching)
pub async fn get_user_transactions(
    user_id: web::Path<String>,
    db: web::Data<PgPool>,
    cache: web::Data<ConnectionManager>,
) -> HttpResponse {
    let user_id = user_id.into_inner();
    let cache_key = format!("transactions:{}", user_id);

    let result = get_or_set_cache(
        &cache.get_ref(),
        &cache_key,
        fetch_transactions_from_db(db.get_ref(), &user_id),
    )
    .await;

    match result {
        Ok(transactions) => HttpResponse::Ok().json(ApiResponse::success(transactions)),
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<Vec<Transaction>>::error(e.to_string())),
    }
}

/// Get a single transaction by ID
pub async fn get_transaction(
    path: web::Path<(String, String)>,
    db: web::Data<PgPool>,
    cache: web::Data<ConnectionManager>,
) -> HttpResponse {
    let (user_id, transaction_id) = path.into_inner();
    let cache_key = format!("transaction:{}:{}", user_id, transaction_id);

    let result = get_or_set_cache(
        &cache.get_ref(),
        &cache_key,
        fetch_transaction_by_id(db.get_ref(), &transaction_id, &user_id),
    )
    .await;

    match result {
        Ok(transaction) => HttpResponse::Ok().json(ApiResponse::success(transaction)),
        Err(e) => HttpResponse::NotFound()
            .json(ApiResponse::<Transaction>::error(e.to_string())),
    }
}

/// Create a new transaction
pub async fn create_transaction(
    req: web::Json<CreateTransactionRequest>,
    db: web::Data<PgPool>,
    cache: web::Data<ConnectionManager>,
) -> HttpResponse {
    let transaction_id = Uuid::new_v4().to_string();
    let now = Utc::now();

    let query = sqlx::query_as::<_, Transaction>(
        "INSERT INTO transactions (id, user_id, amount, transaction_type, category, description, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
         RETURNING *"
    )
    .bind(&transaction_id)
    .bind(&req.user_id)
    .bind(req.amount)
    .bind(&req.transaction_type)
    .bind(&req.category)
    .bind(&req.description)
    .bind(now)
    .bind(now);

    match query.fetch_one(db.get_ref()).await {
        Ok(transaction) => {
            // Invalidate cache for this user's transactions
            let _ = invalidate_cache_pattern(&cache.get_ref(), &format!("transactions:{}*", req.user_id)).await;
            HttpResponse::Created().json(ApiResponse::success(transaction))
        }
        Err(e) => {
            log::error!("Error creating transaction: {}", e);
            HttpResponse::BadRequest()
                .json(ApiResponse::<Transaction>::error("Failed to create transaction".to_string()))
        }
    }
}

/// Update a transaction
pub async fn update_transaction(
    path: web::Path<(String, String)>,
    req: web::Json<UpdateTransactionRequest>,
    db: web::Data<PgPool>,
    cache: web::Data<ConnectionManager>,
) -> HttpResponse {
    let (user_id, transaction_id) = path.into_inner();
    let now = Utc::now();

    let query = sqlx::query_as::<_, Transaction>(
        "UPDATE transactions 
         SET amount = COALESCE($1, amount),
             category = COALESCE($2, category),
             description = COALESCE($3, description),
             updated_at = $4
         WHERE id = $5 AND user_id = $6
         RETURNING *"
    )
    .bind(req.amount)
    .bind(&req.category)
    .bind(&req.description)
    .bind(now)
    .bind(&transaction_id)
    .bind(&user_id);

    match query.fetch_optional(db.get_ref()).await {
        Ok(Some(transaction)) => {
            let _ = invalidate_cache_pattern(&cache.get_ref(), &format!("transaction*:{}*", user_id)).await;
            HttpResponse::Ok().json(ApiResponse::success(transaction))
        }
        Ok(None) => HttpResponse::NotFound()
            .json(ApiResponse::<Transaction>::error("Transaction not found".to_string())),
        Err(e) => {
            log::error!("Error updating transaction: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<Transaction>::error("Failed to update transaction".to_string()))
        }
    }
}

/// Delete a transaction
pub async fn delete_transaction(
    path: web::Path<(String, String)>,
    db: web::Data<PgPool>,
    cache: web::Data<ConnectionManager>,
) -> HttpResponse {
    let (user_id, transaction_id) = path.into_inner();

    let result = sqlx::query("DELETE FROM transactions WHERE id = $1 AND user_id = $2")
        .bind(&transaction_id)
        .bind(&user_id)
        .execute(db.get_ref())
        .await;

    match result {
        Ok(query_result) => {
            if query_result.rows_affected() > 0 {
                let _ = invalidate_cache_pattern(&cache.get_ref(), &format!("transaction*:{}*", user_id)).await;
                HttpResponse::NoContent().finish()
            } else {
                HttpResponse::NotFound()
                    .json(ApiResponse::<String>::error("Transaction not found".to_string()))
            }
        }
        Err(e) => {
            log::error!("Error deleting transaction: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error("Failed to delete transaction".to_string()))
        }
    }
}

// ==================== Database Queries ====================

async fn fetch_transactions_from_db(
    pool: &PgPool,
    user_id: &str,
) -> Result<Vec<Transaction>, sqlx::Error> {
    sqlx::query_as::<_, Transaction>("SELECT * FROM transactions WHERE user_id = $1 ORDER BY created_at DESC")
        .bind(user_id)
        .fetch_all(pool)
        .await
}

async fn fetch_transaction_by_id(
    pool: &PgPool,
    transaction_id: &str,
    user_id: &str,
) -> Result<Transaction, sqlx::Error> {
    sqlx::query_as::<_, Transaction>("SELECT * FROM transactions WHERE id = $1 AND user_id = $2")
        .bind(transaction_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
}

// ==================== Route Configuration ====================

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/transactions")
            .route("/user/{user_id}", web::get().to(get_user_transactions))
            .route("/{user_id}/{transaction_id}", web::get().to(get_transaction))
            .route("", web::post().to(create_transaction))
            .route("/{user_id}/{transaction_id}", web::put().to(update_transaction))
            .route("/{user_id}/{transaction_id}", web::delete().to(delete_transaction)),
    );
}
