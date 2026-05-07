pub mod documents;
pub mod tasks;
pub mod messages;

// Task tools re-exports
pub use tasks::{
    list_tasks,
    get_task,
    create_task,
    update_task_status,
    complete_task,
    ListTasksResponse,
};

// Message tools re-exports
pub use messages::{
    send_message,
    get_messages,
    search_messages,
    SendMessageResponse,
    GetMessagesResponse,
    SearchMessagesResponse,
};