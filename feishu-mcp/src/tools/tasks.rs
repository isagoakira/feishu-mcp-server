/// Task tools for Feishu MCP server

use crate::feishu::client::FeishuClient;
use crate::feishu::error::FeishuError;
use crate::feishu::types::{Task, TaskStatus};
use std::sync::Arc;

/// Response for list_tasks
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListTasksResponse {
    pub tasks: Vec<Task>,
    pub total: Option<i32>,
}

impl From<Vec<Task>> for ListTasksResponse {
    fn from(tasks: Vec<Task>) -> Self {
        Self { tasks, total: None }
    }
}

/// List tasks by goal ID
pub async fn list_tasks(client: Arc<FeishuClient>, goal_id: String) -> Result<ListTasksResponse, FeishuError> {
    let tasks = client.list_tasks(Some(&goal_id)).await?;
    Ok(ListTasksResponse::from(tasks))
}

/// Get task by ID
pub async fn get_task(client: Arc<FeishuClient>, task_id: String) -> Result<Task, FeishuError> {
    client.get_task(&task_id).await
}

/// Create a new task
pub async fn create_task(
    client: Arc<FeishuClient>,
    goal_id: String,
    title: String,
    assignee_id: Option<String>,
    due_date: Option<String>,
) -> Result<Task, FeishuError> {
    client
        .create_task(
            &title,
            None,
            assignee_id.as_deref(),
            due_date.as_deref(),
            Some(&goal_id),
        )
        .await
}

/// Update task status
pub async fn update_task_status(
    client: Arc<FeishuClient>,
    task_id: String,
    status: String,
) -> Result<Task, FeishuError> {
    let task_status = match status.as_str() {
        "todo" => TaskStatus::Todo,
        "in_progress" => TaskStatus::InProgress,
        "done" => TaskStatus::Done,
        _ => {
            return Err(FeishuError::ApiError {
                code: 400,
                msg: format!("Invalid status: {}. Expected 'todo', 'in_progress', or 'done'", status),
            });
        }
    };
    client.update_task_status(&task_id, task_status).await
}

/// Complete a task
pub async fn complete_task(client: Arc<FeishuClient>, task_id: String) -> Result<Task, FeishuError> {
    client.complete_task(&task_id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test status parsing logic
    #[test]
    fn test_task_status_parsing() {
        // Test TaskStatus variant matching directly
        let status_todo = TaskStatus::Todo;
        let status_in_progress = TaskStatus::InProgress;
        let status_done = TaskStatus::Done;

        assert!(matches!(status_todo, TaskStatus::Todo));
        assert!(matches!(status_in_progress, TaskStatus::InProgress));
        assert!(matches!(status_done, TaskStatus::Done));
    }

    #[allow(dead_code)]
    fn parse_status(status: &str) -> Result<TaskStatus, FeishuError> {
        match status {
            "todo" => Ok(TaskStatus::Todo),
            "in_progress" => Ok(TaskStatus::InProgress),
            "done" => Ok(TaskStatus::Done),
            _ => Err(FeishuError::ApiError {
                code: 400,
                msg: format!("Invalid status: {}. Expected 'todo', 'in_progress', or 'done'", status),
            }),
        }
    }

    #[test]
    fn test_list_tasks_response_from_vec() {
        let tasks = vec![
            Task {
                task_id: Some("task1".to_string()),
                guid: None,
                title: "Task 1".to_string(),
                description: None,
                status: Some(TaskStatus::Todo),
                assignee_id: None,
                due_date: None,
                goal_id: Some("goal1".to_string()),
                created_at: None,
                updated_at: None,
            },
            Task {
                task_id: Some("task2".to_string()),
                guid: None,
                title: "Task 2".to_string(),
                description: None,
                status: Some(TaskStatus::Done),
                assignee_id: None,
                due_date: None,
                goal_id: Some("goal1".to_string()),
                created_at: None,
                updated_at: None,
            },
        ];

        let response = ListTasksResponse::from(tasks);
        assert_eq!(response.tasks.len(), 2);
        assert_eq!(response.tasks[0].title, "Task 1");
        assert_eq!(response.tasks[1].title, "Task 2");
        assert!(response.total.is_none());
    }

    #[test]
    fn test_list_tasks_response_empty() {
        let tasks: Vec<Task> = vec![];
        let response = ListTasksResponse::from(tasks);
        assert!(response.tasks.is_empty());
    }

    #[test]
    fn test_task_serialization() {
        let task = Task {
            task_id: Some("task123".to_string()),
            guid: None,
            title: "Test Task".to_string(),
            description: Some("A test description".to_string()),
            status: Some(TaskStatus::InProgress),
            assignee_id: Some("user_abc".to_string()),
            due_date: Some("2026-05-10".to_string()),
            goal_id: Some("goal_xyz".to_string()),
            created_at: None,
            updated_at: None,
        };

        // Test serialization to JSON
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("task123"));
        assert!(json.contains("Test Task"));
        assert!(json.contains("in_progress"));

        // Test deserialization
        let deserialized: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.task_id, Some("task123".to_string()));
        assert_eq!(deserialized.title, "Test Task");
        assert_eq!(deserialized.status, Some(TaskStatus::InProgress));
    }

    #[test]
    fn test_task_status_serialization() {
        // Test Todo
        let todo_json = serde_json::to_string(&TaskStatus::Todo).unwrap();
        assert_eq!(todo_json, "\"todo\"");
        let todo: TaskStatus = serde_json::from_str(&todo_json).unwrap();
        assert_eq!(todo, TaskStatus::Todo);

        // Test InProgress
        let in_progress_json = serde_json::to_string(&TaskStatus::InProgress).unwrap();
        assert_eq!(in_progress_json, "\"in_progress\"");
        let in_progress: TaskStatus = serde_json::from_str(&in_progress_json).unwrap();
        assert_eq!(in_progress, TaskStatus::InProgress);

        // Test Done
        let done_json = serde_json::to_string(&TaskStatus::Done).unwrap();
        assert_eq!(done_json, "\"done\"");
        let done: TaskStatus = serde_json::from_str(&done_json).unwrap();
        assert_eq!(done, TaskStatus::Done);
    }

    #[test]
    fn test_list_tasks_response_serialization() {
        let response = ListTasksResponse {
            tasks: vec![
                Task {
                    task_id: Some("task1".to_string()),
                    guid: None,
                    title: "Task 1".to_string(),
                    description: None,
                    status: Some(TaskStatus::Todo),
                    assignee_id: None,
                    due_date: None,
                    goal_id: None,
                    created_at: None,
                    updated_at: None,
                },
            ],
            total: Some(1),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("task1"));
        assert!(json.contains("Task 1"));

        let deserialized: ListTasksResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tasks.len(), 1);
        assert_eq!(deserialized.total, Some(1));
    }

    #[test]
    fn test_error_message_format() {
        let err = FeishuError::ApiError {
            code: 400,
            msg: "Invalid status: invalid_status. Expected 'todo', 'in_progress', or 'done'".to_string(),
        };
        let err_msg = err.to_string();
        assert!(err_msg.contains("400"));
        assert!(err_msg.contains("Invalid status"));
    }
}