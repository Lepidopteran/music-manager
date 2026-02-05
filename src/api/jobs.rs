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
    AppState, JobExecutionReport, JobManager, JobStateId, JobStates, registry::JobId,
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
        .route("/api/jobs/{id}/queue", post(queue_job).get(queued_job_id))
        .route("/api/jobs/{id}/report", get(job_report))
        .route("/api/jobs/state", get(state))
        .route("/api/jobs", get(list_jobs))
}

async fn queued_job_id(
    State(app): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<JobStateId>> {
    Ok(Json(app.job_manager.unique_job_state_id(&id).await?))
}

async fn queue_job(
    State(manager): State<Arc<JobManager>>,
    Path(id): Path<JobId>,
) -> Result<Json<JobStateId>> {
    Ok(Json(manager.queue(id, true, true).await?.id()))
}

async fn job_report(
    State(manager): State<Arc<JobManager>>,
    Path(job_id): Path<JobId>,
) -> Result<Json<JobExecutionReport>> {
    Ok(Json(manager.job_report(&job_id).await?))
}

async fn state(State(manager): State<Arc<JobManager>>) -> Result<Json<JobStates>> {
    Ok(Json(manager.states().await))
}

async fn list_jobs(State(manager): State<Arc<JobManager>>) -> Result<Json<Vec<RegistryJob>>> {
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
