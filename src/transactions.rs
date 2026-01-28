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

    // Validate wallet exists and belongs to user
    let wallet_check = sqlx::query("SELECT id FROM wallets WHERE id = $1 AND user_id = $2")
        .bind(&req.wallet_id)
        .bind(&req.user_id)
        .fetch_optional(db.get_ref())
        .await;

    match wallet_check {
        Ok(None) => {
            return HttpResponse::BadRequest()
                .json(ApiResponse::<Transaction>::error("Wallet not found or doesn't belong to user".to_string()))
        }
        Err(e) => {
            log::error!("Error validating wallet: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<Transaction>::error("Failed to validate wallet".to_string()));
        }
        _ => {}
    }

    // Start transaction
    let mut tx = match db.begin().await {
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to begin transaction: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<Transaction>::error("Database error".to_string()));
        }
    };

    // Insert transaction
    let query = sqlx::query_as::<_, Transaction>(
        "INSERT INTO transactions (id, user_id, wallet_id, amount, transaction_type, category, description, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
         RETURNING id, user_id, wallet_id, amount, transaction_type, category, description, created_at, updated_at"
    )
    .bind(&transaction_id)
    .bind(&req.user_id)
    .bind(&req.wallet_id)
    .bind(req.amount)
    .bind(&req.transaction_type)
    .bind(&req.category)
    .bind(&req.description)
    .bind(now)
    .bind(now);

    let transaction = match query.fetch_one(&mut *tx).await {
        Ok(t) => t,
        Err(e) => {
            log::error!("Error creating transaction: {}", e);
            let _ = tx.rollback().await;
            return HttpResponse::BadRequest()
                .json(ApiResponse::<Transaction>::error("Failed to create transaction".to_string()));
        }
    };

    // Update wallet balance based on transaction type
    let amount_delta = match req.transaction_type.as_str() {
        "income" => req.amount,
        "expense" => -req.amount,
        _ => {
            let _ = tx.rollback().await;
            return HttpResponse::BadRequest()
                .json(ApiResponse::<Transaction>::error("Invalid transaction type".to_string()));
        }
    };

    if let Err(e) = sqlx::query("UPDATE wallets SET balance = balance + $1 WHERE id = $2")
        .bind(amount_delta)
        .bind(&req.wallet_id)
        .execute(&mut *tx)
        .await
    {
        log::error!("Error updating wallet balance: {}", e);
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError()
            .json(ApiResponse::<Transaction>::error("Failed to update wallet balance".to_string()));
    }

    // Commit transaction
    if let Err(e) = tx.commit().await {
        log::error!("Failed to commit transaction: {}", e);
        return HttpResponse::InternalServerError()
            .json(ApiResponse::<Transaction>::error("Failed to save changes".to_string()));
    }

    // Invalidate caches
    let _ = invalidate_cache_pattern(&cache.get_ref(), &format!("transactions:{}*", req.user_id)).await;
    let _ = invalidate_cache_pattern(&cache.get_ref(), &format!("wallet{}:*", req.user_id)).await;
    let _ = invalidate_cache_pattern(&cache.get_ref(), &format!("wallets:{}*", req.user_id)).await;

    HttpResponse::Created().json(ApiResponse::success(transaction))
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

    // Fetch current transaction
    let current_tx = match sqlx::query_as::<_, Transaction>(
        "SELECT id, user_id, wallet_id, amount, transaction_type, category, description, created_at, updated_at FROM transactions WHERE id = $1 AND user_id = $2"
    )
    .bind(&transaction_id)
    .bind(&user_id)
    .fetch_optional(db.get_ref())
    .await {
        Ok(Some(tx)) => tx,
        Ok(None) => {
            return HttpResponse::NotFound()
                .json(ApiResponse::<Transaction>::error("Transaction not found".to_string()));
        }
        Err(e) => {
            log::error!("Error fetching current transaction: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<Transaction>::error("Database error".to_string()));
        }
    };

    // If wallet_id is being changed, validate new wallet
    let new_wallet_id = match &req.wallet_id {
        Some(wid) => {
            let wallet_check = sqlx::query("SELECT id FROM wallets WHERE id = $1 AND user_id = $2")
                .bind(wid)
                .bind(&user_id)
                .fetch_optional(db.get_ref())
                .await;

            match wallet_check {
                Ok(Some(_)) => Some(wid.clone()),
                Ok(None) => {
                    return HttpResponse::BadRequest()
                        .json(ApiResponse::<Transaction>::error("New wallet not found".to_string()));
                }
                Err(e) => {
                    log::error!("Error validating wallet: {}", e);
                    return HttpResponse::InternalServerError()
                        .json(ApiResponse::<Transaction>::error("Failed to validate wallet".to_string()));
                }
            }
        }
        None => None,
    };

    // Start database transaction
    let mut tx = match db.begin().await {
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to begin transaction: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<Transaction>::error("Database error".to_string()));
        }
    };

    // If wallet changed, reverse old balance and apply to new
    if let Some(new_wid) = &new_wallet_id {
        if new_wid != current_tx.wallet_id.as_ref().unwrap() {
            let old_wallet_id = current_tx.wallet_id.as_ref().unwrap();
            let amount_delta = match current_tx.transaction_type.as_str() {
                "income" => -current_tx.amount,
                "expense" => current_tx.amount,
                _ => {
                    let _ = tx.rollback().await;
                    return HttpResponse::InternalServerError()
                        .json(ApiResponse::<Transaction>::error("Invalid transaction type".to_string()));
                }
            };

            // Reverse on old wallet
            if let Err(e) = sqlx::query("UPDATE wallets SET balance = balance + $1 WHERE id = $2")
                .bind(amount_delta)
                .bind(old_wallet_id)
                .execute(&mut *tx)
                .await
            {
                log::error!("Error reversing old wallet balance: {}", e);
                let _ = tx.rollback().await;
                return HttpResponse::InternalServerError()
                    .json(ApiResponse::<Transaction>::error("Failed to update old wallet".to_string()));
            }

            // Apply to new wallet (negative of reversal)
            if let Err(e) = sqlx::query("UPDATE wallets SET balance = balance + $1 WHERE id = $2")
                .bind(-amount_delta)
                .bind(new_wid)
                .execute(&mut *tx)
                .await
            {
                log::error!("Error applying new wallet balance: {}", e);
                let _ = tx.rollback().await;
                return HttpResponse::InternalServerError()
                    .json(ApiResponse::<Transaction>::error("Failed to update new wallet".to_string()));
            }
        }
    }

    // If amount changed, update wallet balance accordingly
    if let Some(new_amount) = req.amount {
        if new_amount != current_tx.amount && new_wallet_id.is_none() {
            let old_wallet_id = current_tx.wallet_id.as_ref().unwrap();
            let amount_diff = new_amount - current_tx.amount;
            let balance_delta = match current_tx.transaction_type.as_str() {
                "income" => amount_diff,
                "expense" => -amount_diff,
                _ => {
                    let _ = tx.rollback().await;
                    return HttpResponse::InternalServerError()
                        .json(ApiResponse::<Transaction>::error("Invalid transaction type".to_string()));
                }
            };

            if let Err(e) = sqlx::query("UPDATE wallets SET balance = balance + $1 WHERE id = $2")
                .bind(balance_delta)
                .bind(old_wallet_id)
                .execute(&mut *tx)
                .await
            {
                log::error!("Error updating wallet balance for amount change: {}", e);
                let _ = tx.rollback().await;
                return HttpResponse::InternalServerError()
                    .json(ApiResponse::<Transaction>::error("Failed to update wallet balance".to_string()));
            }
        }
    }

    // Update transaction
    let query = sqlx::query_as::<_, Transaction>(
        "UPDATE transactions 
         SET amount = COALESCE($1, amount),
             category = COALESCE($2, category),
             description = COALESCE($3, description),
             wallet_id = COALESCE($4, wallet_id),
             updated_at = $5
         WHERE id = $6 AND user_id = $7
         RETURNING id, user_id, wallet_id, amount, transaction_type, category, description, created_at, updated_at"
    )
    .bind(req.amount)
    .bind(&req.category)
    .bind(&req.description)
    .bind(&new_wallet_id)
    .bind(now)
    .bind(&transaction_id)
    .bind(&user_id);

    match query.fetch_one(&mut *tx).await {
        Ok(transaction) => {
            if let Err(e) = tx.commit().await {
                log::error!("Failed to commit transaction: {}", e);
                return HttpResponse::InternalServerError()
                    .json(ApiResponse::<Transaction>::error("Failed to save changes".to_string()));
            }

            let _ = invalidate_cache_pattern(&cache.get_ref(), &format!("transaction*:{}*", user_id)).await;
            let _ = invalidate_cache_pattern(&cache.get_ref(), &format!("wallet{}:*", user_id)).await;
            let _ = invalidate_cache_pattern(&cache.get_ref(), &format!("wallets:{}*", user_id)).await;

            HttpResponse::Ok().json(ApiResponse::success(transaction))
        }
        Err(e) => {
            let _ = tx.rollback().await;
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

    // Fetch transaction to reverse wallet balance
    let transaction = match sqlx::query_as::<_, Transaction>(
        "SELECT id, user_id, wallet_id, amount, transaction_type, category, description, created_at, updated_at FROM transactions WHERE id = $1 AND user_id = $2"
    )
    .bind(&transaction_id)
    .bind(&user_id)
    .fetch_optional(db.get_ref())
    .await {
        Ok(Some(tx)) => tx,
        Ok(None) => {
            return HttpResponse::NotFound()
                .json(ApiResponse::<String>::error("Transaction not found".to_string()));
        }
        Err(e) => {
            log::error!("Error fetching transaction: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error("Database error".to_string()));
        }
    };

    // Start database transaction
    let mut tx = match db.begin().await {
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to begin transaction: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error("Database error".to_string()));
        }
    };

    // Reverse wallet balance
    if let Some(wallet_id) = &transaction.wallet_id {
        let amount_delta = match transaction.transaction_type.as_str() {
            "income" => -transaction.amount,
            "expense" => transaction.amount,
            _ => {
                let _ = tx.rollback().await;
                return HttpResponse::InternalServerError()
                    .json(ApiResponse::<String>::error("Invalid transaction type".to_string()));
            }
        };

        if let Err(e) = sqlx::query("UPDATE wallets SET balance = balance + $1 WHERE id = $2")
            .bind(amount_delta)
            .bind(wallet_id)
            .execute(&mut *tx)
            .await
        {
            log::error!("Error reversing wallet balance: {}", e);
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error("Failed to update wallet balance".to_string()));
        }
    }

    // Delete transaction
    let result = sqlx::query("DELETE FROM transactions WHERE id = $1 AND user_id = $2")
        .bind(&transaction_id)
        .bind(&user_id)
        .execute(&mut *tx)
        .await;

    match result {
        Ok(query_result) => {
            if query_result.rows_affected() > 0 {
                if let Err(e) = tx.commit().await {
                    log::error!("Failed to commit transaction: {}", e);
                    return HttpResponse::InternalServerError()
                        .json(ApiResponse::<String>::error("Failed to save changes".to_string()));
                }

                let _ = invalidate_cache_pattern(&cache.get_ref(), &format!("transaction*:{}*", user_id)).await;
                let _ = invalidate_cache_pattern(&cache.get_ref(), &format!("wallet{}:*", user_id)).await;
                let _ = invalidate_cache_pattern(&cache.get_ref(), &format!("wallets:{}*", user_id)).await;
                HttpResponse::NoContent().finish()
            } else {
                let _ = tx.rollback().await;
                HttpResponse::NotFound()
                    .json(ApiResponse::<String>::error("Transaction not found".to_string()))
            }
        }
        Err(e) => {
            let _ = tx.rollback().await;
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
    sqlx::query_as::<_, Transaction>(
        "SELECT id, user_id, wallet_id, amount, transaction_type, category, description, created_at, updated_at FROM transactions WHERE user_id = $1 ORDER BY created_at DESC"
    )
        .bind(user_id)
        .fetch_all(pool)
        .await
}

async fn fetch_transaction_by_id(
    pool: &PgPool,
    transaction_id: &str,
    user_id: &str,
) -> Result<Transaction, sqlx::Error> {
    sqlx::query_as::<_, Transaction>(
        "SELECT id, user_id, wallet_id, amount, transaction_type, category, description, created_at, updated_at FROM transactions WHERE id = $1 AND user_id = $2"
    )
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
