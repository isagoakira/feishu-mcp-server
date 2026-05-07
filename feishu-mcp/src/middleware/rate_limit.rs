/// Rate limiting middleware using sliding window algorithm

use dashmap::DashMap;
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, thiserror::Error)]
#[error("Rate limit exceeded, retry after {retry_after_secs}s")]
pub struct RateLimitError {
    pub retry_after_secs: u64,
}

pub struct RateLimiter {
    window_secs: u64,
    max_requests: u64,
    requests: DashMap<String, VecDeque<u64>>,
}

impl RateLimiter {
    pub fn new(window_secs: u64, max_requests: u64) -> Self {
        Self {
            window_secs,
            max_requests,
            requests: DashMap::new(),
        }
    }

    /// Check if request is allowed under rate limit
    /// Returns Ok(()) if allowed, Err(RateLimitError) if exceeded
    pub fn check(&self, user_id: &str) -> Result<(), RateLimitError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let window_start = now.saturating_sub(self.window_secs);

        // Get or create the user's request queue
        let mut queue = self.requests.entry(user_id.to_string()).or_insert_with(VecDeque::new);

        // Remove expired entries (outside the sliding window)
        while let Some(&oldest) = queue.front() {
            if oldest <= window_start {
                queue.pop_front();
            } else {
                break;
            }
        }

        // Check if current window has room
        if queue.len() >= self.max_requests as usize {
            // Calculate retry_after based on the oldest request in the window
            let retry_after_secs = queue
                .front()
                .map(|&oldest| oldest + self.window_secs - now)
                .unwrap_or(self.window_secs);
            return Err(RateLimitError { retry_after_secs });
        }

        // Add current request timestamp
        queue.push_back(now);
        Ok(())
    }

    /// Get current request count for a user (for testing/monitoring)
    #[allow(dead_code)]
    pub fn current_count(&self, user_id: &str) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let window_start = now.saturating_sub(self.window_secs);

        self.requests
            .get(user_id)
            .map(|queue| {
                queue.iter().filter(|&&ts| ts > window_start).count() as u64
            })
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_rate_limiter_allows_requests_under_limit() {
        let limiter = RateLimiter::new(60, 5);

        // First 5 requests should succeed
        for i in 0..5 {
            let result = limiter.check(&format!("user_{}", i % 3));
            assert!(result.is_ok(), "Request {} should be allowed", i + 1);
        }
    }

    #[test]
    fn test_rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new(60, 3);

        // Make 3 requests for the same user
        for _ in 0..3 {
            assert!(limiter.check("user_a").is_ok());
        }

        // 4th request should be blocked
        let result = limiter.check("user_a");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.retry_after_secs, 60);
    }

    #[test]
    fn test_rate_limiter_per_user_isolation() {
        let limiter = RateLimiter::new(60, 2);

        // user_a makes 2 requests (limit reached)
        assert!(limiter.check("user_a").is_ok());
        assert!(limiter.check("user_a").is_ok());
        assert!(limiter.check("user_a").is_err());

        // user_b should still be able to make requests
        assert!(limiter.check("user_b").is_ok());
        assert!(limiter.check("user_b").is_ok());
        assert!(limiter.check("user_b").is_err());
    }

    #[test]
    fn test_window_sliding_effect() {
        // Use a very short window for testing
        let limiter = RateLimiter::new(1, 2);

        // Make 2 requests
        assert!(limiter.check("user").is_ok());
        assert!(limiter.check("user").is_ok());
        assert!(limiter.check("user").is_err());

        // Simulate time passing (wait 1.1 seconds)
        thread::sleep(Duration::from_millis(1100));

        // Window should have slid, allowing new requests
        assert!(limiter.check("user").is_ok());
        assert!(limiter.check("user").is_ok());
        assert!(limiter.check("user").is_err());
    }

    #[test]
    fn test_concurrent_requests() {
        let _limiter = RateLimiter::new(60, 10);
        let mut handles = vec![];

        for i in 0..20 {
            let limiter = RateLimiter::new(60, 10);
            let user_id = format!("user_{}", i % 5);

            let handle = thread::spawn(move || {
                // Each thread makes a request
                limiter.check(&user_id)
            });
            handles.push(handle);
        }

        // All requests should complete without panic
        for handle in handles {
            let _ = handle.join();
        }
    }

    #[test]
    fn test_empty_queue_for_unknown_user() {
        let limiter = RateLimiter::new(60, 5);

        // Unknown user should have 0 requests
        let count = limiter.current_count("unknown_user");
        assert_eq!(count, 0);

        // And should be allowed
        assert!(limiter.check("unknown_user").is_ok());
    }
}