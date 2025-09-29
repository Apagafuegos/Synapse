use crate::components::ProjectCard;
use crate::types::Project;
use crate::services::api::ApiService;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[function_component(Projects)]
pub fn projects() -> Html {
    let projects = use_state(|| Vec::<Project>::new());
    let loading = use_state(|| true);
    let show_create_modal = use_state(|| false);
    let new_project_name = use_state(|| String::new());
    let new_project_description = use_state(|| String::new());
    let creating = use_state(|| false);

    let _api = ApiService::new();

    // Load projects on component mount
    {
        let projects = projects.clone();
        let loading = loading.clone();

        use_effect(move || {
            spawn_local(async move {
                // Simulate API call - replace with actual API call when backend is ready
                let project_list = vec![
                    Project {
                        id: "project-001".to_string(),
                        name: "Web Server Logs".to_string(),
                        description: Some("Production web server error analysis".to_string()),
                        created_at: chrono::Utc::now() - chrono::Duration::days(2),
                        updated_at: chrono::Utc::now() - chrono::Duration::hours(6),
                    },
                    Project {
                        id: "project-002".to_string(),
                        name: "Database Performance".to_string(),
                        description: Some("Query performance and connection issues".to_string()),
                        created_at: chrono::Utc::now() - chrono::Duration::days(5),
                        updated_at: chrono::Utc::now() - chrono::Duration::days(1),
                    },
                    Project {
                        id: "project-003".to_string(),
                        name: "API Gateway Logs".to_string(),
                        description: Some("Authentication and routing error analysis".to_string()),
                        created_at: chrono::Utc::now() - chrono::Duration::days(7),
                        updated_at: chrono::Utc::now() - chrono::Duration::days(3),
                    },
                    Project {
                        id: "project-004".to_string(),
                        name: "Mobile App Backend".to_string(),
                        description: None,
                        created_at: chrono::Utc::now() - chrono::Duration::days(10),
                        updated_at: chrono::Utc::now() - chrono::Duration::days(8),
                    },
                ];

                projects.set(project_list);
                loading.set(false);
            });
            || ()
        });
    }

    let on_create_project = {
        let show_create_modal = show_create_modal.clone();
        move |_| show_create_modal.set(true)
    };

    let on_close_modal = {
        let show_create_modal = show_create_modal.clone();
        let new_project_name = new_project_name.clone();
        let new_project_description = new_project_description.clone();
        move |_| {
            show_create_modal.set(false);
            new_project_name.set(String::new());
            new_project_description.set(String::new());
        }
    };

    let on_name_change = {
        let new_project_name = new_project_name.clone();
        move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            new_project_name.set(input.value());
        }
    };

    let on_description_change = {
        let new_project_description = new_project_description.clone();
        move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            new_project_description.set(input.value());
        }
    };

    let on_submit_create = {
        let projects = projects.clone();
        let creating = creating.clone();
        let show_create_modal = show_create_modal.clone();
        let new_project_name = new_project_name.clone();
        let new_project_description = new_project_description.clone();

        move |e: SubmitEvent| {
            e.prevent_default();

            if new_project_name.is_empty() {
                return;
            }

            let projects = projects.clone();
            let creating = creating.clone();
            let show_create_modal = show_create_modal.clone();
            let name = (*new_project_name).clone();
            let description = if new_project_description.is_empty() {
                None
            } else {
                Some((*new_project_description).clone())
            };
            let new_project_name = new_project_name.clone();
            let new_project_description = new_project_description.clone();

            spawn_local(async move {
                creating.set(true);

                // Simulate API call
                let new_project = Project {
                    id: format!("project-{:03}", projects.len() + 1),
                    name,
                    description,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };

                let mut updated_projects = (*projects).clone();
                updated_projects.insert(0, new_project);
                projects.set(updated_projects);

                creating.set(false);
                show_create_modal.set(false);
                new_project_name.set(String::new());
                new_project_description.set(String::new());
            });
        }
    };

    let on_delete_project = {
        let projects = projects.clone();
        move |project_id: String| {
            let mut updated_projects = (*projects).clone();
            updated_projects.retain(|p| p.id != project_id);
            projects.set(updated_projects);
        }
    };

    if *loading {
        return html! {
            <div class="flex items-center justify-center min-h-screen">
                <div class="animate-spin rounded-full h-32 w-32 border-b-2 border-blue-600"></div>
            </div>
        };
    }

    html! {
        <div class="space-y-6">
            // Header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">{"Projects"}</h1>
                    <p class="text-gray-600">{"Manage your log analysis projects"}</p>
                </div>
                <button
                    onclick={on_create_project.clone()}
                    class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-medium"
                >
                    {"+ New Project"}
                </button>
            </div>

            // Projects Grid
            if projects.is_empty() {
                <div class="text-center py-12">
                    <div class="text-gray-400 mb-4">
                        <svg class="w-16 h-16 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
                        </svg>
                    </div>
                    <h3 class="text-lg font-medium text-gray-900 mb-2">{"No projects yet"}</h3>
                    <p class="text-gray-600 mb-4">{"Create your first project to start analyzing logs"}</p>
                    <button
                        onclick={on_create_project.clone()}
                        class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                    >
                        {"Create Project"}
                    </button>
                </div>
            } else {
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    {for projects.iter().map(|project| {
                        html! {
                            <ProjectCard
                                project={project.clone()}
                                file_count={Some(5)}
                                analysis_count={Some(2)}
                                on_delete={Some(on_delete_project.clone())}
                            />
                        }
                    })}
                </div>
            }

            // Create Project Modal
            if *show_create_modal {
                <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
                    <div class="bg-white rounded-lg shadow-xl w-full max-w-md">
                        <form onsubmit={on_submit_create}>
                            <div class="p-6">
                                <div class="flex items-center justify-between mb-4">
                                    <h2 class="text-xl font-semibold text-gray-900">{"Create New Project"}</h2>
                                    <button
                                        type="button"
                                        onclick={on_close_modal.clone()}
                                        class="text-gray-400 hover:text-gray-600"
                                    >
                                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                                        </svg>
                                    </button>
                                </div>

                                <div class="space-y-4">
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">
                                            {"Project Name"}
                                        </label>
                                        <input
                                            type="text"
                                            value={(*new_project_name).clone()}
                                            onchange={on_name_change}
                                            placeholder="Enter project name"
                                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                            required=true
                                        />
                                    </div>

                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">
                                            {"Description (Optional)"}
                                        </label>
                                        <textarea
                                            value={(*new_project_description).clone()}
                                            onchange={on_description_change}
                                            placeholder="Enter project description"
                                            rows="3"
                                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                        />
                                    </div>
                                </div>
                            </div>

                            <div class="px-6 py-4 bg-gray-50 rounded-b-lg flex justify-end space-x-3">
                                <button
                                    type="button"
                                    onclick={on_close_modal}
                                    class="px-4 py-2 text-gray-700 border border-gray-300 rounded-md hover:bg-gray-50 transition-colors"
                                >
                                    {"Cancel"}
                                </button>
                                <button
                                    type="submit"
                                    disabled={*creating || new_project_name.is_empty()}
                                    class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                                >
                                    if *creating {
                                        {"Creating..."}
                                    } else {
                                        {"Create Project"}
                                    }
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            }
        </div>
    }
}
