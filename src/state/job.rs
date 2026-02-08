use std::{collections::BTreeMap, sync::Arc};

use serde::Serialize;
use time::OffsetDateTime;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use ts_rs::TS;

use crate::jobs::{JobEvent, JobHandle};

pub mod manager;

type Result<T, E = JobRegistryError> = std::result::Result<T, E>;

pub type JobStateId = uuid::Uuid;
pub type JobId = String;

#[derive(Debug, thiserror::Error)]
pub enum JobRegistryError {
    #[error("Job already exists")]
    AlreadyExists,
    #[error("Job not found")]
    NotFound,
}

#[derive(Debug, Clone)]
pub struct JobInfo {
    pub name: String,
    pub description: String,
    pub steps: BTreeMap<u8, String>,
}

impl JobInfo {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        steps: BTreeMap<u8, String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            steps,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Job {
    /// Job info for the registry used for UI.
    info: JobInfo,
    /// Job handle for the actual job.
    handle: Arc<dyn JobHandle>,
}

impl Job {
    pub fn new<H: JobHandle>(info: JobInfo, handle: H) -> Self {
        Self {
            info,
            handle: Arc::new(handle),
        }
    }

    pub fn info(&self) -> &JobInfo {
        &self.info
    }

    pub fn handle(&self) -> Arc<dyn JobHandle> {
        self.handle.clone()
    }
}

#[derive(Debug, Default)]
pub struct JobRegistry {
    jobs: BTreeMap<JobId, Job>,
}

impl JobRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn jobs(&self) -> &BTreeMap<JobId, Job> {
        &self.jobs
    }

    pub fn register_job(&mut self, id: impl Into<JobId>, job: Job) -> Result<()> {
        let id = id.into();

        if self.jobs.contains_key(&id) {
            return Err(JobRegistryError::AlreadyExists);
        }

        self.jobs.insert(id, job);
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "bindings.ts")]
#[serde(rename_all = "camelCase")]
pub struct JobState {
    pub job_id: JobId,
    pub status: JobStatus,
    pub current_step: u8,
    pub values: BTreeMap<u8, String>,
    #[serde(skip)]
    pub token: CancellationToken,
}

impl JobState {
    pub fn new(job_id: JobId) -> Self {
        Self {
            job_id,
            status: JobStatus::Pending,
            current_step: 1,
            values: BTreeMap::new(),
            token: CancellationToken::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, TS)]
#[ts(export, export_to = "bindings.ts")]
#[serde(rename_all = "camelCase")]
pub enum JobStatus {
    Pending,
    InProgress,
}

#[derive(Debug, Clone, Serialize, Default, TS)]
#[ts(export, export_to = "bindings.ts")]
#[serde(rename_all = "camelCase")]
pub struct JobExecutionReport {
    #[ts(type = "Date")]
    #[serde(with = "time::serde::rfc3339::option")]
    pub started_at: Option<OffsetDateTime>,
    #[ts(type = "Date")]
    #[serde(with = "time::serde::rfc3339::option")]
    pub completed_at: Option<OffsetDateTime>,
    #[ts(type = "Date")]
    #[serde(with = "time::serde::rfc3339::option")]
    pub cancelled_at: Option<OffsetDateTime>,
    pub completed_successfully: bool,
}
