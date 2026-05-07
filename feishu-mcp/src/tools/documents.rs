/// Document tools for Feishu MCP server

use crate::feishu::client::FeishuClient;
use crate::feishu::types::Document;
use crate::feishu::error::FeishuError;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// Search documents response with items wrapper
#[derive(Debug, Deserialize, Serialize)]
pub struct SearchDocumentsResponse {
    pub items: Option<Vec<Document>>,
}

/// List documents response
#[derive(Debug, Deserialize, Serialize)]
pub struct ListDocumentsResponse {
    pub files: Option<Vec<Document>>,
}

/// Search documents by query string
pub async fn search_documents(
    client: Arc<FeishuClient>,
    query: String,
    limit: Option<u32>,
) -> Result<SearchDocumentsResponse, FeishuError> {
    let documents = client
        .search_documents(&query, limit.map(|l| l as i32))
        .await?;

    Ok(SearchDocumentsResponse {
        items: Some(documents),
    })
}

/// Get document by ID
pub async fn get_document(
    client: Arc<FeishuClient>,
    doc_id: String,
) -> Result<Document, FeishuError> {
    client.get_document(&doc_id).await
}

/// Create a new document
pub async fn create_document(
    client: Arc<FeishuClient>,
    title: String,
    content: Option<String>,
    parent_token: Option<String>,
) -> Result<Document, FeishuError> {
    client
        .create_document(
            &title,
            content.as_deref(),
            parent_token.as_deref(),
        )
        .await
}

/// Update document content
pub async fn update_document(
    client: Arc<FeishuClient>,
    doc_id: String,
    content: String,
) -> Result<Document, FeishuError> {
    client.update_document(&doc_id, &content).await
}

