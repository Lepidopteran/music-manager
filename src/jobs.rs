use std::fmt::Debug;

use async_trait::async_trait;
use color_eyre::eyre::Result;
use serde::Serialize;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

mod scan_songs;
pub use scan_songs::*;

type Sender = mpsc::Sender<JobEvent>;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum JobEvent {
    StepCompleted {
        step: u8,
        value: Option<String>,
    },
    Warning {
        message: String,
    },
    Progress {
        current: u64,
        total: u64,
        step: u8,
    },
}

/// Util function to send job events without freaking out if the channel is closed
async fn emit_event(tx: &Sender, event: JobEvent) {
    if let Err(err) = tx.send(event).await {
        tracing::error!("Failed to send event: {err}");
    }
}

/// Util function to send blocking job events without freaking out if the channel is closed
fn emit_blocking_event(tx: &Sender, event: JobEvent) {
    if let Err(err) = tx.blocking_send(event) {
        tracing::error!("Failed to send event: {err}");
    }
}

#[async_trait]
pub trait JobHandle: 'static + Send + Sync + Debug {
    async fn execute(&self, cancel_token: CancellationToken, tx: Sender) -> Result<()>;
}
