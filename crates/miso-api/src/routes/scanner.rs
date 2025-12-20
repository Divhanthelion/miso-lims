//! VisionMate scanner route handlers.

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use miso_application::dto::{RackScanResult, TubeScanResult};
use miso_domain::repositories::{ProjectRepository, SampleRepository};

use crate::{error::ApiError, middleware::AuthUser, state::AppState};

/// Creates scanner routes.
pub fn routes<PR, SR>() -> Router<AppState<PR, SR>>
where
    PR: ProjectRepository + 'static,
    SR: SampleRepository + 'static,
{
    Router::new()
        .route("/status", get(scanner_status))
        .route("/scan", post(scan_rack))
}

/// Scanner status response.
#[derive(Serialize)]
pub struct ScannerStatusResponse {
    pub connected: bool,
    pub ip: Option<String>,
    pub message: String,
}

/// Get scanner status.
async fn scanner_status<PR: ProjectRepository, SR: SampleRepository>(
    State(state): State<AppState<PR, SR>>,
) -> Json<ScannerStatusResponse> {
    match &state.scanner {
        Some(scanner) => {
            let connected = scanner.ping().await;
            Json(ScannerStatusResponse {
                connected,
                ip: Some(format!("configured")),
                message: if connected {
                    "Scanner is ready".to_string()
                } else {
                    "Scanner is not responding".to_string()
                },
            })
        }
        None => Json(ScannerStatusResponse {
            connected: false,
            ip: None,
            message: "No scanner configured".to_string(),
        }),
    }
}

/// Scan request.
#[derive(Deserialize)]
pub struct ScanRequest {
    /// Optional override scanner IP
    pub ip: Option<String>,
}

/// Trigger a rack scan.
async fn scan_rack<PR: ProjectRepository, SR: SampleRepository>(
    State(state): State<AppState<PR, SR>>,
    user: AuthUser,
    Json(_request): Json<ScanRequest>,
) -> Result<Json<RackScanResult>, ApiError> {
    if !user.can_edit() {
        return Err(ApiError::Forbidden);
    }

    let scanner = state.scanner.as_ref().ok_or_else(|| {
        ApiError::BadRequest("No scanner configured".to_string())
    })?;

    let result = scanner.scan().await.map_err(|e| {
        ApiError::BadRequest(format!("Scan failed: {}", e))
    })?;

    // Convert scanner result to API response
    let tubes: Vec<TubeScanResult> = result
        .positions
        .into_iter()
        .map(|(position, barcode)| TubeScanResult {
            position,
            barcode,
            sample_id: None, // TODO: Look up sample by barcode
            sample_name: None,
        })
        .collect();

    let response = RackScanResult {
        rack_barcode: result.rack_barcode,
        tubes,
        empty_count: result.empty_positions.len(),
        error_count: result.error_positions.len(),
    };

    Ok(Json(response))
}

