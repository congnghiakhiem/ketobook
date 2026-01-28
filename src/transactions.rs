use actix_web::{web, HttpResponse};
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use sqlx::types::BigDecimal;
use std::str::FromStr;

use crate::models::{ApiResponse, CreateTransactionRequest, Transaction, UpdateTransactionRequest, Wallet, WalletType};
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

/// Create a new transaction with atomic balance updates
pub async fn create_transaction(
    req: web::Json<CreateTransactionRequest>,
    db: web::Data<PgPool>,
    cache: web::Data<ConnectionManager>,
) -> HttpResponse {
    let transaction_id = Uuid::new_v4().to_string();
    let now = Utc::now();

    // Fetch wallet to validate and check balance
    let wallet: Option<Wallet> = match sqlx::query_as::<_, Wallet>(
        "SELECT id, user_id, name, balance, credit_limit, wallet_type, created_at, updated_at FROM wallets WHERE id = $1 AND user_id = $2"
    )
    .bind(&req.wallet_id)
    .bind(&req.user_id)
    .fetch_optional(db.get_ref())
    .await {
        Ok(w) => w,
        Err(e) => {
            log::error!("Error fetching wallet: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<Transaction>::error("Failed to validate wallet".to_string()));
        }
    };

    let wallet = match wallet {
        Some(w) => w,
        None => {
            return HttpResponse::BadRequest()
                .json(ApiResponse::<Transaction>::error("Wallet not found or doesn't belong to user".to_string()));
        }
    };

    // Validate transaction type
    if req.transaction_type != "income" && req.transaction_type != "expense" {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<Transaction>::error("Invalid transaction type. Must be 'income' or 'expense'".to_string()));
    }

    // Validate amount is positive
    if req.amount <= BigDecimal::from_str("0").unwrap() {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<Transaction>::error("Amount must be greater than 0".to_string()));
    }

    // Balance validation for expenses
    if req.transaction_type == "expense" {
        let wallet_type = WalletType::from_str(&wallet.wallet_type).unwrap_or(WalletType::Other);
        
        match wallet_type {
            WalletType::CreditCard => {
                // For credit cards: check available credit (credit_limit - balance)
                if let Some(limit) = &wallet.credit_limit {
                    let available = limit - &wallet.balance;
                    if req.amount > available {
                        return HttpResponse::BadRequest()
                            .json(ApiResponse::<Transaction>::error(
                                format!("Insufficient credit. Available: {}, Required: {}", available, req.amount)
                            ));
                    }
                } else {
                    return HttpResponse::InternalServerError()
                        .json(ApiResponse::<Transaction>::error("Credit card missing credit limit".to_string()));
                }
            }
            _ => {
                // For other wallets: balance cannot go negative
                if req.amount > wallet.balance {
                    return HttpResponse::BadRequest()
                        .json(ApiResponse::<Transaction>::error(
                            format!("Insufficient balance. Available: {}, Required: {}", wallet.balance, req.amount)
                        ));
                }
            }
        }
    }

    // Start database transaction (BEGIN/COMMIT)
    let mut db_tx = match db.begin().await {
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to begin database transaction: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<Transaction>::error("Database error".to_string()));
        }
    };

    // Insert transaction record
    let insert_result = sqlx::query_as::<_, Transaction>(
        "INSERT INTO transactions (id, user_id, wallet_id, amount, transaction_type, category, description, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
         RETURNING id, user_id, wallet_id, amount, transaction_type, category, description, created_at, updated_at"
    )
    .bind(&transaction_id)
    .bind(&req.user_id)
    .bind(&req.wallet_id)
    .bind(&req.amount)
    .bind(&req.transaction_type)
    .bind(&req.category)
    .bind(&req.description)
    .bind(now)
    .bind(now)
    .fetch_one(&mut *db_tx)
    .await;

    let transaction = match insert_result {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Error inserting transaction: {}", e);
            let _ = db_tx.rollback().await;
            return HttpResponse::BadRequest()
                .json(ApiResponse::<Transaction>::error("Failed to create transaction".to_string()));
        }
    };

    // Calculate balance delta
    let balance_delta = match req.transaction_type.as_str() {
        "income" => req.amount.clone(),
        "expense" => -req.amount.clone(),
        _ => {
            let _ = db_tx.rollback().await;
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<Transaction>::error("Invalid transaction type".to_string()));
        }
    };

    // Update wallet balance atomically
    let update_result = sqlx::query("UPDATE wallets SET balance = balance + $1 WHERE id = $2")
        .bind(&balance_delta)
        .bind(&req.wallet_id)
        .execute(&mut *db_tx)
        .await;

    match update_result {
        Ok(_) => {},
        Err(e) => {
            log::error!("Error updating wallet balance: {}", e);
            let _ = db_tx.rollback().await;
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<Transaction>::error("Failed to update wallet balance".to_string()));
        }
    }

    // Commit database transaction
    if let Err(e) = db_tx.commit().await {
        log::error!("Failed to commit database transaction: {}", e);
        return HttpResponse::InternalServerError()
            .json(ApiResponse::<Transaction>::error("Failed to save changes".to_string()));
    }

    // Invalidate caches (specific wallet + all user transactions)
    let mut cache_clone = cache.get_ref().clone();
    let _ = invalidate_cache_pattern(&mut cache_clone, &format!("wallet:{}:{}*", req.user_id, req.wallet_id)).await;
    let _ = invalidate_cache_pattern(&mut cache_clone, &format!("wallets:{}*", req.user_id)).await;
    let _ = invalidate_cache_pattern(&mut cache_clone, &format!("transactions:{}*", req.user_id)).await;

    HttpResponse::Created().json(ApiResponse::success(transaction))
}

