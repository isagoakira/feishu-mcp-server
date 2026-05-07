/// Feishu API HTTP client wrapper

use crate::auth::token_store::TokenStore;
use crate::feishu::error::FeishuError;
use crate::feishu::types::*;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Health check response
#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    pub status: String,
}

/// Feishu API client
#[derive(Clone)]
pub struct FeishuClient {
    http_client: Arc<Client>,
    base_url: String,
    token_store: Arc<TokenStore>,
    app_id: String,
}

impl FeishuClient {
    /// Create a new FeishuClient
    pub fn new(base_url: String, token_store: Arc<TokenStore>, app_id: String) -> Self {
        let http_client = Arc::new(Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client"));
        Self {
            http_client,
            base_url,
            token_store,
            app_id,
        }
    }

    /// Get the HTTP client
    pub fn client(&self) -> Arc<Client> {
        self.http_client.clone()
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the app ID
    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    /// Get current tenant access token from token store
    fn get_token(&self) -> Result<String, FeishuError> {
        match self.token_store.load_token(&self.app_id) {
            Ok(Some((token, _))) => Ok(token),
            Ok(None) => Err(FeishuError::TokenExpired),
            Err(e) => Err(FeishuError::InternalError(format!("Failed to load token: {}", e))),
        }
    }

    /// Build full URL from path
    fn build_url(&self, path: &str) -> String {
        if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{}{}", self.base_url, path)
        }
    }

    /// Execute a GET request with automatic token injection and error handling
    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, FeishuError> {
        let url = self.build_url(path);
        let token = self.get_token()?;

        let response = self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(FeishuError::from_response(response).await);
        }

        let api_response: ApiResponse<T> = response.json().await?;

        if api_response.code != 0 {
            return Err(FeishuError::ApiError {
                code: api_response.code,
                msg: api_response.msg,
            });
        }

        api_response.data.ok_or_else(|| {
            FeishuError::ApiError {
                code: -1,
                msg: "No data in response".to_string(),
            }
        })
    }

    /// Execute a POST request with automatic token injection and error handling
    async fn post<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T, FeishuError> {
        let url = self.build_url(path);
        let token = self.get_token()?;

        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(FeishuError::from_response(response).await);
        }

        let api_response: ApiResponse<T> = response.json().await?;

        if api_response.code != 0 {
            return Err(FeishuError::ApiError {
                code: api_response.code,
                msg: api_response.msg,
            });
        }

