use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::{Dashboard, ProjectDetail, Projects, SettingsPage, AnalysisView};
use crate::components::Layout;
use crate::router::Route;

fn switch(routes: Route) -> Html {
    match routes {
        Route::Dashboard => html! { <Dashboard /> },
        Route::Projects => html! { <Projects /> },
        Route::ProjectDetail { id } => html! { <ProjectDetail id={id} /> },
        Route::AnalysisView { project_id, analysis_id } => html! { <AnalysisView project_id={project_id} analysis_id={analysis_id} /> },
        Route::Settings => html! { <SettingsPage /> },
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Layout>
                <Switch<Route> render={switch} />
            </Layout>
        </BrowserRouter>
    }
}