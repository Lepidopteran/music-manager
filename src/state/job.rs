use std::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
};

use serde::Serialize;
use time::OffsetDateTime;
use tokio::{
    sync::{Mutex, Notify, broadcast, mpsc},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use ts_rs::TS;

use crate::jobs::{JobEvent, JobHandle};

type RegistryResult<T, E = JobRegistryError> = std::result::Result<T, E>;
type ManagerResult<T, E = JobManagerError> = std::result::Result<T, E>;

pub type JobStateId = i64;

pub type JobStates = BTreeMap<JobStateId, JobState>;
pub type JobReports = BTreeMap<JobId, JobExecutionReport>;

pub type JobId = String;

#[derive(Debug)]
struct QueueItem {
    entry: (JobId, Job),
    state_id: JobStateId,
    cancel_token: CancellationToken,
    job_events: mpsc::Sender<JobEvent>,
    unique: bool,
}

#[derive(Debug)]
pub struct Queue {
    list: Mutex<VecDeque<QueueItem>>,
    notify: Notify,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            list: Mutex::new(VecDeque::new()),
            notify: Notify::new(),
        }
    }

    /// Returns the rank of an item in the queue
    async fn item_rank(&self, state_id: JobStateId) -> Option<usize> {
        tracing::debug!("Locking Queue");
        let list = self.list.lock().await;
        list.iter().position(|item| item.state_id == state_id)
    }

    /// Returns true if any item in the queue matches the predicate
    async fn any<F: Fn(&QueueItem) -> bool>(&self, predicate: F) -> bool {
        tracing::debug!("Locking Queue");
        let list = self.list.lock().await;
        list.iter().any(predicate)
    }

    /// Adds an item to the queue
    async fn add_item(&self, item: QueueItem, high_priority: bool) {
        tracing::debug!("Locking Queue");
        let mut list = self.list.lock().await;

        if high_priority {
            list.push_front(item);
            drop(list);
        } else {
            list.push_back(item);
            drop(list);
        }

        self.notify.notify_waiters();
    }

    /// Removes an item from the queue
    async fn remove_item(&self, state_id: JobStateId, notify: bool) {
        let mut list = self.list.lock().await;
        list.retain(|item| item.state_id != state_id);
        drop(list);

        if notify {
            self.notify.notify_waiters();
        }
    }
}

#[derive(Debug, Clone)]
pub struct Job {
    name: String,
    description: String,
    steps: u8,
    handle: Arc<dyn JobHandle>,
}

impl Job {
    pub fn new<H: JobHandle>(name: &str, description: &str, steps: u8, handle: H) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            handle: Arc::new(handle),
            steps,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn steps(&self) -> u8 {
        self.steps
    }

    pub fn handle(&self) -> Arc<dyn crate::jobs::JobHandle> {
        self.handle.clone()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JobRegistryError {
    #[error("Job already exists")]
    AlreadyExists,
    #[error("Job not found")]
    NotFound,
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

    pub fn register_job(&mut self, id: impl Into<JobId>, job: Job) -> RegistryResult<()> {
        let id = id.into();

        if self.jobs.contains_key(&id) {
            return Err(JobRegistryError::AlreadyExists);
        }

        self.jobs.insert(id, job);
        Ok(())
    }
}

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
    started_at: Option<OffsetDateTime>,
    #[ts(type = "Date")]
    #[serde(with = "time::serde::rfc3339::option")]
    completed_at: Option<OffsetDateTime>,
    #[ts(type = "Date")]
    #[serde(with = "time::serde::rfc3339::option")]
    cancelled_at: Option<OffsetDateTime>,
    completed_successfully: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "bindings.ts")]
#[serde(rename_all = "camelCase")]
pub struct JobState {
    status: JobStatus,
    current_step: u8,
    #[serde(skip)]
    token: CancellationToken,
}

impl JobState {
    pub fn status(&self) -> &JobStatus {
        &self.status
    }

    pub fn current_step(&self) -> &u8 {
        &self.current_step
    }

