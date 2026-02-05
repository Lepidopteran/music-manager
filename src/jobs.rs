use std::fmt::Debug;

use async_trait::async_trait;
use color_eyre::eyre::Result;
use serde::Serialize;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

mod scan_songs;
pub use scan_songs::*;

type Sender = mpsc::Sender<JobEvent>;

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum JobEvent {
    #[default]
    Started,
    Completed,
    Cancelled,
    Progress {
        current: u64,
        total: u64,
        step: u8,
    },
}

#[async_trait]
pub trait JobHandle: 'static + Send + Sync + Debug {
    async fn execute(&self, cancel_token: CancellationToken, tx: &Sender) -> Result<()>;
}

/// Util function to check if a job was cancelled, and if so, send a cancel event
fn is_job_token_cancelled(token: &CancellationToken, tx: &Sender) -> bool {
    if token.is_cancelled() {
        let _ = tx.blocking_send(JobEvent::Cancelled);
        true
    } else {
        false
    }
}
