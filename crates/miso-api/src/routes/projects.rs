//! Project route handlers.

use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use validator::Validate;

use miso_application::dto::{
    CreateProjectRequest, ProjectResponse, ProjectSummary, UpdateProjectRequest,
};
use miso_domain::repositories::{ProjectRepository, SampleRepository};

use crate::{error::ApiError, middleware::AuthUser, state::AppState};

/// Creates project routes.
pub fn routes<PR, SR>() -> Router<AppState<PR, SR>>
where
    PR: ProjectRepository + 'static,
    SR: SampleRepository + 'static,
{
    Router::new()
        .route("/", get(list_projects).post(create_project))
        .route("/:id", get(get_project).put(update_project).delete(delete_project))
}

/// Query parameters for listing projects.
#[derive(Debug, Deserialize)]
pub struct ListProjectsQuery {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

/// List all projects.
async fn list_projects<PR: ProjectRepository, SR: SampleRepository>(
    State(state): State<AppState<PR, SR>>,
    Query(query): Query<ListProjectsQuery>,
) -> Result<Json<Vec<ProjectSummary>>, ApiError> {
    let projects = state
        .project_service
        .list_projects(query.limit, query.offset)
        .await?;

    Ok(Json(projects))
}

/// Get a project by ID.
async fn get_project<PR: ProjectRepository, SR: SampleRepository>(
    State(state): State<AppState<PR, SR>>,
    Path(id): Path<i32>,
) -> Result<Json<ProjectResponse>, ApiError> {
    let project = state.project_service.get_project(id).await?;
    Ok(Json(project))
}

/// Create a new project.
async fn create_project<PR: ProjectRepository, SR: SampleRepository>(
    State(state): State<AppState<PR, SR>>,
    user: AuthUser,
    Json(request): Json<CreateProjectRequest>,
) -> Result<Json<ProjectResponse>, ApiError> {
    request.validate()?;

    let project = state
        .project_service
        .create_project(request, &user.username)
        .await?;

    Ok(Json(project))
}

/// Update a project.
async fn update_project<PR: ProjectRepository, SR: SampleRepository>(
    State(state): State<AppState<PR, SR>>,
    Path(id): Path<i32>,
    user: AuthUser,
    Json(request): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectResponse>, ApiError> {
    if !user.can_edit() {
        return Err(ApiError::Forbidden);
    }

    request.validate()?;

    let project = state.project_service.update_project(id, request).await?;

    Ok(Json(project))
}

/// Delete a project.
async fn delete_project<PR: ProjectRepository, SR: SampleRepository>(
    State(state): State<AppState<PR, SR>>,
    Path(id): Path<i32>,
    user: AuthUser,
) -> Result<(), ApiError> {
    if !user.can_delete() {
        return Err(ApiError::Forbidden);
    }

    state.project_service.delete_project(id).await?;

    Ok(())
}

