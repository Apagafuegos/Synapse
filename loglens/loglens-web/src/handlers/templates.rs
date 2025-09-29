use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
};
use uuid::Uuid;

use crate::AppState;

/// Serve the streaming dashboard template
pub async fn streaming_dashboard(
    Path(project_id): Path<Uuid>,
    State(_state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let template = include_str!("../../templates/streaming_dashboard.html");
    
    // Replace template variables
    let html = template.replace("{{PROJECT_ID}}", &project_id.to_string());
    
    Ok(Html(html))
}