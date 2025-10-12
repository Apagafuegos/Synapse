use axum::{
    extract::{Path, State},
    response::Html,
};
use uuid::Uuid;

use crate::{error_handling::AppError, AppState};

/// Serve the streaming dashboard template
pub async fn streaming_dashboard(
    Path(project_id): Path<Uuid>,
    State(_state): State<AppState>,
) -> Result<Html<String>, AppError> {
    let template = include_str!("../../templates/streaming_dashboard.html");
    
    // Replace template variables
    let html = template.replace("{{PROJECT_ID}}", &project_id.to_string());
    
    Ok(Html(html))
}