/// List documents in a folder
pub async fn list_documents(
    client: Arc<FeishuClient>,
    folder_token: String,
    page_size: Option<u32>,
) -> Result<ListDocumentsResponse, FeishuError> {
    let documents = client
        .list_documents(Some(&folder_token), page_size.map(|p| p as i32))
        .await?;

    Ok(ListDocumentsResponse {
        files: Some(documents),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feishu::client::FeishuClient;
    use crate::auth::token_store::TokenStore;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{method, path, header, body_json};

    /// Helper to create a test client with mock server
    async fn setup_test_client(mock_server: &MockServer) -> Arc<FeishuClient> {
        // Use unique temp db path per test to avoid conflicts
        let test_db_path = format!("/tmp/feishu_documents_test_{}.db", rand::random::<u64>());
        let _ = std::fs::remove_file(&test_db_path);

        let test_key: [u8; 32] = rand::random();
        let token_store = TokenStore::new(test_db_path, &test_key)
            .expect("Failed to create token store");
        token_store.save_token("test-app-id", "test-token", 9999999999)
            .expect("Failed to save token");

        let client = FeishuClient::new(
            mock_server.uri(),
            Arc::new(token_store),
            "test-app-id".to_string(),
        );
        Arc::new(client)
    }

    #[tokio::test]
    async fn test_search_documents_success() {
        let mock_server = MockServer::start().await;
        let client = setup_test_client(&mock_server).await;

        let response_body = serde_json::json!({
            "code": 0,
            "msg": "success",
            "data": {
                "items": [
                    {
                        "doc_id": "doc123",
                        "title": "Test Document",
                        "content": "Test content",
                        "created_time": "2024-01-01T00:00:00Z",
                        "updated_time": "2024-01-02T00:00:00Z",
                        "owner_id": "user456",
                        "parent_token": null
                    }
                ]
            }
        });

        Mock::given(method("POST"))
            .and(path("/doc/v2/search"))
            .and(header("Authorization", "Bearer test-token"))
            .and(body_json(serde_json::json!({
                "search_key": "test query",
                "count": 10
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
            .mount(&mock_server)
            .await;

        let result = search_documents(client, "test query".to_string(), Some(10)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.items.is_some());
        assert_eq!(response.items.as_ref().unwrap().len(), 1);
        assert_eq!(response.items.as_ref().unwrap()[0].doc_id, "doc123");
    }

    #[tokio::test]
    async fn test_search_documents_empty() {
        let mock_server = MockServer::start().await;
        let client = setup_test_client(&mock_server).await;

        let response_body = serde_json::json!({
            "code": 0,
            "msg": "success",
            "data": {
                "items": []
            }
        });

        Mock::given(method("POST"))
            .and(path("/doc/v2/search"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
            .mount(&mock_server)
            .await;

        let result = search_documents(client, "nonexistent".to_string(), None).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.items.is_some());
        assert!(response.items.as_ref().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_document_success() {
        let mock_server = MockServer::start().await;
        let client = setup_test_client(&mock_server).await;

        let response_body = serde_json::json!({
            "code": 0,
            "msg": "success",
            "data": {
                "doc_id": "doc789",
                "title": "My Document",
                "content": "Document content here",
                "created_time": "2024-01-01T00:00:00Z",
                "updated_time": "2024-01-02T00:00:00Z",
                "owner_id": "user123",
                "parent_token": "folder_token_abc"
            }
        });

        Mock::given(method("GET"))
            .and(path("/doc/v2/doc789"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
            .mount(&mock_server)
            .await;

        let result = get_document(client, "doc789".to_string()).await;

        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.doc_id, "doc789");
        assert_eq!(doc.title, "My Document");
        assert_eq!(doc.content, Some("Document content here".to_string()));
    }

    #[tokio::test]
    async fn test_get_document_not_found() {
        let mock_server = MockServer::start().await;
        let client = setup_test_client(&mock_server).await;

        let response_body = serde_json::json!({
            "code": 99991663,
            "msg": "token expired"
        });

        Mock::given(method("GET"))
            .and(path("/doc/v2/nonexistent"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(401).set_body_json(&response_body))
            .mount(&mock_server)
            .await;

        let result = get_document(client, "nonexistent".to_string()).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            FeishuError::TokenExpired => {}
            other => panic!("Expected TokenExpired, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_create_document_success() {
        let mock_server = MockServer::start().await;
        let client = setup_test_client(&mock_server).await;

        let response_body = serde_json::json!({
            "code": 0,
            "msg": "success",
            "data": {
                "doc_id": "new_doc_id",
                "title": "New Document",
                "content": "Initial content",
                "created_time": "2024-01-01T00:00:00Z",
                "updated_time": "2024-01-01T00:00:00Z",
                "owner_id": "user123",
                "parent_token": "parent_folder"
            }
        });

        Mock::given(method("POST"))
            .and(path("/doc/v2"))
            .and(header("Authorization", "Bearer test-token"))
            .and(body_json(serde_json::json!({
                "title": "New Document",
                "content": "Initial content",
                "parent_token": "parent_folder"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
            .mount(&mock_server)
            .await;

        let result = create_document(
            client,
            "New Document".to_string(),
            Some("Initial content".to_string()),
            Some("parent_folder".to_string()),
        ).await;

        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.doc_id, "new_doc_id");
        assert_eq!(doc.title, "New Document");
    }

    #[tokio::test]
    async fn test_create_document_minimal() {
        let mock_server = MockServer::start().await;
        let client = setup_test_client(&mock_server).await;

        let response_body = serde_json::json!({
            "code": 0,
            "msg": "success",
            "data": {
                "doc_id": "minimal_doc",
                "title": "Minimal",
                "content": null,
                "created_time": "2024-01-01T00:00:00Z",
                "updated_time": "2024-01-01T00:00:00Z",
                "owner_id": "user123",
                "parent_token": null
            }
        });

        Mock::given(method("POST"))
            .and(path("/doc/v2"))
            .and(header("Authorization", "Bearer test-token"))
            .and(body_json(serde_json::json!({
                "title": "Minimal",
                "content": null,
                "parent_token": null
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
            .mount(&mock_server)
            .await;

        let result = create_document(
            client,
            "Minimal".to_string(),
            None,
            None,
        ).await;

        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.doc_id, "minimal_doc");
    }

    #[tokio::test]
    async fn test_update_document_success() {
        let mock_server = MockServer::start().await;
        let client = setup_test_client(&mock_server).await;

        let response_body = serde_json::json!({
            "code": 0,
            "msg": "success",
            "data": {
                "doc_id": "doc_to_update",
                "title": "Updated Document",
                "content": "New content here",
                "created_time": "2024-01-01T00:00:00Z",
                "updated_time": "2024-01-03T00:00:00Z",
                "owner_id": "user123",
                "parent_token": null
            }
        });

        Mock::given(method("PUT"))
            .and(path("/doc/v2/doc_to_update"))
            .and(header("Authorization", "Bearer test-token"))
            .and(body_json(serde_json::json!({
                "content": "New content here"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
            .mount(&mock_server)
            .await;

        let result = update_document(
            client,
            "doc_to_update".to_string(),
            "New content here".to_string(),
        ).await;

        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.doc_id, "doc_to_update");
        assert_eq!(doc.content, Some("New content here".to_string()));
    }

    #[tokio::test]
    async fn test_list_documents_success() {
        let mock_server = MockServer::start().await;
        let client = setup_test_client(&mock_server).await;

        let response_body = serde_json::json!({
            "code": 0,
            "msg": "success",
            "data": {
                "files": [
                    {
                        "doc_id": "file1",
                        "title": "File One",
                        "content": null,
                        "created_time": "2024-01-01T00:00:00Z",
                        "updated_time": "2024-01-01T00:00:00Z",
                        "owner_id": "user123",
                        "parent_token": "folder123"
                    },
                    {
                        "doc_id": "file2",
                        "title": "File Two",
                        "content": null,
                        "created_time": "2024-01-02T00:00:00Z",
                        "updated_time": "2024-01-02T00:00:00Z",
                        "owner_id": "user456",
                        "parent_token": "folder123"
                    }
                ]
            }
        });

        Mock::given(method("GET"))
            .and(path("/drive/v1/files/folders/folder123"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
            .mount(&mock_server)
            .await;

        let result = list_documents(client, "folder123".to_string(), None).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.files.is_some());
        assert_eq!(response.files.as_ref().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_list_documents_with_page_size() {
        let mock_server = MockServer::start().await;
        let client = setup_test_client(&mock_server).await;

        let response_body = serde_json::json!({
            "code": 0,
            "msg": "success",
            "data": {
                "files": [
                    {
                        "doc_id": "file1",
                        "title": "First File",
                        "content": null,
                        "created_time": "2024-01-01T00:00:00Z",
                        "updated_time": "2024-01-01T00:00:00Z",
                        "owner_id": "user123",
                        "parent_token": "folder123"
                    }
                ]
            }
        });

        Mock::given(method("GET"))
            .and(path("/drive/v1/files/folders/folder123"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
            .mount(&mock_server)
            .await;

        let result = list_documents(client, "folder123".to_string(), Some(5)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.files.is_some());
        assert_eq!(response.files.as_ref().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_list_documents_empty_folder() {
        let mock_server = MockServer::start().await;
        let client = setup_test_client(&mock_server).await;

        let response_body = serde_json::json!({
            "code": 0,
            "msg": "success",
            "data": {
                "files": []
            }
        });

        Mock::given(method("GET"))
            .and(path("/drive/v1/files/folders/empty_folder"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
            .mount(&mock_server)
            .await;

        let result = list_documents(client, "empty_folder".to_string(), None).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.files.is_some());
        assert!(response.files.as_ref().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_search_documents_api_error() {
        let mock_server = MockServer::start().await;
        let client = setup_test_client(&mock_server).await;

        let response_body = serde_json::json!({
            "code": 1001001,
            "msg": "permission denied"
        });

        Mock::given(method("POST"))
            .and(path("/doc/v2/search"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
            .mount(&mock_server)
            .await;

        let result = search_documents(client, "secret".to_string(), None).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            FeishuError::ApiError { code, msg } => {
                assert_eq!(code, 1001001);
                assert_eq!(msg, "permission denied");
            }
            other => panic!("Expected ApiError, got {:?}", other),
        }
    }
}