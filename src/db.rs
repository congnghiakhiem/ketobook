use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub struct DbPool(pub PgPool);

impl DbPool {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        // Run migrations (optional)
        // sqlx::migrate!().run(&pool).await?;

        Ok(DbPool(pool))
    }

    pub fn get_pool(&self) -> &PgPool {
        &self.0
    }
}

// Database initialization helper
pub async fn init_database(database_url: &str) -> Result<DbPool, sqlx::Error> {
    DbPool::new(database_url).await
}
