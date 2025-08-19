use num_enum::{FromPrimitive, IntoPrimitive};
use serde::Serialize;
use std::collections::HashMap;
use time::OffsetDateTime;

use tokio::sync::watch::Receiver;

#[derive(Debug, Clone, Default, PartialEq, Eq, FromPrimitive, IntoPrimitive, Serialize)]
#[repr(u8)]
pub enum TaskStatus {
    #[default]
    Idle,
    Running,
    Stopped,
}

impl TaskStatus {
    pub fn is_idle(value: impl Into<TaskStatus>) -> bool {
        value.into() == TaskStatus::Idle
    }

    pub fn is_running(value: impl Into<TaskStatus>) -> bool {
        value.into() == TaskStatus::Running
    }

    pub fn is_stopped(value: impl Into<TaskStatus>) -> bool {
        value.into() == TaskStatus::Stopped
    }
}

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum TaskError {
    #[error("Task is already running")]
    Running,
    #[error("Task is already stopped or is idle")]
    Stop,
    #[error("Task failed: {0}")]
    Failed(String),
}

/// Information about the task
///
/// This is used to display the task in the UI
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: u8,
}

impl TaskInfo {
    pub fn new(id: &str, name: &str, description: &str, steps: u8) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            steps,
        }
    }
}

impl Default for TaskInfo {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            steps: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum TaskEventType {
    #[default]
    Initial,
    Info,
    Error,
    Warning,
    Progress,
    Complete,
    Start,
    Stop,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskEvent {
    pub kind: TaskEventType,
    pub message: String,
    pub current: Option<u64>,
    pub total: Option<u64>,
    pub step: Option<u8>,
    pub timestamp: OffsetDateTime,
}

impl Default for TaskEvent {
    fn default() -> Self {
        Self {
            kind: TaskEventType::default(),
            message: String::new(),
            current: None,
            total: None,
            step: None,
            timestamp: OffsetDateTime::now_utc(),
        }
    }
}

impl TaskEvent {
    pub fn new(
        kind: TaskEventType,
        message: String,
        current: Option<u64>,
        total: Option<u64>,
        step: Option<u8>,
    ) -> Self {
        Self {
            kind,
            message,
            step,
            current,
            total,
            ..Default::default()
        }
    }

    pub fn info(message: &str) -> Self {
        Self {
            kind: TaskEventType::Info,
            message: message.to_string(),
            ..Default::default()
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            kind: TaskEventType::Error,
            message: message.to_string(),
            ..Default::default()
        }
    }

    pub fn warning(message: &str) -> Self {
        Self {
            kind: TaskEventType::Warning,
            message: message.to_string(),
            ..Default::default()
        }
    }

    pub fn complete(message: &str) -> Self {
        Self {
            kind: TaskEventType::Complete,
            message: message.to_string(),
            ..Default::default()
        }
    }

    pub fn progress(
        message: &str,
        current: u64,
        total: u64,
        step: Option<u8>,
    ) -> Self {
        Self {
            kind: TaskEventType::Progress,
            message: message.to_string(),
            current: Some(current),
            total: Some(total),
            step,
            ..Default::default()
        }
    }

    pub fn start(message: &str) -> Self {
        Self {
            kind: TaskEventType::Start,
            message: message.to_string(),
            ..Default::default()
        }
    }

    pub fn stop(message: &str) -> Self {
        Self {
            kind: TaskEventType::Stop,
            message: message.to_string(),
            ..Default::default()
        }
    }

    pub fn initial(name: &str) -> Self {
        Self {
            kind: TaskEventType::Initial,
            message: format!("Initialized \"{name}\" event channel"),
            ..Default::default()
        }
    }
}

pub trait Task: Send + Sync {
    /// Get information about the task
    fn info(&self) -> &TaskInfo;

    /// Start the task
    fn start(&mut self) -> Result<(), TaskError>;

    /// Stop the task
    fn stop(&mut self) -> Result<(), TaskError>;

    /// Get the status of the task
    fn status(&self) -> TaskStatus;

    /// ID of the task
    ///
    /// The ID is used to identify the task in the registry
    /// Usually, it gets the ID from the `info` function
    fn id(&self) -> String {
        self.info().id.clone()
    }

    /// Name of the task
    ///
    /// Usually, it gets the name from the `info` function
    fn name(&self) -> String {
        self.info().name.clone()
    }

    /// Get the event channel of the task
    fn channel(&self) -> Option<Receiver<TaskEvent>> {
        None
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RegistryError {
    #[error("Task not found")]
    NotFound,

    #[error("Task already exists")]
    AlreadyExists,

    #[error("Could not set task state: {0}")]
    StateError(#[from] TaskError),
}

#[derive(Default)]
pub struct Registry {
    tasks: HashMap<String, Box<dyn Task + 'static>>,
}

impl Registry {
    /// Registers a task that implements the `Default` trait
    pub fn register_default<T: Task + Default + 'static>(&mut self) -> Result<(), RegistryError> {
        self.register(|| Box::new(T::default()))
    }

    /// Registers a task by providing a function that returns a boxed task
    pub fn register<F>(&mut self, task_fn: F) -> Result<(), RegistryError>
    where
        F: Fn() -> Box<dyn Task> + 'static,
    {
        if self
            .tasks
            .values()
            .any(|task| task.info().name == task_fn().info().name)
        {
            return Err(RegistryError::AlreadyExists);
        }

        let id = task_fn().id();
        self.tasks.insert(id, task_fn());

        Ok(())
    }

    /// Gets the list of tasks registered
    pub fn list(&self) -> Vec<String> {
        self.tasks.keys().cloned().collect()
    }

    /// Gets the list of information for all tasks
    pub fn list_tasks(&self) -> Vec<TaskInfo> {
        self.tasks
            .values()
            .map(|task| task.info().clone())
            .collect()
    }

    /// Starts a task
    ///
    /// If the task is not found, an error is returned
    pub fn start_task(&mut self, name: &str) -> Result<(), RegistryError> {
        match self.tasks.get_mut(name) {
            Some(task) => task.start()?,
            None => return Err(RegistryError::NotFound),
        }

        Ok(())
    }

    /// Gets the event channel for a task
    pub fn get_event_channel(&self, name: &str) -> Option<Receiver<TaskEvent>> {
        self.tasks.get(name).and_then(|task| task.channel())
    }

    /// Stops a task
    ///
    /// If the task is not found, an error is returned
    pub fn stop_task(&mut self, name: &str) -> Result<(), RegistryError> {
        match self.tasks.get_mut(name) {
            Some(task) => task.stop()?,
            None => return Err(RegistryError::NotFound),
        }

        Ok(())
    }
}
