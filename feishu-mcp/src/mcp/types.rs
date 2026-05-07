/// Input parameter types for all 13 MCP tools.
///
/// Each type derives `JsonSchema` (for auto-generating inputSchema)
/// and `Deserialize` (for parsing MCP invocation arguments).

use rmcp::schemars;
use serde::Deserialize;

// ============================================================================
// Document Tools (5)
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchDocumentsInput {
    /// The search query string
    pub query: String,
    /// Maximum number of results to return (optional)
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetDocumentInput {
    /// The document ID to retrieve
    pub doc_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateDocumentInput {
    /// The title of the new document
    pub title: String,
    /// Optional initial content
    pub content: Option<String>,
    /// Optional parent folder token
    pub parent_token: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateDocumentInput {
    /// The document ID to update
    pub doc_id: String,
    /// The new content for the document
    pub content: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListDocumentsInput {
    /// The folder token to list documents from
    pub folder_token: String,
    /// Optional page size limit
    pub page_size: Option<u32>,
}

// ============================================================================
// Task Tools (5)
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListTasksInput {
    /// The goal ID to list tasks for
    pub goal_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetTaskInput {
    /// The task ID to retrieve
    pub task_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateTaskInput {
    /// The goal ID to create the task under
    pub goal_id: String,
    /// The title of the new task
    pub title: String,
    /// Optional assignee user ID
    pub assignee_id: Option<String>,
    /// Optional due date (ISO 8601)
    pub due_date: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateTaskStatusInput {
    /// The task ID to update
    pub task_id: String,
    /// New status: "todo", "in_progress", or "done"
    pub status: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CompleteTaskInput {
    /// The task ID to mark as complete
    pub task_id: String,
}

// ============================================================================
// Message Tools (3)
// ============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendMessageInput {
    /// The chat ID to send the message to
    pub chat_id: String,
    /// The message content
    pub content: String,
    /// Optional message type (default: "text")
    pub msg_type: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetMessagesInput {
    /// The chat ID to retrieve messages from
    pub chat_id: String,
    /// Optional limit on number of messages
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchMessagesInput {
    /// The search query string
    pub query: String,
    /// Optional chat ID to scope the search
    pub chat_id: Option<String>,
}
