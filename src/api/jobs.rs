use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    response::{IntoResponse, Result},
    routing::{get, post},
};
use serde::Serialize;
use ts_rs::TS;

use crate::state::{AppState, Job, JobExecutionReport, JobStateId};

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "bindings.ts")]
pub struct RegistryJob {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: u8,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/jobs/{id}/queue", post(queue_job).get(queued_job_id))
        .route("/api/jobs", get(list_jobs))
}

async fn list_jobs(State(app): State<AppState>) -> Result<Json<Vec<RegistryJob>>> {
    let jobs = app
        .job_manager
        .registry()
        .jobs()
        .iter()
        .map(|(id, job)| RegistryJob {
            id: id.to_string(),
            name: job.name().to_string(),
            description: job.description().to_string(),
            steps: job.steps(),
        })
        .collect::<Vec<_>>();

    Ok(Json(jobs))
}

async fn queued_job_id(
    State(app): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<JobStateId>> {
    Ok(Json(app.job_manager.unique_job_state_id(&id).await?))
}

async fn queue_job(
    State(app): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<JobStateId>> {
    Ok(Json(app.job_manager.queue(id, true, true).await?.id()))
}
