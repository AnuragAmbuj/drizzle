use anyhow::Result;
use async_trait::async_trait;
use dashmap::DashMap;
use domain::policy::LimitPolicy;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check if the request is allowed.
    /// Returns:
    /// - Ok(true): Allowed
    /// - Ok(false): Limited (429)
    async fn check(&self, key: &str, policy: &LimitPolicy) -> Result<bool>;
}

/// A simple in-memory Token Bucket implementation.
/// Buckets are stored in a DashMap: Key -> Mutex<BucketState>
pub struct TokenBucketRateLimiter {
    buckets: Arc<DashMap<String, Mutex<BucketState>>>,
}

struct BucketState {
    tokens: f64,
    last_refill: Instant,
}

impl TokenBucketRateLimiter {
    pub fn new() -> Self {
        Self {
            buckets: Arc::new(DashMap::new()),
        }
    }
}

#[async_trait]
impl RateLimiter for TokenBucketRateLimiter {
    async fn check(&self, key: &str, policy: &LimitPolicy) -> Result<bool> {
        // Basic Token Bucket Logic
        // Rate: tokens per second
        // Burst: max tokens

        let now = Instant::now();
        let rate = policy.rate as f64;
        let capacity = policy.burst as f64;

        // 1. Get or Create bucket
        // We use dashmap entry API, but since the value is a Mutex, we need to lock it.
        // DashMap entry API with async mutex is tricky, simpler to just get or insert.

        let bucket_mutex = self.buckets.entry(key.to_string()).or_insert_with(|| {
            Mutex::new(BucketState {
                tokens: capacity, // Start full? Or empty? Usually full for new users.
                last_refill: now,
            })
        });

        let mut bucket = bucket_mutex.lock().await;

        // 2. Refill
        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        let new_tokens = elapsed * rate;

        if new_tokens > 0.0 {
            bucket.tokens = (bucket.tokens + new_tokens).min(capacity);
            bucket.last_refill = now;
        }

        // 3. Consume
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
