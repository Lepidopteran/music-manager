use std::{collections::BTreeMap, sync::Arc};

use tokio::sync::mpsc;
use ts_rs::TS;

use crate::jobs::{JobEvent, JobHandle};

pub mod manager;
pub mod state;

use state::JobStateId;

type Result<T, E = JobRegistryError> = std::result::Result<T, E>;

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

#[derive(Debug)]
pub struct JobHandler {
    state_id: JobStateId,
    job_id: JobId,
    events: mpsc::Receiver<JobEvent>,
}

impl JobHandler {
    pub fn new(id: JobStateId, job_id: JobId, events: mpsc::Receiver<JobEvent>) -> Self {
        Self {
            state_id: id,
            job_id,
            events,
        }
    }

    pub fn id(&self) -> JobStateId {
        self.state_id
    }

    pub fn job_id(&self) -> &JobId {
        &self.job_id
    }

    pub fn events(&mut self) -> &mut mpsc::Receiver<JobEvent> {
        &mut self.events
    }
}