/// Update a transaction with balance adjustments
pub async fn update_transaction(
    path: web::Path<(String, String)>,
    req: web::Json<UpdateTransactionRequest>,
    db: web::Data<PgPool>,
    cache: web::Data<ConnectionManager>,
) -> HttpResponse {
    let (user_id, transaction_id) = path.into_inner();
    let now = Utc::now();

    // Fetch current transaction
    let current_tx: Option<Transaction> = match sqlx::query_as::<_, Transaction>(
        "SELECT id, user_id, wallet_id, amount, transaction_type, category, description, created_at, updated_at FROM transactions WHERE id = $1 AND user_id = $2"
    )
    .bind(&transaction_id)
    .bind(&user_id)
    .fetch_optional(db.get_ref())
    .await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Error fetching transaction: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<Transaction>::error("Database error".to_string()));
        }
    };

    let current_tx = match current_tx {
        Some(tx) => tx,
        None => {
            return HttpResponse::NotFound()
                .json(ApiResponse::<Transaction>::error("Transaction not found".to_string()));
        }
    };

    // Determine new wallet and amount
    let new_wallet_id = req.wallet_id.clone().unwrap_or_else(|| current_tx.wallet_id.clone().unwrap());
    let new_amount = req.amount.clone().unwrap_or_else(|| current_tx.amount.clone());

    // Validate new amount if changed
    if req.amount.is_some() && new_amount <= BigDecimal::from_str("0").unwrap() {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<Transaction>::error("Amount must be greater than 0".to_string()));
    }

    // Start database transaction
    let mut db_tx = match db.begin().await {
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to begin transaction: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<Transaction>::error("Database error".to_string()));
        }
    };

    // If wallet or amount changed, reverse old balance and validate new balance
    if new_wallet_id != *current_tx.wallet_id.as_ref().unwrap_or(&"".to_string()) || req.amount.is_some() {
        // Reverse old wallet balance
        let old_wallet_id = current_tx.wallet_id.clone().unwrap();
        let reverse_delta = match current_tx.transaction_type.as_str() {
            "income" => -current_tx.amount.clone(),
            "expense" => current_tx.amount.clone(),
            _ => {
                let _ = db_tx.rollback().await;
                return HttpResponse::InternalServerError()
                    .json(ApiResponse::<Transaction>::error("Invalid transaction type".to_string()));
            }
        };

        if let Err(e) = sqlx::query("UPDATE wallets SET balance = balance + $1 WHERE id = $2")
            .bind(&reverse_delta)
            .bind(&old_wallet_id)
            .execute(&mut *db_tx)
            .await
        {
            log::error!("Error reversing old wallet balance: {}", e);
            let _ = db_tx.rollback().await;
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<Transaction>::error("Failed to reverse old balance".to_string()));
        }

        // Check new wallet balance if amount is changing and it's an expense
        if current_tx.transaction_type == "expense" && req.amount.is_some() {
            let new_wallet: Option<Wallet> = match sqlx::query_as::<_, Wallet>(
                "SELECT id, user_id, name, balance, credit_limit, wallet_type, created_at, updated_at FROM wallets WHERE id = $1"
            )
            .bind(&new_wallet_id)
            .fetch_optional(&mut *db_tx)
            .await {
                Ok(w) => w,
                Err(e) => {
                    log::error!("Error fetching new wallet: {}", e);
                    let _ = db_tx.rollback().await;
                    return HttpResponse::InternalServerError()
                        .json(ApiResponse::<Transaction>::error("Failed to validate wallet".to_string()));
                }
            };

            if let Some(wallet) = new_wallet {
                let wallet_type = WalletType::from_str(&wallet.wallet_type).unwrap_or(WalletType::Other);
                match wallet_type {
                    WalletType::CreditCard => {
                        if let Some(limit) = &wallet.credit_limit {
                            let available = limit - &wallet.balance;
                            if new_amount > available {
                                let _ = db_tx.rollback().await;
                                return HttpResponse::BadRequest()
                                    .json(ApiResponse::<Transaction>::error(
                                        format!("Insufficient credit. Available: {}", available)
                                    ));
                            }
                        }
                    }
                    _ => {
                        if new_amount > wallet.balance {
                            let _ = db_tx.rollback().await;
                            return HttpResponse::BadRequest()
                                .json(ApiResponse::<Transaction>::error(
                                    format!("Insufficient balance. Available: {}", wallet.balance)
                                ));
                        }
                    }
                }
            }
        }

        // Apply new wallet balance
        let new_delta = match current_tx.transaction_type.as_str() {
            "income" => new_amount.clone(),
            "expense" => -new_amount.clone(),
            _ => {
                let _ = db_tx.rollback().await;
                return HttpResponse::InternalServerError()
                    .json(ApiResponse::<Transaction>::error("Invalid transaction type".to_string()));
            }
        };

        if let Err(e) = sqlx::query("UPDATE wallets SET balance = balance + $1 WHERE id = $2")
            .bind(&new_delta)
            .bind(&new_wallet_id)
            .execute(&mut *db_tx)
            .await
        {
            log::error!("Error applying new wallet balance: {}", e);
            let _ = db_tx.rollback().await;
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<Transaction>::error("Failed to apply new balance".to_string()));
        }
    }

    // Update transaction
    let update_result = sqlx::query_as::<_, Transaction>(
        "UPDATE transactions 
         SET amount = $1, category = COALESCE($2, category), description = COALESCE($3, description), wallet_id = $4, updated_at = $5
         WHERE id = $6 AND user_id = $7
         RETURNING id, user_id, wallet_id, amount, transaction_type, category, description, created_at, updated_at"
    )
    .bind(&new_amount)
    .bind(&req.category)
    .bind(&req.description)
    .bind(&new_wallet_id)
    .bind(now)
    .bind(&transaction_id)
    .bind(&user_id)
    .fetch_one(&mut *db_tx)
    .await;

    let updated_tx = match update_result {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Error updating transaction: {}", e);
            let _ = db_tx.rollback().await;
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<Transaction>::error("Failed to update transaction".to_string()));
        }
    };

    // Commit transaction
    if let Err(e) = db_tx.commit().await {
        log::error!("Failed to commit transaction: {}", e);
        return HttpResponse::InternalServerError()
            .json(ApiResponse::<Transaction>::error("Failed to save changes".to_string()));
    }

    // Invalidate caches
    let mut cache_clone = cache.get_ref().clone();
    let _ = invalidate_cache_pattern(&mut cache_clone, &format!("wallet*{}*", user_id)).await;
    let _ = invalidate_cache_pattern(&mut cache_clone, &format!("wallets:{}*", user_id)).await;
    let _ = invalidate_cache_pattern(&mut cache_clone, &format!("transactions:{}*", user_id)).await;
    let _ = invalidate_cache_pattern(&mut cache_clone, &format!("transaction:{}*", user_id)).await;

    HttpResponse::Ok().json(ApiResponse::success(updated_tx))
}

