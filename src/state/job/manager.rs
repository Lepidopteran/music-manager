use std::{collections::VecDeque, sync::Arc};

use serde::Serialize;
use time::OffsetDateTime;
use tokio::sync::{Mutex, Notify, broadcast, oneshot};
use tokio_util::sync::CancellationToken;

use super::*;

pub type JobStates = BTreeMap<JobStateId, JobState>;
pub type JobReports = BTreeMap<JobId, JobExecutionReport>;

type Result<T, E = JobManagerError> = std::result::Result<T, E>;

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum JobManagerEvent {
    Started {
        source: JobStateId,
    },
    Completed {
        source: JobStateId,
        report: JobExecutionReport,
    },
    Cancelled {
        source: JobStateId,
    },
    Warning {
        source: JobStateId,
        message: String,
    },
    Failed {
        source: JobStateId,
        message: String,
    },
    StepCompleted {
        source: JobStateId,
        step: u8,
        value: Option<String>,
    },
    Progress {
        source: JobStateId,
        current: u64,
        total: u64,
        step: u8,
    },
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
        let (events, _) = broadcast::channel(1024 * 2);
        let states: Arc<Mutex<JobStates>> = Arc::new(Mutex::new(BTreeMap::new()));
        let reports: Arc<Mutex<JobReports>> = Arc::new(Mutex::new(
            registry
                .jobs()
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
                    job,
                    state_id,
                    report_id,
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
                    let job_token = cancel_token.child_token();
                    tokio::spawn(async move {
                        while let Some(event) = rx.recv().await
                            && !job_token.is_cancelled()
                        {
                            let _ = job_events.send(event.clone()).await;

                            match event {
                                JobEvent::Progress {
                                    current,
                                    total,
                                    step,
                                } => {
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
                                JobEvent::StepCompleted { step, value } => {
                                    let mut state = state.lock().await;
                                    if let Some(state) = state.get_mut(&state_id) {
                                        state.current_step = step + 1;
                                        state
                                            .values
                                            .insert(step, value.clone().unwrap_or_default());
                                    }

                                    drop(state);

                                    send_event(
                                        &manager_events,
                                        &JobManagerEvent::StepCompleted {
                                            source: state_id,
                                            step,
                                            value,
                                        },
                                    )
                                    .await;
                                }
                                JobEvent::Warning { message } => {
                                    send_event(
                                        &manager_events,
                                        &JobManagerEvent::Warning {
                                            source: state_id,
                                            message,
                                        },
                                    )
                                    .await;
                                }
                            }
                        }

                        drop(job_events);
                    });

                    let mut states = state_clone.lock().await;
                    let state = states.get_mut(&state_id).unwrap();
                    state.status = JobStatus::InProgress;

                    drop(states);

                    send_event(
                        &events_clone,
                        &JobManagerEvent::Started { source: state_id },
                    )
                    .await;

                    let result = job.execute(cancel_token.child_token(), tx).await;

                    if result.is_ok() && !cancel_token.is_cancelled() {
                        let mut reports = reports_clone.lock().await;
                        let report = Self::report(&mut reports, &report_id);
                        report.completed_at.replace(OffsetDateTime::now_utc());
                        report.completed_successfully = true;

                        send_event(
                            &events_clone,
                            &JobManagerEvent::Completed {
                                source: state_id,
                                report: report.clone(),
                            },
                        )
                        .await;

                    } else if let Err(err) = result.as_ref() {
                        tracing::error!("Job failed: {err}");
                        let mut reports = reports_clone.lock().await;
                        let report = Self::report(&mut reports, &report_id);
                        report.completed_at.replace(OffsetDateTime::now_utc());
                        report.completed_successfully = false;

                        send_event(
                            &events_clone,
                            &JobManagerEvent::Failed {
                                source: state_id,
                                message: err.to_string(),
                            },
                        )
                        .await;
                    } else {
                        let mut reports = reports_clone.lock().await;
                        let report = Self::report(&mut reports, &report_id);
                        report.cancelled_at.replace(OffsetDateTime::now_utc());
                        report.completed_successfully = false;

                        send_event(
                            &events_clone,
                            &JobManagerEvent::Cancelled { source: state_id },
                        )
                        .await;
                    }

                    state_clone.lock().await.remove(&state_id);
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
        job_id: impl Into<JobId>,
        unique: bool,
        high_priority: bool,
    ) -> Result<JobHandler> {
        let job_id = job_id.into();

        if unique
            && self
                .queue
                .any(|item| item.unique && item.report_id == job_id)
                .await
        {
            return Err(JobManagerError::AlreadyQueued);
        }

        tracing::debug!("Queueing job: {job_id}");

        let id = JobStateId::new_v4();
        let (tx, rx) = mpsc::channel(256);
        let state = JobState::new(job_id.clone());
        let cancel_token = state.token.child_token();

        let mut states = self.states.lock().await;
        states.insert(id, state);

        tracing::debug!("States: {states:#?}");

        drop(states);

        self.queue
            .add_item(
                QueueItem {
                    unique,
                    cancel_token,
                    state_id: id,
                    job_events: tx,
                    report_id: job_id.clone(),
                    job: self
                        .registry
                        .jobs()
                        .get(&job_id)
                        .map(|job| job.handle())
                        .ok_or(JobRegistryError::NotFound)?,
                },
                high_priority,
            )
            .await;

        tracing::debug!("Job queued: {job_id}");

        Ok(JobHandler {
            state_id: id,
            job_id,
            events: rx,
        })
    }

    pub async fn cancel_job(&self, state_id: JobStateId) -> Result<()> {
        let mut states = self.states.lock().await;
        tracing::debug!("Cancelling job: {state_id}, {states:#?}");

        if let Some(state) = states.get_mut(&state_id) {
            if state.status == JobStatus::InProgress {
                let mut reports = self.reports.lock().await;
                state.token.cancel();

                let report = Self::report(&mut reports, &state.job_id);
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

    pub async fn reports(&self) -> JobReports {
        self.reports.lock().await.clone()
    }

    pub async fn states(&self) -> JobStates {
        self.states.lock().await.clone()
    }

    fn report<'r>(reports: &'r mut JobReports, job_id: &JobId) -> &'r mut JobExecutionReport {
        reports
            .get_mut(job_id)
            .expect("Job not found, this shouldn't happen...")
    }
}

async fn send_event(tx: &broadcast::Sender<JobManagerEvent>, event: &JobManagerEvent) {
    let _ = tx.send(event.clone());
}

#[derive(Debug)]
pub struct JobHandler {
    state_id: JobStateId,
    job_id: JobId,
    events: mpsc::Receiver<JobEvent>,
}

impl JobHandler {
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

#[derive(Debug)]
struct QueueItem {
    job: Arc<dyn JobHandle>,
    report_id: JobId,
    state_id: JobStateId,
    cancel_token: CancellationToken,
    job_events: mpsc::Sender<JobEvent>,
    unique: bool,
}

#[derive(Debug)]
struct Queue {
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use async_trait::async_trait;
    use color_eyre::eyre::Result;
    use test_log::test;
    use tokio::time::sleep;
    use tokio_util::sync::CancellationToken;

    #[derive(Debug)]
    struct TestJob {}

    #[async_trait]
    impl JobHandle for TestJob {
        async fn execute(
            &self,
            token: CancellationToken,
            tx: mpsc::Sender<JobEvent>,
        ) -> Result<()> {
            let mut index = 0;

            loop {
                tokio::select! {
                    _ = token.cancelled() => break,
                    _ = sleep(Duration::from_millis(100)) => {
                        index += 1;
                        tx.send(JobEvent::Progress {
                            current: index,
                            total: u64::MAX,
                            step: 1,
                        })
                        .await
                        .expect("Failed to send event");
                    }
                }
            }

            Ok(())
        }
    }
    fn registry() -> JobRegistry {
        let mut registry = JobRegistry::new();
        registry
            .register_job(
                "test",
                Job::new(
                    JobInfo::new(
                        "Test Job",
                        "Literally just a test job, what did you expect?",
                        BTreeMap::new(),
                    ),
                    TestJob {},
                ),
            )
            .expect("Failed to register job");

        registry
    }

    #[test(tokio::test)]
    async fn test_adding_fast_jobs() -> Result<()> {
        let manager = JobManager::new(registry());

        for _ in 0..100 {
            let _ = manager.queue("test".to_string(), false, false).await?;
        }

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_failing_adding_duplicate_jobs() -> Result<()> {
        let manager = JobManager::new(registry());

        let _ = manager.queue("test", true, true).await;
        assert!(manager.queue("test", true, true).await.is_err());

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_cancelling_jobs() -> Result<()> {
        let manager = JobManager::new(registry());
        let mut job = manager.queue("test", true, true).await?;
        let id = job.state_id;
        log::debug!("{job:#?}");

        let mut manager_events = manager.events();
        tokio::spawn(async move {
            while let Ok(event) = manager_events.recv().await {
                tracing::info!("Job Managaer Event: {event:?}");
            }
        });

        let job_events = tokio::spawn(async move {
            while let Some(event) = job.events().recv().await {
                tracing::info!("Job Event: {event:?}");
            }
        });

        sleep(Duration::from_secs(1)).await;

        tracing::debug!("Attempting to cancel \"{id}\"");
        manager.cancel_job(id).await?;

        tracing::debug!("Cancelled \"{id}\"");

        job_events.await?;

        Ok(())
    }
}
