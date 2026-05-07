/// Feishu API error types

use thiserror::Error;
use serde::Deserialize;

#[derive(Error, Debug, Clone)]
pub enum FeishuError {
    #[error("API error: {code} - {msg}")]
    ApiError { code: i32, msg: String },

    #[error("Token expired, need refresh")]
    TokenExpired,

    #[error("Rate limit exceeded, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Feishu API error response structure
#[derive(Debug, Deserialize)]
pub struct FeishuApiErrorResponse {
    pub code: i32,
    pub msg: String,
}

impl FeishuError {
    /// Create FeishuError from a reqwest response
    pub async fn from_response(response: reqwest::Response) -> Self {
        let status = response.status();

        // Check for specific HTTP status codes
        match status.as_u16() {
            401 => return FeishuError::TokenExpired,
            404 => return FeishuError::NotFound("Resource not found".to_string()),
            429 => {
                // Try to get retry-after header
                let retry_after = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(60);
                return FeishuError::RateLimited { retry_after_secs: retry_after };
            }
            _ => {}
        }

        // Try to parse the error response body
        if let Ok(err_resp) = response.json::<FeishuApiErrorResponse>().await {
            match err_resp.code {
                99991663 | 99991664 | 99991665 => FeishuError::TokenExpired,
                0 => FeishuError::ApiError { code: 0, msg: "Success".to_string() },
                _ => FeishuError::ApiError { code: err_resp.code, msg: err_resp.msg },
            }
        } else {
            // Fallback: create error from status
            FeishuError::ApiError {
                code: status.as_u16() as i32,
                msg: format!("HTTP {}", status),
            }
        }
    }
}

impl From<reqwest::Error> for FeishuError {
    fn from(err: reqwest::Error) -> Self {
        FeishuError::NetworkError(err.to_string())
    }
}