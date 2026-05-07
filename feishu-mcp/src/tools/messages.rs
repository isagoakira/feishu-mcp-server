/// Message tools for Feishu MCP server

use crate::feishu::client::FeishuClient;
use crate::feishu::error::FeishuError;
use crate::feishu::types::Message;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Send a message to a chat
pub async fn send_message(
    client: Arc<FeishuClient>,
    chat_id: String,
    content: String,
    msg_type: Option<String>,
) -> Result<Message, FeishuError> {
    let msg_type = msg_type.unwrap_or_else(|| "text".to_string());
    client.send_message(&chat_id, &content, &msg_type).await
}

/// Get messages from a chat
pub async fn get_messages(
    client: Arc<FeishuClient>,
    chat_id: String,
    limit: Option<u32>,
) -> Result<Vec<Message>, FeishuError> {
    client.get_messages(&chat_id, limit.map(|l| l as i32)).await
}

/// Search messages
pub async fn search_messages(
    client: Arc<FeishuClient>,
    query: String,
    chat_id: Option<String>,
) -> Result<Vec<Message>, FeishuError> {
    client.search_messages(&query, chat_id.as_deref()).await
}

// ==================== Response Types for Tools ====================

/// Response type for send_message tool
#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub message_id: String,
    pub chat_id: String,
    pub content: String,
    pub msg_type: String,
}

impl From<Message> for SendMessageResponse {
    fn from(msg: Message) -> Self {
        Self {
            message_id: msg.message_id,
            chat_id: msg.chat_id,
            content: msg.content,
            msg_type: msg.msg_type,
        }
    }
}

/// Response type for get_messages tool
#[derive(Debug, Serialize, Deserialize)]
pub struct GetMessagesResponse {
    pub messages: Vec<Message>,
    pub total: usize,
}

impl From<Vec<Message>> for GetMessagesResponse {
    fn from(messages: Vec<Message>) -> Self {
        let total = messages.len();
        Self { messages, total }
    }
}

/// Response type for search_messages tool
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchMessagesResponse {
    pub messages: Vec<Message>,
    pub matched: usize,
}

