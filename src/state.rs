use std::sync::{Arc, Mutex};

use axum::{extract::FromRef, response::sse::Event};
use tokio::sync::broadcast::Sender;

use super::{
    events::TaskEvent,
    tasks::{self, Registry, RegistryError},
};

use crate::config::Settings;

pub type Database = sqlx::Pool<sqlx::Sqlite>;
pub type TaskRegistry = Arc<Mutex<Registry>>;

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub tasks: TaskRegistry,
    pub event_sender: Sender<Event>,
    pub db: Database,
}

impl AppState {
    pub fn new(db: Database, settings: Settings) -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(1024);
        let tasks = setup_tasks(db.clone(), tx.clone());
        Self {
            db,
            tasks,
            settings,
            event_sender: tx,
        }
    }
}

/// Set up the tasks that can run in the background
fn setup_tasks(pool: sqlx::Pool<sqlx::Sqlite>, tx: Sender<Event>) -> Arc<Mutex<Registry>> {
    let mut registry = Registry::default();

    let scan_songs_pool = pool.clone();

    if let Err(RegistryError::AlreadyExists) =
        registry.register(move || Box::new(tasks::ScanSongs::new(scan_songs_pool.clone())))
    {
        tracing::warn!("Task already registered");
    }

    for (name, channel) in registry.event_channels() {
        let sender = tx.clone();
        tokio::spawn(async move {
            let mut channel = channel;

            loop {
                let event = channel.borrow_and_update().clone();
                let _ = sender.send(TaskEvent::new(&name, event).into());

                if channel.changed().await.is_err() {
                    break;
                }
            }
        });
    }

    Arc::new(Mutex::new(registry))
}

impl FromRef<AppState> for Sender<Event> {
    fn from_ref(state: &AppState) -> Self {
        state.event_sender.clone()
    }
}

impl FromRef<AppState> for TaskRegistry {
    fn from_ref(state: &AppState) -> Self {
        state.tasks.clone()
    }
}

impl FromRef<AppState> for sqlx::Pool<sqlx::Sqlite> {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}

impl FromRef<AppState> for Settings {
    fn from_ref(state: &AppState) -> Self {
        state.settings.clone()
    }
}
