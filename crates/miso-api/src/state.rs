//! Application state shared across handlers.

use std::sync::Arc;

use miso_application::{ProjectService, SampleService};
use miso_domain::repositories::{ProjectRepository, SampleRepository};
use miso_infrastructure::hardware::scanner::VisionMateClient;
use miso_infrastructure::hardware::printer::ZebraPrinter;

use crate::Config;

/// Shared application state.
#[derive(Clone)]
pub struct AppState<PR: ProjectRepository, SR: SampleRepository> {
    /// Application configuration
    pub config: Arc<Config>,
    /// Project service
    pub project_service: Arc<ProjectService<PR>>,
    /// Sample service
    pub sample_service: Arc<SampleService<SR>>,
    /// VisionMate scanner client (optional)
    pub scanner: Option<Arc<VisionMateClient>>,
    /// Zebra printer client (optional)
    pub printer: Option<Arc<ZebraPrinter>>,
}

impl<PR: ProjectRepository, SR: SampleRepository> AppState<PR, SR> {
    /// Creates a new application state.
    pub fn new(
        config: Config,
        project_repo: Arc<PR>,
        sample_repo: Arc<SR>,
    ) -> Self {
        Self {
            config: Arc::new(config),
            project_service: Arc::new(ProjectService::new(project_repo)),
            sample_service: Arc::new(SampleService::new(sample_repo)),
            scanner: None,
            printer: None,
        }
    }

    /// Sets the VisionMate scanner client.
    pub fn with_scanner(mut self, scanner: VisionMateClient) -> Self {
        self.scanner = Some(Arc::new(scanner));
        self
    }

    /// Sets the Zebra printer client.
    pub fn with_printer(mut self, printer: ZebraPrinter) -> Self {
        self.printer = Some(Arc::new(printer));
        self
    }
}