impl From<Vec<Message>> for SearchMessagesResponse {
    fn from(messages: Vec<Message>) -> Self {
        let matched = messages.len();
        Self { messages, matched }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::*;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Helper to create a mock FeishuClient for testing
    async fn create_test_client(base_url: &str) -> FeishuClient {
        use crate::auth::token_store::TokenStore;
        use std::sync::Arc;

        // Use unique temp db path per test to avoid conflicts
        let test_db_path = format!("/tmp/feishu_messages_test_{}.db", rand::random::<u64>());
        let _ = std::fs::remove_file(&test_db_path);

        let test_key: [u8; 32] = rand::random();
        let store = TokenStore::new(test_db_path.to_string(), &test_key)
            .expect("Failed to create token store");

        // Save a dummy token for testing
        store.save_token("test_app_id", "test_token", 9999999999)
            .expect("Failed to save token");

        FeishuClient::new(base_url.to_string(), Arc::new(store), "test_app_id".to_string())
    }

    #[tokio::test]
    async fn test_send_message_success() {
        let mock_server = MockServer::start().await;
        let client = create_test_client(&mock_server.uri()).await;

        // Mock the API response
        Mock::given(method("POST"))
            .and(path("/im/v1/messages"))
            .and(header("Authorization", "Bearer test_token"))
            .and(body_json(serde_json::json!({
                "receive_id": "test_chat_id",
                "content": "Hello World",
                "msg_type": "text"
            })))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "code": 0,
                    "msg": "success",
                    "data": {
                        "message": {
                            "message_id": "msg_123456",
                            "chat_id": "test_chat_id",
                            "content": "Hello World",
                            "msg_type": "text",
                            "sender_id": "user_001",
                            "created_time": 1704067200
                        }
                    }
                })))
            .mount(&mock_server)
            .await;

        let result = send_message(
            Arc::new(client),
            "test_chat_id".to_string(),
            "Hello World".to_string(),
            Some("text".to_string()),
        ).await;

        assert!(result.is_ok(), "send_message should succeed");
        let message = result.unwrap();
        assert_eq!(message.message_id, "msg_123456");
        assert_eq!(message.chat_id, "test_chat_id");
        assert_eq!(message.content, "Hello World");
        assert_eq!(message.msg_type, "text");
    }

    #[tokio::test]
    async fn test_send_message_default_msg_type() {
        let mock_server = MockServer::start().await;
        let client = create_test_client(&mock_server.uri()).await;

        Mock::given(method("POST"))
            .and(path("/im/v1/messages"))
            .and(header("Authorization", "Bearer test_token"))
            .and(body_json(serde_json::json!({
                "receive_id": "test_chat_id",
                "content": "Hello World",
                "msg_type": "text"
            })))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "code": 0,
                    "msg": "success",
                    "data": {
                        "message": {
                            "message_id": "msg_123456",
                            "chat_id": "test_chat_id",
                            "content": "Hello World",
                            "msg_type": "text",
                            "sender_id": "user_001",
                            "created_time": 1704067200
                        }
                    }
                })))
            .mount(&mock_server)
            .await;

        // Don't specify msg_type - should default to "text"
        let result = send_message(
            Arc::new(client),
            "test_chat_id".to_string(),
            "Hello World".to_string(),
            None,
        ).await;

        assert!(result.is_ok(), "send_message with default msg_type should succeed");
    }

    #[tokio::test]
    async fn test_get_messages_success() {
        let mock_server = MockServer::start().await;
        let client = create_test_client(&mock_server.uri()).await;

        Mock::given(method("GET"))
            .and(path("/im/v1/messages"))
            .and(query_param("container_id_type", "chat"))
            .and(query_param("container_id", "test_chat_id"))
            .and(header("Authorization", "Bearer test_token"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "code": 0,
                    "msg": "success",
                    "data": {
                        "items": [
                            {
                                "message_id": "msg_001",
                                "chat_id": "test_chat_id",
                                "content": "Message 1",
                                "msg_type": "text",
                                "sender_id": "user_001",
                                "created_time": 1704067200
                            },
                            {
                                "message_id": "msg_002",
                                "chat_id": "test_chat_id",
                                "content": "Message 2",
                                "msg_type": "text",
                                "sender_id": "user_002",
                                "created_time": 1704067300
                            }
                        ]
                    }
                })))
            .mount(&mock_server)
            .await;

        let result = get_messages(
            Arc::new(client),
            "test_chat_id".to_string(),
            None,
        ).await;

        assert!(result.is_ok(), "get_messages should succeed");
        let messages = result.unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].message_id, "msg_001");
        assert_eq!(messages[1].message_id, "msg_002");
    }

    #[tokio::test]
    async fn test_get_messages_with_limit() {
        let mock_server = MockServer::start().await;
        let client = create_test_client(&mock_server.uri()).await;

        Mock::given(method("GET"))
            .and(path("/im/v1/messages"))
            .and(query_param("container_id_type", "chat"))
            .and(query_param("container_id", "test_chat_id"))
            .and(query_param("page_size", "10"))
            .and(header("Authorization", "Bearer test_token"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "code": 0,
                    "msg": "success",
                    "data": {
                        "items": [
                            {
                                "message_id": "msg_001",
                                "chat_id": "test_chat_id",
                                "content": "Message 1",
                                "msg_type": "text",
                                "sender_id": "user_001",
                                "created_time": 1704067200
                            }
                        ]
                    }
                })))
            .mount(&mock_server)
            .await;

        let result = get_messages(
            Arc::new(client),
            "test_chat_id".to_string(),
            Some(10),
        ).await;

        assert!(result.is_ok(), "get_messages with limit should succeed");
        let messages = result.unwrap();
        assert_eq!(messages.len(), 1);
    }

    #[tokio::test]
    async fn test_search_messages_success() {
        let mock_server = MockServer::start().await;
        let client = create_test_client(&mock_server.uri()).await;

        Mock::given(method("POST"))
            .and(path("/search/v1/messages"))
            .and(header("Authorization", "Bearer test_token"))
            .and(body_json(serde_json::json!({
                "query": "hello",
                "chat_ids": ["test_chat_id"]
            })))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "code": 0,
                    "msg": "success",
                    "data": {
                        "messages": [
                            {
                                "message_id": "msg_001",
                                "chat_id": "test_chat_id",
                                "content": "hello world",
                                "msg_type": "text",
                                "sender_id": "user_001",
                                "created_time": 1704067200
                            }
                        ]
                    }
                })))
            .mount(&mock_server)
            .await;

        let result = search_messages(
            Arc::new(client),
            "hello".to_string(),
            Some("test_chat_id".to_string()),
        ).await;

        assert!(result.is_ok(), "search_messages should succeed");
        let messages = result.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].message_id, "msg_001");
    }

    #[tokio::test]
    async fn test_search_messages_without_chat_id() {
        let mock_server = MockServer::start().await;
        let client = create_test_client(&mock_server.uri()).await;

        Mock::given(method("POST"))
            .and(path("/search/v1/messages"))
            .and(header("Authorization", "Bearer test_token"))
            .and(body_json(serde_json::json!({
                "query": "hello",
                "chat_ids": null
            })))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "code": 0,
                    "msg": "success",
                    "data": {
                        "messages": [
                            {
                                "message_id": "msg_001",
                                "chat_id": "chat_001",
                                "content": "hello world",
                                "msg_type": "text",
                                "sender_id": "user_001",
                                "created_time": 1704067200
                            }
                        ]
                    }
                })))
            .mount(&mock_server)
            .await;

        let result = search_messages(
            Arc::new(client),
            "hello".to_string(),
            None,
        ).await;

        assert!(result.is_ok(), "search_messages without chat_id should succeed");
        let messages = result.unwrap();
        assert_eq!(messages.len(), 1);
    }

    #[tokio::test]
    async fn test_response_types_serialization() {
        let message = Message {
            message_id: "msg_123".to_string(),
            chat_id: "chat_456".to_string(),
            content: "test content".to_string(),
            msg_type: "text".to_string(),
            sender_id: Some("user_789".to_string()),
            created_time: Some(1704067200),
        };

        // Test SendMessageResponse
        let send_response: SendMessageResponse = message.clone().into();
        let json = serde_json::to_string(&send_response).unwrap();
        assert!(json.contains("msg_123"));
        assert!(json.contains("chat_456"));

        // Test GetMessagesResponse
        let get_response: GetMessagesResponse = vec![message.clone()].into();
        let json = serde_json::to_string(&get_response).unwrap();
        assert!(json.contains("\"total\":1"));
        assert!(json.contains("msg_123"));

        // Test SearchMessagesResponse
        let search_response: SearchMessagesResponse = vec![message].into();
        let json = serde_json::to_string(&search_response).unwrap();
        assert!(json.contains("\"matched\":1"));
        assert!(json.contains("msg_123"));
    }
}