/// Delete a transaction and reverse wallet balance
pub async fn delete_transaction(
    path: web::Path<(String, String)>,
    db: web::Data<PgPool>,
    cache: web::Data<ConnectionManager>,
) -> HttpResponse {
    let (user_id, transaction_id) = path.into_inner();

    // Fetch transaction to reverse balance
    let transaction: Option<Transaction> = match sqlx::query_as::<_, Transaction>(
        "SELECT id, user_id, wallet_id, amount, transaction_type, category, description, created_at, updated_at FROM transactions WHERE id = $1 AND user_id = $2"
    )
    .bind(&transaction_id)
    .bind(&user_id)
    .fetch_optional(db.get_ref())
    .await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Error fetching transaction: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error("Database error".to_string()));
        }
    };

    let transaction = match transaction {
        Some(tx) => tx,
        None => {
            return HttpResponse::NotFound()
                .json(ApiResponse::<String>::error("Transaction not found".to_string()));
        }
    };

    // Start database transaction
    let mut db_tx = match db.begin().await {
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to begin transaction: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error("Database error".to_string()));
        }
    };

    // Reverse wallet balance
    if let Some(wallet_id) = &transaction.wallet_id {
        let delta = match transaction.transaction_type.as_str() {
            "income" => -transaction.amount.clone(),
            "expense" => transaction.amount.clone(),
            _ => {
                let _ = db_tx.rollback().await;
                return HttpResponse::InternalServerError()
                    .json(ApiResponse::<String>::error("Invalid transaction type".to_string()));
            }
        };

        if let Err(e) = sqlx::query("UPDATE wallets SET balance = balance + $1 WHERE id = $2")
            .bind(&delta)
            .bind(wallet_id)
            .execute(&mut *db_tx)
            .await
        {
            log::error!("Error reversing wallet balance: {}", e);
            let _ = db_tx.rollback().await;
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error("Failed to reverse balance".to_string()));
        }
    }

    // Delete transaction
    let delete_result = sqlx::query("DELETE FROM transactions WHERE id = $1 AND user_id = $2")
        .bind(&transaction_id)
        .bind(&user_id)
        .execute(&mut *db_tx)
        .await;

    match delete_result {
        Ok(result) => {
            if result.rows_affected() > 0 {
                if let Err(e) = db_tx.commit().await {
                    log::error!("Failed to commit transaction: {}", e);
                    return HttpResponse::InternalServerError()
                        .json(ApiResponse::<String>::error("Failed to save changes".to_string()));
                }

                // Invalidate caches
                let mut cache_clone = cache.get_ref().clone();
                let _ = invalidate_cache_pattern(&mut cache_clone, &format!("wallet*{}*", user_id)).await;
                let _ = invalidate_cache_pattern(&mut cache_clone, &format!("wallets:{}*", user_id)).await;
                let _ = invalidate_cache_pattern(&mut cache_clone, &format!("transactions:{}*", user_id)).await;
                let _ = invalidate_cache_pattern(&mut cache_clone, &format!("transaction:{}*", user_id)).await;

                HttpResponse::NoContent().finish()
            } else {
                let _ = db_tx.rollback().await;
                HttpResponse::NotFound()
                    .json(ApiResponse::<String>::error("Transaction not found".to_string()))
            }
        }
        Err(e) => {
            let _ = db_tx.rollback().await;
            log::error!("Error deleting transaction: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error("Failed to delete transaction".to_string()))
        }
    }
}

// ==================== Database Functions ====================

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
