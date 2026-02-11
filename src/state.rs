use std::sync::Arc;

use axum::{extract::FromRef, response::sse::Event};
use tokio::sync::broadcast::Sender;

use crate::state::job::{Job, JobRegistry};

use super::{config::Settings, jobs::ScanSongs};

mod fs;
pub mod job;

pub use fs::*;

pub type JobManager = Arc<job::manager::JobManager>;
pub type Pool = sqlx::SqlitePool;
pub type FileOperationManager = Arc<OperationManager>;

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub job_manager: JobManager,
    pub event_sender: Sender<Event>,
    pub file_operation_manager: FileOperationManager,
    pub pool: Pool,
}

impl AppState {
    pub fn new(db: Pool, settings: Settings) -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(1024);

        let file_operation_manager = OperationManager::new();
        let mut rx = file_operation_manager.events();

        let tx_clone = tx.clone();
        tokio::spawn(async move {
            while let Ok(item) = rx.recv().await {
                let _ = tx_clone.send(Event::from(super::events::FileOperationManagerEvent::from(
                    item,
                )));
            }
        });

        let job_manager = job::manager::JobManager::new(setup_jobs(&db));
        let mut rx = job_manager.events();
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            while let Ok(item) = rx.recv().await {
                let _ = tx_clone.send(Event::from(super::events::JobManagerEvent::from(item)));
            }
        });

        Self {
            pool: db,
            settings,
            event_sender: tx,
            job_manager: Arc::new(job_manager),
            file_operation_manager: Arc::new(file_operation_manager),
        }
    }
}

fn setup_jobs(pool: &sqlx::Pool<sqlx::Sqlite>) -> JobRegistry {
    let mut registry = JobRegistry::default();

    registry
        .register_job(
            "scan-songs",
            Job::new(ScanSongs::job_info(), ScanSongs::new(pool.clone())),
        )
        .expect("Failed to register job");

    registry
}

impl FromRef<AppState> for JobManager {
    fn from_ref(state: &AppState) -> Self {
        state.job_manager.clone()
    }
}

impl FromRef<AppState> for Sender<Event> {
    fn from_ref(state: &AppState) -> Self {
        state.event_sender.clone()
    }
}

impl FromRef<AppState> for FileOperationManager {
    fn from_ref(state: &AppState) -> Self {
        state.file_operation_manager.clone()
    }
}

impl FromRef<AppState> for Pool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

impl FromRef<AppState> for Settings {
    fn from_ref(state: &AppState) -> Self {
        state.settings.clone()
    }
}
