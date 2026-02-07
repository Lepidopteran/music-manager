use std::{collections::BTreeMap, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, State},
    response::{IntoResponse, Result},
    routing::{get, post},
};
use serde::Serialize;
use ts_rs::TS;

use crate::state::{
    AppState, JobManager,
    job::{
        JobId, JobStateId,
        manager::{JobReports, JobStates},
    },
};

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "bindings.ts")]
pub struct RegistryJob {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: BTreeMap<u8, String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/jobs/{id}/queue", post(queue_job))
        .route("/api/jobs/state", get(state))
        .route("/api/jobs/state/{id}/cancel", post(cancel_job))
        .route("/api/jobs/reports", get(job_reports))
        .route("/api/jobs", get(list_jobs))
}

async fn queue_job(
    State(manager): State<JobManager>,
    Path(id): Path<JobId>,
) -> Result<Json<JobStateId>> {
    Ok(Json(manager.queue(id, true, true).await?.id()))
}

async fn job_reports(State(manager): State<JobManager>) -> Result<Json<JobReports>> {
    Ok(Json(manager.reports().await))
}

async fn state(State(manager): State<JobManager>) -> Result<Json<JobStates>> {
    Ok(Json(manager.states().await))
}

async fn cancel_job(State(manager): State<JobManager>, Path(id): Path<JobStateId>) -> Result<()> {
    Ok(manager.cancel_job(id).await?)
}

async fn list_jobs(State(manager): State<JobManager>) -> Result<Json<Vec<RegistryJob>>> {
    let jobs = manager
        .registry()
        .jobs()
        .iter()
        .map(|(id, job)| {
            let info = job.info().clone();

            RegistryJob {
                id: id.to_string(),
                name: info.name,
                description: info.description,
                steps: info.steps,
            }
        })
        .collect::<Vec<_>>();

    Ok(Json(jobs))
}
