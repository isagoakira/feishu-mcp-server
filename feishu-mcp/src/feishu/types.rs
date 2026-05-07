/// Feishu API response types

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Document type representing a Feishu document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub doc_id: String,
    pub title: String,
    pub content: Option<String>,
    pub created_time: Option<DateTime<Utc>>,
    pub updated_time: Option<DateTime<Utc>>,
    pub owner_id: Option<String>,
    pub parent_token: Option<String>,
}

/// Task status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Todo
    }
}

/// Task type representing a Feishu task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub task_id: Option<String>,
    pub guid: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub assignee_id: Option<String>,
    pub due_date: Option<String>,
    pub goal_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Message type representing a Feishu message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub message_id: String,
    pub chat_id: String,
    pub content: String,
    pub msg_type: String,
    pub sender_id: Option<String>,
    pub created_time: Option<i64>,
}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub msg: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    /// Check if the API response indicates success
    pub fn is_success(&self) -> bool {
        self.code == 0
    }
}

/// Create document request body
#[derive(Debug, Serialize)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub content: Option<String>,
    pub parent_token: Option<String>,
}

/// Update document request body
#[derive(Debug, Serialize)]
pub struct UpdateDocumentRequest {
    pub content: String,
}

/// Create task request body
#[derive(Debug, Serialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub assignee_id: Option<String>,
    pub due_date: Option<String>,
    pub goal_id: Option<String>,
}

/// Update task status request body
#[derive(Debug, Serialize)]
pub struct UpdateTaskRequest {
    pub status: TaskStatus,
}

/// Send message request body
#[derive(Debug, Serialize)]
pub struct SendMessageRequest {
    pub content: String,
    pub msg_type: String,
}