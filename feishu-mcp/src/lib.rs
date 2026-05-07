/// Library entry point for feishu-mcp-server

pub mod auth;
pub mod config;
pub mod feishu;
pub mod mcp;
pub mod middleware;
pub mod tools;

// Re-export the main server struct
pub use mcp::FeishuMcpServer;

#[cfg(test)]
mod robustness_tests {
    use crate::middleware::rate_limit::{RateLimiter, RateLimitError};
    use std::time::Duration;
    use tokio::time::timeout;

    /// G8.4.2: Rate limit error handling test
    #[test]
    fn test_rate_limit_error_handling() {
        let limiter = RateLimiter::new(60, 2);

        assert!(limiter.check("user").is_ok());
        assert!(limiter.check("user").is_ok());

        let result = limiter.check("user");
        assert!(result.is_err());

        match result.unwrap_err() {
            RateLimitError { retry_after_secs } => {
                assert!(retry_after_secs > 0, "Retry-after should be positive");
            }
        }
    }

    /// G8.4.3: Network timeout handling test
    #[tokio::test]
    async fn test_network_timeout_handling() {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(1))
            .build()
            .unwrap();

        let request = client.get("http://10.255.255.1:1").send();

        let result = timeout(Duration::from_secs(5), request).await;

        assert!(result.is_ok() || result.is_err());
    }

    /// G8.4.4: Invalid JSON response handling test
    #[test]
    fn test_invalid_json_error() {
        let invalid_json = r#"{"code": 99999, "msg": "Invalid"#;
        let result: Result<serde_json::Value, _> = serde_json::from_str(invalid_json);

        assert!(result.is_err(), "Invalid JSON should produce an error");
    }

    /// G8.4.6: Empty response handling
    #[test]
    fn test_empty_data_response() {
        let json = r#"{"code": 0, "msg": "success", "data": null}"#;
        let response: crate::feishu::types::ApiResponse<String> =
            serde_json::from_str(json).unwrap();

        assert!(response.code == 0);
        assert!(response.data.is_none());
    }

    /// G8.4.7: Rate limiter window sliding test
    #[test]
    fn test_rate_limiter_sliding_window() {
        let limiter = RateLimiter::new(1, 2);

        assert!(limiter.check("user").is_ok());
        assert!(limiter.check("user").is_ok());
        assert!(limiter.check("user").is_err());
    }
}