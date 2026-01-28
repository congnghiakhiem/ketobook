mod cache;
mod config;
mod db;
mod debts;
mod models;
mod transactions;
mod wallets;

use actix_web::{web, App, HttpServer, middleware};
use cache::CacheManager;
use config::AppConfig;
use db::DbPool;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Load configuration from .env
    let config = AppConfig::from_env();
    log::info!("Loaded configuration: {:?}", config);

    // Initialize database connection pool
    let db_pool = DbPool::new(&config.database_url)
        .await
        .expect("Failed to initialize database pool");
    log::info!("Database pool initialized successfully");

    // Initialize Redis cache manager (optional - continue without cache if connection fails)
    let cache_manager = match CacheManager::new(&config.redis_url).await {
        Ok(cache) => {
            log::info!("Redis cache initialized successfully");
            Some(cache)
        }
        Err(e) => {
            log::warn!("Failed to initialize Redis cache: {}. Continuing without cache.", e);
            None
        }
    };

    let server_address = config.server_address();
    log::info!("Starting server on {}", server_address);

    // Create and start HTTP server
    HttpServer::new(move || {
        let mut app = App::new()
            // Add logging middleware
            .wrap(middleware::Logger::default())
            // Share database pool across requests
            .app_data(web::Data::new(db_pool.get_pool().clone()));

        // Add cache manager if available
        if let Some(ref cache) = cache_manager {
            app = app.app_data(web::Data::new(cache.get_connection_manager().clone()));
        }

        app
            // Health check endpoint
            .route("/health", web::get().to(health_check))
            // Configure wallet routes
            .configure(wallets::configure_routes)
            // Configure transaction routes
            .configure(transactions::configure_routes)
            // Configure debt routes
            .configure(debts::configure_routes)
    })
    .bind(&server_address)?
    .run()
    .await
}

/// Health check endpoint
async fn health_check() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