        api_response.data.ok_or_else(|| {
            FeishuError::ApiError {
                code: -1,
                msg: "No data in response".to_string(),
            }
        })
    }

    /// Execute a PUT request with automatic token injection and error handling
    async fn put<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T, FeishuError> {
        let url = self.build_url(path);
        let token = self.get_token()?;

        let response = self.http_client
            .put(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(FeishuError::from_response(response).await);
        }

        let api_response: ApiResponse<T> = response.json().await?;

        if api_response.code != 0 {
            return Err(FeishuError::ApiError {
                code: api_response.code,
                msg: api_response.msg,
            });
        }

        api_response.data.ok_or_else(|| {
            FeishuError::ApiError {
                code: -1,
                msg: "No data in response".to_string(),
            }
        })
    }

    /// Execute a PATCH request with automatic token injection and error handling
    async fn patch<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T, FeishuError> {
        let url = self.build_url(path);
        let token = self.get_token()?;

        let response = self.http_client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(FeishuError::from_response(response).await);
        }

        let api_response: ApiResponse<T> = response.json().await?;

        if api_response.code != 0 {
            return Err(FeishuError::ApiError {
                code: api_response.code,
                msg: api_response.msg,
            });
        }

        api_response.data.ok_or_else(|| {
            FeishuError::ApiError {
                code: -1,
                msg: "No data in response".to_string(),
            }
        })
    }

    // ==================== Health Check ====================

    /// Health check endpoint
    pub async fn health_check(&self) -> Result<HealthResponse, FeishuError> {
        let url = format!("{}/health", self.base_url);
        let response = self.http_client.get(&url).send().await?;
        response.json().await.map_err(FeishuError::from)
    }

    // ==================== Documents API ====================

    /// Search documents
    pub async fn search_documents(
        &self,
        query: &str,
        limit: Option<i32>,
    ) -> Result<Vec<Document>, FeishuError> {
        #[derive(Serialize)]
        struct SearchRequest<'a> {
            search_key: &'a str,
            count: Option<i32>,
        }

        #[derive(Deserialize)]
        struct ApiData {
            items: Option<Vec<Document>>,
        }

        let body = SearchRequest {
            search_key: query,
            count: limit,
        };

        let data: ApiData = self.post("/doc/v2/search", &body).await?;
        Ok(data.items.unwrap_or_default())
    }

    /// Get a document by ID
    pub async fn get_document(&self, doc_id: &str) -> Result<Document, FeishuError> {
        let path = format!("/doc/v2/{}", doc_id);
        self.get(&path).await
    }

    /// Create a new document
    pub async fn create_document(
        &self,
        title: &str,
        content: Option<&str>,
        parent_token: Option<&str>,
    ) -> Result<Document, FeishuError> {
        #[derive(Serialize)]
        struct Request<'a> {
            title: &'a str,
            content: Option<&'a str>,
            parent_token: Option<&'a str>,
        }

        let request = Request {
            title,
            content,
            parent_token,
        };
        self.post("/doc/v2", &request).await
    }

    /// Update a document
    pub async fn update_document(
        &self,
        doc_id: &str,
        content: &str,
    ) -> Result<Document, FeishuError> {
        let path = format!("/doc/v2/{}", doc_id);
        #[derive(Serialize)]
        struct Request<'a> {
            content: &'a str,
        }
        let request = Request { content };
        self.put(&path, &request).await
    }

    /// List documents in a folder
    pub async fn list_documents(
        &self,
        folder_token: Option<&str>,
        page_size: Option<i32>,
    ) -> Result<Vec<Document>, FeishuError> {
        #[derive(Deserialize)]
        struct ApiData {
            files: Option<Vec<Document>>,
        }

        let mut path = "/drive/v1/files".to_string();
        if let Some(token) = folder_token {
            path = format!("{}/folders/{}", path, token);
        }
        if let Some(size) = page_size {
            path = format!("{}?page_size={}", path, size);
        }

        let data: ApiData = self.get(&path).await?;
        Ok(data.files.unwrap_or_default())
    }

    // ==================== Tasks API ====================

    /// List tasks
    pub async fn list_tasks(&self, goal_id: Option<&str>) -> Result<Vec<Task>, FeishuError> {
        #[derive(Deserialize)]
        struct ApiData {
            items: Option<Vec<Task>>,
        }

        let mut path = "/task/v2/tasks".to_string();
        if let Some(id) = goal_id {
            path = format!("{}?goal_id={}", path, id);
        }

        let data: ApiData = self.get(&path).await?;
        Ok(data.items.unwrap_or_default())
    }

    /// Get a task by ID
    pub async fn get_task(&self, task_id: &str) -> Result<Task, FeishuError> {
        let path = format!("/task/v2/tasks/{}", task_id);
        self.get(&path).await
    }

    /// Create a new task
    pub async fn create_task(
        &self,
        title: &str,
        description: Option<&str>,
        assignee_id: Option<&str>,
        due_date: Option<&str>,
        goal_id: Option<&str>,
    ) -> Result<Task, FeishuError> {
        #[derive(Serialize)]
        struct Request<'a> {
            title: &'a str,
            description: Option<&'a str>,
            assignee_id: Option<&'a str>,
            due_date: Option<&'a str>,
            goal_id: Option<&'a str>,
        }

        let request = Request {
            title,
            description,
            assignee_id,
            due_date,
            goal_id,
        };
        self.post("/task/v2/tasks", &request).await
    }

    /// Update task status
    pub async fn update_task_status(
        &self,
        task_id: &str,
        status: TaskStatus,
    ) -> Result<Task, FeishuError> {
        let path = format!("/task/v2/tasks/{}", task_id);
        #[derive(Serialize)]
        struct Request {
            status: TaskStatus,
        }
        let request = Request { status };
        self.patch(&path, &request).await
    }

    /// Complete a task
    pub async fn complete_task(&self, task_id: &str) -> Result<Task, FeishuError> {
        let path = format!("/task/v2/tasks/{}/complete", task_id);
        self.post(&path, &()).await
    }

    // ==================== Messages API ====================

    /// Send a message
    pub async fn send_message(
        &self,
        chat_id: &str,
        content: &str,
        msg_type: &str,
    ) -> Result<Message, FeishuError> {
        #[derive(Serialize)]
        struct Request<'a> {
            receive_id: &'a str,
            content: &'a str,
            msg_type: &'a str,
        }

        #[derive(Deserialize)]
        struct ApiData {
            message: Option<Message>,
        }

        let request = Request {
            receive_id: chat_id,
            content,
            msg_type,
        };
        let data: ApiData = self.post("/im/v1/messages?receive_id_type=chat_id", &request).await?;
        data.message.ok_or_else(|| {
            FeishuError::ApiError {
                code: -1,
                msg: "No message in response".to_string(),
            }
        })
    }

    /// Get messages from a chat
    pub async fn get_messages(
        &self,
        chat_id: &str,
        limit: Option<i32>,
    ) -> Result<Vec<Message>, FeishuError> {
        #[derive(Deserialize)]
        struct ApiData {
            items: Option<Vec<Message>>,
        }

        let mut path = format!("/im/v1/messages?container_id_type=chat&container_id={}", chat_id);
        if let Some(l) = limit {
            path = format!("{}&page_size={}", path, l);
        }

        let data: ApiData = self.get(&path).await?;
        Ok(data.items.unwrap_or_default())
    }

    /// Search messages
    pub async fn search_messages(
        &self,
        query: &str,
        chat_id: Option<&str>,
    ) -> Result<Vec<Message>, FeishuError> {
        #[derive(Serialize)]
        struct Request<'a> {
            query: &'a str,
            chat_ids: Option<Vec<&'a str>>,
        }

        #[derive(Deserialize)]
        struct ApiData {
            messages: Option<Vec<Message>>,
        }

        let request = Request {
            query,
            chat_ids: chat_id.map(|s| vec![s]),
        };

        let data: ApiData = self.post("/search/v1/messages", &request).await?;
        Ok(data.messages.unwrap_or_default())
    }
}