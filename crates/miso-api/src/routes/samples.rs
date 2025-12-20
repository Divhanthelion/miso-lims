//! Sample route handlers.

use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use validator::Validate;

use miso_application::dto::{
    CreatePlainSampleRequest, SampleResponse, SampleSummary, UpdateSampleRequest,
};
use miso_domain::repositories::{ProjectRepository, SampleRepository};

use crate::{error::ApiError, middleware::AuthUser, state::AppState};

/// Creates sample routes.
pub fn routes<PR, SR>() -> Router<AppState<PR, SR>>
where
    PR: ProjectRepository + 'static,
    SR: SampleRepository + 'static,
{
    Router::new()
        .route("/", get(list_samples).post(create_sample))
        .route("/:id", get(get_sample).put(update_sample).delete(delete_sample))
        .route("/barcode/:barcode", get(get_sample_by_barcode))
        .route("/project/:project_id", get(list_samples_by_project))
}

/// Query parameters for listing samples.
#[derive(Debug, Deserialize)]
pub struct ListSamplesQuery {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub project_id: Option<i32>,
}

/// List samples.
async fn list_samples<PR: ProjectRepository, SR: SampleRepository>(
    State(state): State<AppState<PR, SR>>,
    Query(query): Query<ListSamplesQuery>,
) -> Result<Json<Vec<SampleSummary>>, ApiError> {
    if let Some(project_id) = query.project_id {
        let samples = state
            .sample_service
            .list_samples_by_project(project_id, query.limit, query.offset)
            .await?;
        Ok(Json(samples))
    } else {
        Err(ApiError::BadRequest("project_id is required".to_string()))
    }
}

/// List samples by project.
async fn list_samples_by_project<PR: ProjectRepository, SR: SampleRepository>(
    State(state): State<AppState<PR, SR>>,
    Path(project_id): Path<i32>,
    Query(query): Query<ListSamplesQuery>,
) -> Result<Json<Vec<SampleSummary>>, ApiError> {
    let samples = state
        .sample_service
        .list_samples_by_project(project_id, query.limit, query.offset)
        .await?;
    Ok(Json(samples))
}

/// Get a sample by ID.
async fn get_sample<PR: ProjectRepository, SR: SampleRepository>(
    State(state): State<AppState<PR, SR>>,
    Path(id): Path<i32>,
) -> Result<Json<SampleResponse>, ApiError> {
    let sample = state.sample_service.get_sample(id).await?;
    Ok(Json(sample))
}

/// Get a sample by barcode.
async fn get_sample_by_barcode<PR: ProjectRepository, SR: SampleRepository>(
    State(state): State<AppState<PR, SR>>,
    Path(barcode): Path<String>,
) -> Result<Json<SampleResponse>, ApiError> {
    let sample = state.sample_service.get_sample_by_barcode(&barcode).await?;
    Ok(Json(sample))
}

/// Create a new sample.
async fn create_sample<PR: ProjectRepository, SR: SampleRepository>(
    State(state): State<AppState<PR, SR>>,
    user: AuthUser,
    Json(request): Json<CreatePlainSampleRequest>,
) -> Result<Json<SampleResponse>, ApiError> {
    if !user.can_edit() {
        return Err(ApiError::Forbidden);
    }

    request.validate()?;

    let sample = state
        .sample_service
        .create_plain_sample(request, &user.username)
        .await?;

    Ok(Json(sample))
}

/// Update a sample.
async fn update_sample<PR: ProjectRepository, SR: SampleRepository>(
    State(state): State<AppState<PR, SR>>,
    Path(id): Path<i32>,
    user: AuthUser,
    Json(request): Json<UpdateSampleRequest>,
) -> Result<Json<SampleResponse>, ApiError> {
    if !user.can_edit() {
        return Err(ApiError::Forbidden);
    }

    request.validate()?;

    let sample = state.sample_service.update_sample(id, request).await?;

    Ok(Json(sample))
}

/// Delete a sample.
async fn delete_sample<PR: ProjectRepository, SR: SampleRepository>(
    State(state): State<AppState<PR, SR>>,
    Path(id): Path<i32>,
    user: AuthUser,
) -> Result<(), ApiError> {
    if !user.can_delete() {
        return Err(ApiError::Forbidden);
    }

    state.sample_service.delete_sample(id).await?;

    Ok(())
}

