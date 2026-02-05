use std::{collections::BTreeMap, sync::Arc};

use crate::jobs::JobHandle;

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
    pub steps: Vec<String>,
}

impl JobInfo {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        steps: Vec<String>,
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
