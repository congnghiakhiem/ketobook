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

    // Initialize Redis cache manager
    let cache_manager = CacheManager::new(&config.redis_url)
        .await
        .expect("Failed to initialize Redis cache");
    log::info!("Redis cache initialized successfully");

    let server_address = config.server_address();
    log::info!("Starting server on {}", server_address);

    // Create and start HTTP server
    HttpServer::new(move || {
        App::new()
            // Add logging middleware
            .wrap(middleware::Logger::default())
            // Share database pool and cache manager across requests
            .app_data(web::Data::new(db_pool.get_pool().clone()))
            .app_data(web::Data::new(cache_manager.get_connection_manager().clone()))
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
