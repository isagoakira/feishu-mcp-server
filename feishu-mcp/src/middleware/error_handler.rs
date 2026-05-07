/// Unified error handling middleware

use crate::feishu::error::FeishuError;

pub enum ToolError {
    UserError(String),
    RetryableError(String),
    RateLimit { retry_after_secs: u64 },
    InternalError(String),
}

/// Convert FeishuError to ToolError
/// - Token expired → RetryableError (will trigger auto-refresh in caller)
/// - Rate limit → RateLimit
/// - Feishu 5xx → RetryableError
/// - Other errors → UserError
pub fn handle_error(e: &FeishuError) -> ToolError {
    match e {
        FeishuError::RateLimited { retry_after_secs } => {
            ToolError::RateLimit { retry_after_secs: *retry_after_secs }
        }
        FeishuError::TokenExpired => ToolError::RetryableError("Token expired".to_string()),
        FeishuError::NotFound(msg) => ToolError::UserError(format!("Not found: {}", msg)),
        FeishuError::InternalError(msg) => ToolError::InternalError(msg.clone()),
        FeishuError::ApiError { code, msg } => {
            if *code >= 500 {
                ToolError::RetryableError(format!("Server error: {} - {}", code, msg))
            } else {
                ToolError::UserError(format!("API error: {} - {}", code, msg))
            }
        }
        FeishuError::NetworkError(e) => {
            ToolError::RetryableError(format!("Network error: {}", e))
        }
    }
}

impl From<FeishuError> for ToolError {
    fn from(err: FeishuError) -> Self {
        handle_error(&err)
    }
}