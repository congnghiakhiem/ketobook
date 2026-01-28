use actix_web::{web, HttpResponse};
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

use crate::models::{ApiResponse, CreateDebtRequest, Debt, UpdateDebtRequest};
use crate::cache::{get_or_set_cache, invalidate_cache_pattern};

// ==================== CRUD Handlers ====================

/// Get all debts for a user (with caching)
pub async fn get_user_debts(
    user_id: web::Path<String>,
    db: web::Data<PgPool>,
    cache: web::Data<ConnectionManager>,
) -> HttpResponse {
    let user_id = user_id.into_inner();
    let cache_key = format!("debts:{}", user_id);

    let result = get_or_set_cache(
        &cache.get_ref(),
        &cache_key,
        fetch_debts_from_db(db.get_ref(), &user_id),
    )
    .await;

    match result {
        Ok(debts) => HttpResponse::Ok().json(ApiResponse::success(debts)),
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<Vec<Debt>>::error(e.to_string())),
    }
}

/// Get a single debt by ID
pub async fn get_debt(
    path: web::Path<(String, String)>,
    db: web::Data<PgPool>,
    cache: web::Data<ConnectionManager>,
) -> HttpResponse {
    let (user_id, debt_id) = path.into_inner();
    let cache_key = format!("debt:{}:{}", user_id, debt_id);

    let result = get_or_set_cache(
        &cache.get_ref(),
        &cache_key,
        fetch_debt_by_id(db.get_ref(), &debt_id, &user_id),
    )
    .await;

    match result {
        Ok(debt) => HttpResponse::Ok().json(ApiResponse::success(debt)),
        Err(e) => HttpResponse::NotFound()
            .json(ApiResponse::<Debt>::error(e.to_string())),
    }
}

/// Create a new debt
pub async fn create_debt(
    req: web::Json<CreateDebtRequest>,
    db: web::Data<PgPool>,
    cache: web::Data<ConnectionManager>,
) -> HttpResponse {
    let debt_id = Uuid::new_v4().to_string();
    let now = Utc::now();

    let query = sqlx::query_as::<_, Debt>(
        "INSERT INTO debts (id, user_id, creditor_name, amount, interest_rate, due_date, status, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
         RETURNING *"
    )
    .bind(&debt_id)
    .bind(&req.user_id)
    .bind(&req.creditor_name)
    .bind(req.amount.clone())
    .bind(req.interest_rate.clone())
    .bind(req.due_date)
    .bind("active")
    .bind(now)
    .bind(now);

    match query.fetch_one(db.get_ref()).await {
        Ok(debt) => {
            // Invalidate cache for this user's debts
            let _ = invalidate_cache_pattern(&cache.get_ref(), &format!("debts:{}*", req.user_id)).await;
            HttpResponse::Created().json(ApiResponse::success(debt))
        }
        Err(e) => {
            log::error!("Error creating debt: {}", e);
            HttpResponse::BadRequest()
                .json(ApiResponse::<Debt>::error("Failed to create debt".to_string()))
        }
    }
}

/// Update a debt
pub async fn update_debt(
    path: web::Path<(String, String)>,
    req: web::Json<UpdateDebtRequest>,
    db: web::Data<PgPool>,
    cache: web::Data<ConnectionManager>,
) -> HttpResponse {
    let (user_id, debt_id) = path.into_inner();
    let now = Utc::now();

    let query = sqlx::query_as::<_, Debt>(
        "UPDATE debts 
         SET creditor_name = COALESCE($1, creditor_name),
             amount = COALESCE($2, amount),
             interest_rate = COALESCE($3, interest_rate),
             due_date = COALESCE($4, due_date),
             status = COALESCE($5, status),
             updated_at = $6
         WHERE id = $7 AND user_id = $8
         RETURNING *"
    )
    .bind(&req.creditor_name)
    .bind(req.amount.clone())
    .bind(req.interest_rate.clone())
    .bind(req.due_date)
    .bind(&req.status)
    .bind(now)
    .bind(&debt_id)
    .bind(&user_id);

    match query.fetch_optional(db.get_ref()).await {
        Ok(Some(debt)) => {
            let _ = invalidate_cache_pattern(&cache.get_ref(), &format!("debt*:{}*", user_id)).await;
            HttpResponse::Ok().json(ApiResponse::success(debt))
        }
        Ok(None) => HttpResponse::NotFound()
            .json(ApiResponse::<Debt>::error("Debt not found".to_string())),
        Err(e) => {
            log::error!("Error updating debt: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<Debt>::error("Failed to update debt".to_string()))
        }
    }
}

/// Delete a debt
pub async fn delete_debt(
    path: web::Path<(String, String)>,
    db: web::Data<PgPool>,
    cache: web::Data<ConnectionManager>,
) -> HttpResponse {
    let (user_id, debt_id) = path.into_inner();

    let result = sqlx::query("DELETE FROM debts WHERE id = $1 AND user_id = $2")
        .bind(&debt_id)
        .bind(&user_id)
        .execute(db.get_ref())
        .await;

    match result {
        Ok(query_result) => {
            if query_result.rows_affected() > 0 {
                let _ = invalidate_cache_pattern(&cache.get_ref(), &format!("debt*:{}*", user_id)).await;
                HttpResponse::NoContent().finish()
            } else {
                HttpResponse::NotFound()
                    .json(ApiResponse::<String>::error("Debt not found".to_string()))
            }
        }
        Err(e) => {
            log::error!("Error deleting debt: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error("Failed to delete debt".to_string()))
        }
    }
}

// ==================== Database Queries ====================

async fn fetch_debts_from_db(
    pool: &PgPool,
    user_id: &str,
) -> Result<Vec<Debt>, sqlx::Error> {
    sqlx::query_as::<_, Debt>("SELECT * FROM debts WHERE user_id = $1 ORDER BY due_date ASC")
        .bind(user_id)
        .fetch_all(pool)
        .await
}

async fn fetch_debt_by_id(
    pool: &PgPool,
    debt_id: &str,
    user_id: &str,
) -> Result<Debt, sqlx::Error> {
    sqlx::query_as::<_, Debt>("SELECT * FROM debts WHERE id = $1 AND user_id = $2")
        .bind(debt_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
}

// ==================== Route Configuration ====================

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/debts")
            .route("/user/{user_id}", web::get().to(get_user_debts))
            .route("/{user_id}/{debt_id}", web::get().to(get_debt))
            .route("", web::post().to(create_debt))
            .route("/{user_id}/{debt_id}", web::put().to(update_debt))
            .route("/{user_id}/{debt_id}", web::delete().to(delete_debt)),
    );
}