    pub fn cancel_token(&self) -> &CancellationToken {
        &self.token
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase", tag = "kind")]
#[ts(export, export_to = "bindings.ts", rename = "JobRegistryEvent")]
pub enum JobManagerEvent {
    Started {
        source: JobStateId,
    },
    Completed {
        source: JobStateId,
    },
    Cancelled {
        source: JobStateId,
    },
    Progress {
        source: JobStateId,
        current: u64,
        total: u64,
        step: u8,
    },
}

impl JobManagerEvent {
    pub fn source(&self) -> &JobStateId {
        match self {
            JobManagerEvent::Started { source, .. } => source,
            JobManagerEvent::Completed { source, .. } => source,
            JobManagerEvent::Cancelled { source, .. } => source,
            JobManagerEvent::Progress { source, .. } => source,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JobManagerError {
    #[error(transparent)]
    Registry(#[from] JobRegistryError),
    #[error("Unique job already has been queued")]
    AlreadyQueued,
    #[error("Job state not found")]
    StateNotFound,
    #[error("Job report not found")]
    ReportNotFound,
}

#[derive(Debug)]
pub struct JobManager {
    registry: JobRegistry,
    queue: Arc<Queue>,
    events: broadcast::Sender<JobManagerEvent>,
    reports: Arc<Mutex<JobReports>>,
    states: Arc<Mutex<JobStates>>,
}

impl JobManager {
    pub fn new(registry: JobRegistry) -> Self {
        let (events, _) = broadcast::channel(256);
        let states: Arc<Mutex<JobStates>> = Arc::new(Mutex::new(BTreeMap::new()));
        let reports: Arc<Mutex<JobReports>> = Arc::new(Mutex::new(
            registry
                .jobs
                .keys()
                .map(|id| (id.clone(), JobExecutionReport::default()))
                .collect(),
        ));

        let queue = Arc::new(Queue::new());

        let events_clone = events.clone();
        let state_clone = states.clone();
        let reports_clone = reports.clone();
        let queued = queue.clone();
        tokio::spawn(async move {
            loop {
                if let Some(QueueItem {
                    state_id,
                    entry: (job_id, job),
                    cancel_token,
                    job_events,
                    ..
                }) = {
                    let mut queued = queued.list.lock().await;
                    queued.pop_front()
                } {
                    let (tx, mut rx) = mpsc::channel::<JobEvent>(256);

                    let manager_events = events_clone.clone();
                    let state = state_clone.clone();
                    let reports = reports_clone.clone();
                    let id = job_id.clone();
                    tokio::spawn(async move {
                        while let Some(event) = rx.recv().await {
                            let _ = job_events.send(event.clone()).await;

                            match event {
                                JobEvent::Started => {
                                    let mut state = state.lock().await;
                                    if let Some(state) = state.get_mut(&state_id) {
                                        state.status = JobStatus::InProgress;
                                    }

                                    send_event(
                                        &manager_events,
                                        &JobManagerEvent::Started { source: state_id },
                                    )
                                    .await;

                                    let mut reports = reports.lock().await;
                                    Self::report(&mut reports, &id)
                                        .started_at
                                        .replace(OffsetDateTime::now_utc());
                                }
                                JobEvent::Progress {
                                    current,
                                    total,
                                    step,
                                } => {
                                    let mut state = state.lock().await;
                                    if let Some(state) = state.get_mut(&state_id)
                                        && state.current_step < step
                                    {
                                        state.current_step = step;
                                    }

                                    send_event(
                                        &manager_events,
                                        &JobManagerEvent::Progress {
                                            source: state_id,
                                            current,
                                            total,
                                            step,
                                        },
                                    )
                                    .await;
                                }
                                JobEvent::Completed => {
                                    let mut state = state.lock().await;
                                    state.remove(&state_id);

                                    send_event(
                                        &manager_events,
                                        &JobManagerEvent::Completed { source: state_id },
                                    )
                                    .await;

                                    let mut reports = reports.lock().await;
                                    let report = Self::report(&mut reports, &id);
                                    report.completed_at.replace(OffsetDateTime::now_utc());
                                    report.completed_successfully = true;
                                }
                                JobEvent::Cancelled => {
                                    let mut state = state.lock().await;
                                    state.remove(&state_id);

                                    send_event(
                                        &manager_events,
                                        &JobManagerEvent::Cancelled { source: state_id },
                                    )
                                    .await;

                                    let mut reports = reports.lock().await;
                                    Self::report(&mut reports, &id)
                                        .cancelled_at
                                        .replace(OffsetDateTime::now_utc());
                                }
                            }
                        }
                    });

                    if let Err(err) = job.handle().execute(cancel_token, &tx).await {
                        tracing::error!("Job failed: {err}");
                        state_clone.lock().await.remove(&state_id);
                        let mut reports = reports_clone.lock().await;
                        let report = Self::report(&mut reports, &job_id);

                        report.completed_at.replace(OffsetDateTime::now_utc());
                        report.completed_successfully = false;
                    }
                } else {
                    queued.notify.notified().await;
                }
            }
        });

        Self {
            queue,
            states,
            reports,
            events,
            registry,
        }
    }

    pub async fn queue(
        &self,
        job_id: JobId,
        unique: bool,
        high_priority: bool,
    ) -> ManagerResult<JobHandler> {
        if unique
            && self
                .queue
                .any(|item| item.unique && item.entry.0 == job_id)
                .await
        {
            return Err(JobManagerError::AlreadyQueued);
        }

        tracing::debug!("Queueing job: {job_id}");

        let id = OffsetDateTime::now_utc().unix_timestamp();
        let (tx, rx) = mpsc::channel(256);
        let state = JobState {
            status: JobStatus::Pending,
            current_step: 0,
            token: CancellationToken::new(),
        };

        self.queue
            .add_item(
                QueueItem {
                    state_id: id,
                    cancel_token: state.cancel_token().clone(),
                    job_events: tx,
                    unique,
                    entry: (
                        job_id.clone(),
                        self.registry
                            .jobs
                            .get(&job_id)
                            .cloned()
                            .ok_or(JobRegistryError::NotFound)?,
                    ),
                },
                high_priority,
            )
            .await;

        self.states.lock().await.insert(id, state);

        tracing::debug!("Job queued: {job_id}");

        Ok(JobHandler {
            state_id: id,
            job_id,
            events: rx,
        })
    }

    pub async fn cancel_job(&self, state_id: JobStateId) -> ManagerResult<()> {
        let mut states = self.states.lock().await;

        if let Some(state) = states.get_mut(&state_id) {
            if state.status == JobStatus::InProgress {
                let mut reports = self.reports.lock().await;
                state.token.cancel();

                let report = Self::report(&mut reports, &state_id.to_string());
                report.cancelled_at.replace(OffsetDateTime::now_utc());
                report.completed_successfully = false;

                Ok(())
            } else {
                states.remove(&state_id);
                self.queue.remove_item(state_id, false).await;

                Ok(())
            }
        } else {
            Err(JobManagerError::StateNotFound)
        }
    }

    pub fn events(&self) -> broadcast::Receiver<JobManagerEvent> {
        self.events.subscribe()
    }

    pub fn registry(&self) -> &JobRegistry {
        &self.registry
    }

    pub async fn job_queue_rank(&self, state_id: JobStateId) -> ManagerResult<usize> {
        self.queue
            .item_rank(state_id)
            .await
            .ok_or(JobManagerError::StateNotFound)
    }

    pub async fn job_report(&self, job_id: &JobId) -> ManagerResult<JobExecutionReport> {
        let reports = self.reports.lock().await;

        reports
            .get(job_id)
            .cloned()
            .ok_or(JobManagerError::ReportNotFound)
    }

    pub async fn unique_job_state_id(&self, job_id: &JobId) -> ManagerResult<JobStateId> {
        let list = self.queue.list.lock().await;

        list.iter()
            .find(|item| item.unique && item.entry.0 == *job_id)
            .map(|item| item.state_id)
            .ok_or(JobManagerError::StateNotFound)
    }

    pub async fn job_state(&self, state_id: JobStateId) -> ManagerResult<JobState> {
        let states = self.states.lock().await;

        states
            .get(&state_id)
            .cloned()
            .ok_or(JobManagerError::StateNotFound)
    }

    fn report<'r>(reports: &'r mut JobReports, job_id: &JobId) -> &'r mut JobExecutionReport {
        reports
            .get_mut(job_id)
            .expect("Job not found, this shouldn't happen...")
    }
}

async fn send_event(tx: &broadcast::Sender<JobManagerEvent>, event: &JobManagerEvent) {
    if let Err(err) = tx.send(event.clone()) {
        tracing::error!("Failed to send manager event: {err}, This shouldn't happen...");
    };
}
