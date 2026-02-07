use std::collections::BTreeMap;

use serde::Serialize;
use time::OffsetDateTime;
use tokio_util::sync::CancellationToken;
use ts_rs::TS;

pub type JobStateId = i64;

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
    started_at: Option<OffsetDateTime>,
    #[ts(type = "Date")]
    #[serde(with = "time::serde::rfc3339::option")]
    completed_at: Option<OffsetDateTime>,
    #[ts(type = "Date")]
    #[serde(with = "time::serde::rfc3339::option")]
    cancelled_at: Option<OffsetDateTime>,
    completed_successfully: bool,
}
