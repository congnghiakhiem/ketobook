use redis::aio::ConnectionManager;
use redis::Client;

#[derive(Clone)]
pub struct CacheManager(pub ConnectionManager);

impl CacheManager {
    pub async fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        let conn_manager = ConnectionManager::new(client).await?;
        Ok(CacheManager(conn_manager))
    }

    pub fn get_connection_manager(&self) -> &ConnectionManager {
        &self.0
    }
}

// Cache-Aside Pattern Implementation
pub async fn get_or_set_cache<T: serde::Serialize + serde::de::DeserializeOwned>(
    cache: &ConnectionManager,
    key: &str,
    fetch_fn: impl std::future::Future<Output = Result<T, sqlx::Error>>,
) -> Result<T, CacheError> {
    use redis::AsyncCommands;
    let mut cache = cache.clone();

    // Try to get from cache
    match cache.get::<&str, String>(key).await {
        Ok(cached_data) => {
            if let Ok(data) = serde_json::from_str::<T>(&cached_data) {
                log::info!("Cache hit for key: {}", key);
                return Ok(data);
            }
        }
        Err(redis::RedisError { .. }) => {
            log::debug!("Cache miss for key: {}", key);
        }
    }

    // Fetch from database
    let data = fetch_fn.await.map_err(CacheError::DatabaseError)?;

    // Store in cache (with 1 hour TTL)
    let json_data = serde_json::to_string(&data).map_err(CacheError::SerializationError)?;
    let _: () = cache
        .set_ex(key, json_data, 3600)
        .await
        .map_err(CacheError::CacheError)?;

    log::info!("Data cached for key: {}", key);
    Ok(data)
}

// Invalidate cache by key
pub async fn invalidate_cache(cache: &ConnectionManager, key: &str) -> Result<(), redis::RedisError> {
    use redis::AsyncCommands;
    let mut cache = cache.clone();
    let _: () = cache.del(key).await?;
    log::info!("Cache invalidated for key: {}", key);
    Ok(())
}

// Invalidate cache by pattern
pub async fn invalidate_cache_pattern(
    cache: &ConnectionManager,
    pattern: &str,
) -> Result<(), redis::RedisError> {
    use redis::AsyncCommands;
    let mut cache = cache.clone();
    let keys: Vec<String> = cache.keys(pattern).await?;
    if !keys.is_empty() {
        let _: () = cache.del(keys).await?;
        log::info!("Cache invalidated for pattern: {}", pattern);
    }
    Ok(())
}

// Invalidate all cache for a user (transactions and wallets)
pub async fn invalidate_user_cache(
    cache: &ConnectionManager,
    user_id: &str,
) -> Result<(), redis::RedisError> {
    let patterns = vec![
        format!("transactions:{}*", user_id),
        format!("transaction{}:*", user_id),
        format!("wallets:{}*", user_id),
        format!("wallet{}:*", user_id),
        format!("wallet:{}*", user_id),
    ];

    for pattern in patterns {
        invalidate_cache_pattern(cache, &pattern).await?;
    }

    log::info!("All cache invalidated for user: {}", user_id);
    Ok(())
}

#[derive(Debug)]
pub enum CacheError {
    CacheError(redis::RedisError),
    DatabaseError(sqlx::Error),
    SerializationError(serde_json::Error),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::CacheError(e) => write!(f, "Cache error: {}", e),
            CacheError::DatabaseError(e) => write!(f, "Database error: {}", e),
            CacheError::SerializationError(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl std::error::Error for CacheError {}
