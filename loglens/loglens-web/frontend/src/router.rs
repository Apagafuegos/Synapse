use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Dashboard,
    #[at("/projects")]
    Projects,
    #[at("/projects/:id")]
    ProjectDetail { id: String },
    #[at("/projects/:project_id/analyses/:analysis_id")]
    AnalysisView { project_id: String, analysis_id: String },
    #[at("/settings")]
    Settings,
}