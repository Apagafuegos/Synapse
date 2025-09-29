use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::router::Route;
use crate::services::api::ApiService;
use crate::components::{FileUpload, AnalysisCard, AnalysisRequestForm};
use crate::types::{Analysis, Project, AnalysisStatus, AnalysisRequest};

#[derive(Properties, PartialEq)]
pub struct ProjectDetailProps {
    pub id: String,
}

#[function_component(ProjectDetail)]
pub fn project_detail(props: &ProjectDetailProps) -> Html {
    let project_id = props.id.clone();
    let project = use_state(|| None::<Project>);
    let analyses = use_state(|| Vec::<Analysis>::new());
    let loading = use_state(|| true);
    let show_upload = use_state(|| false);
    let show_analysis_form = use_state(|| false);

    let api = ApiService::new();
    let navigator = use_navigator();

    // Load project data on component mount
    {
        let project = project.clone();
        let analyses = analyses.clone();
        let loading = loading.clone();
        let _api = api.clone();
        let project_id_for_effect = project_id.clone();

        use_effect(move || {
            spawn_local(async move {
                // Simulate API calls - replace with actual API calls when backend is ready
                let project_data = Project {
                    id: project_id_for_effect.clone(),
                    name: "Web Server Logs".to_string(),
                    description: Some("Production web server error analysis".to_string()),
                    created_at: chrono::Utc::now() - chrono::Duration::days(2),
                    updated_at: chrono::Utc::now() - chrono::Duration::hours(6),
                };

                let analyses_data = vec![
                    Analysis {
                        id: "analysis-001-completed".to_string(),
                        project_id: project_id_for_effect.clone(),
                        log_file_id: Some("logfile-001".to_string()),
                        analysis_type: "error_analysis".to_string(),
                        provider: "openrouter".to_string(),
                        level_filter: "ERROR".to_string(),
                        status: AnalysisStatus::Completed,
                        result: Some(r#"{"summary": "Found 23 critical errors", "error_patterns": ["Database connection timeout", "Memory leak in user session handler"]}"#.to_string()),
                        error_message: None,
                        started_at: chrono::Utc::now() - chrono::Duration::hours(2),
                        completed_at: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
                    },
                    Analysis {
                        id: "analysis-002-running".to_string(),
                        project_id: project_id_for_effect.clone(),
                        log_file_id: Some("logfile-002".to_string()),
                        analysis_type: "performance_analysis".to_string(),
                        provider: "claude".to_string(),
                        level_filter: "WARN".to_string(),
                        status: AnalysisStatus::Running,
                        result: None,
                        error_message: None,
                        started_at: chrono::Utc::now() - chrono::Duration::minutes(30),
                        completed_at: None,
                    },
                    Analysis {
                        id: "analysis-003-failed".to_string(),
                        project_id: project_id_for_effect.clone(),
                        log_file_id: Some("logfile-003".to_string()),
                        analysis_type: "anomaly_detection".to_string(),
                        provider: "openai".to_string(),
                        level_filter: "INFO".to_string(),
                        status: AnalysisStatus::Failed,
                        result: None,
                        error_message: Some("Invalid log format detected - unable to parse timestamps".to_string()),
                        started_at: chrono::Utc::now() - chrono::Duration::hours(4),
                        completed_at: Some(chrono::Utc::now() - chrono::Duration::hours(3)),
                    },
                ];

                project.set(Some(project_data));
                analyses.set(analyses_data);
                loading.set(false);
            });
        });
    }

    let on_back = {
        let navigator = navigator.clone();
        move |_| {
            if let Some(navigator) = &navigator {
                navigator.push(&Route::Projects);
            }
        }
    };

    let on_upload_success = {
        let show_upload = show_upload.clone();
        move |_| {
            show_upload.set(false);
            web_sys::console::log_1(&"File uploaded successfully".into());
            // TODO: Refresh analyses list
        }
    };

    let on_upload_error = {
        move |error: String| {
            web_sys::console::error_1(&format!("File upload error: {}", error).into());
            // TODO: Show error message to user
        }
    };

    let on_view_analysis = {
        let navigator = navigator.clone();
        let project_id = project_id.clone();
        Callback::from(move |analysis_id: String| {
            if let Some(navigator) = &navigator {
                navigator.push(&Route::AnalysisView { 
                    project_id: project_id.clone(),
                    analysis_id 
                });
            }
        })
    };

    let on_show_analysis_form = {
        let show_analysis_form = show_analysis_form.clone();
        move |_| show_analysis_form.set(true)
    };

    let on_cancel_analysis = {
        let show_analysis_form = show_analysis_form.clone();
        move |_| show_analysis_form.set(false)
    };

    let on_submit_analysis = {
        let analyses = analyses.clone();
        let project_id = project_id.clone();
        let show_analysis_form = show_analysis_form.clone();
        move |request: AnalysisRequest| {
            show_analysis_form.set(false);

            // Create new analysis with running status
            let analysis_id = format!("analysis-{:03}-new", analyses.len() + 1);
            let new_analysis = Analysis {
                id: analysis_id.clone(),
                project_id: project_id.clone(),
                log_file_id: None, // TODO: Get from selected file
                analysis_type: "ai_analysis".to_string(),
                provider: request.provider,
                level_filter: request.level,
                status: AnalysisStatus::Running,
                result: None,
                error_message: None,
                started_at: chrono::Utc::now(),
                completed_at: None,
            };

            let mut updated_analyses = (*analyses).clone();
            updated_analyses.insert(0, new_analysis);
            analyses.set(updated_analyses);

            // TODO: Start actual analysis via API
            let analyses_for_async = analyses.clone();
            spawn_local(async move {
                // Simulate API call
                gloo::timers::future::TimeoutFuture::new(3_000).await;

                // Update analysis to completed
                let mut current_analyses = (*analyses_for_async).clone();
                if let Some(analysis) = current_analyses.iter_mut().find(|a| a.id == analysis_id) {
                    analysis.status = AnalysisStatus::Completed;
                    analysis.completed_at = Some(chrono::Utc::now());
                    analysis.result = Some(r#"{"summary": "Analysis completed successfully", "findings": ["Sample finding 1", "Sample finding 2"]}"#.to_string());
                }
                analyses_for_async.set(current_analyses);
            });
        }
    };

    if *loading {
        return html! {
            <div class="flex items-center justify-center min-h-screen">
                <div class="animate-spin rounded-full h-32 w-32 border-b-2 border-blue-600"></div>
            </div>
        };
    }

    let project_data = project.as_ref().unwrap();

    html! {
        <div class="space-y-6">
            // Header
            <div class="flex items-center justify-between">
                <div class="flex items-center space-x-4">
                    <button
                        onclick={on_back}
                        class="p-2 text-gray-600 hover:text-gray-900 transition-colors"
                    >
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"></path>
                        </svg>
                    </button>
                    <div>
                        <h1 class="text-2xl font-bold text-gray-900">{&project_data.name}</h1>
                        if let Some(desc) = &project_data.description {
                            <p class="text-gray-600">{desc}</p>
                        }
                    </div>
                </div>
                <div class="flex space-x-3">
                    <button
                        onclick={{
                            let show_upload = show_upload.clone();
                            move |_| show_upload.set(true)
                        }}
                        class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors font-medium"
                    >
                        {"📁 Upload Files"}
                    </button>
                    <button
                        onclick={on_show_analysis_form.clone()}
                        class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-medium"
                    >
                        {"🔍 Start Analysis"}
                    </button>
                </div>
            </div>

            // Project Stats
            <div class="grid grid-cols-1 md:grid-cols-4 gap-6">
                <div class="bg-white rounded-lg shadow p-6">
                    <div class="flex items-center">
                        <div class="p-3 rounded-full bg-blue-100 text-blue-600">
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
                            </svg>
                        </div>
                        <div class="ml-4">
                            <p class="text-sm font-medium text-gray-600">{"Log Files"}</p>
                            <p class="text-2xl font-semibold text-gray-900">{"12"}</p>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow p-6">
                    <div class="flex items-center">
                        <div class="p-3 rounded-full bg-green-100 text-green-600">
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"></path>
                            </svg>
                        </div>
                        <div class="ml-4">
                            <p class="text-sm font-medium text-gray-600">{"Analyses"}</p>
                            <p class="text-2xl font-semibold text-gray-900">{analyses.len()}</p>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow p-6">
                    <div class="flex items-center">
                        <div class="p-3 rounded-full bg-red-100 text-red-600">
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                            </svg>
                        </div>
                        <div class="ml-4">
                            <p class="text-sm font-medium text-gray-600">{"Errors Found"}</p>
                            <p class="text-2xl font-semibold text-gray-900">{"87"}</p>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow p-6">
                    <div class="flex items-center">
                        <div class="p-3 rounded-full bg-purple-100 text-purple-600">
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"></path>
                            </svg>
                        </div>
                        <div class="ml-4">
                            <p class="text-sm font-medium text-gray-600">{"Last Analysis"}</p>
                            <p class="text-sm font-semibold text-gray-900">{"2 hours ago"}</p>
                        </div>
                    </div>
                </div>
            </div>

            // Analyses Section
            <div class="bg-white rounded-lg shadow">
                <div class="px-6 py-4 border-b border-gray-200">
                    <div class="flex items-center justify-between">
                        <h2 class="text-lg font-semibold text-gray-900">{"Analysis History"}</h2>
                        <span class="text-sm text-gray-500">{format!("{} total", analyses.len())}</span>
                    </div>
                </div>
                <div class="p-6">
                    if analyses.is_empty() {
                        <div class="text-center py-8">
                            <div class="text-gray-400 mb-4">
                                <svg class="w-16 h-16 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"></path>
                                </svg>
                            </div>
                            <h3 class="text-lg font-medium text-gray-900 mb-2">{"No analyses yet"}</h3>
                            <p class="text-gray-600 mb-4">{"Start your first analysis to see results here"}</p>
                            <button
                                onclick={on_show_analysis_form.clone()}
                                class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                            >
                                {"Start Analysis"}
                            </button>
                        </div>
                    } else {
                        <div class="space-y-4">
                            {for analyses.iter().map(|analysis| {
                                html! {
                                    <AnalysisCard
                                        analysis={analysis.clone()}
                                        on_view={on_view_analysis.clone()}
                                    />
                                }
                            })}
                        </div>
                    }
                </div>
            </div>

            // File Upload Modal
            if *show_upload {
                <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
                    <div class="bg-white rounded-lg shadow-xl w-full max-w-2xl max-h-[90vh] overflow-y-auto">
                        <div class="p-6">
                            <div class="flex items-center justify-between mb-4">
                                <h2 class="text-xl font-semibold text-gray-900">{"Upload Log Files"}</h2>
                                <button
                                    onclick={move |_| show_upload.set(false)}
                                    class="text-gray-400 hover:text-gray-600"
                                >
                                    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                                    </svg>
                                </button>
                            </div>
                            <FileUpload
                                project_id={project_id.clone()}
                                on_upload={on_upload_success}
                                on_error={on_upload_error}
                            />
                        </div>
                    </div>
                </div>
            }

            // Analysis Request Form Modal
            if *show_analysis_form {
                <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
                    <div class="bg-white rounded-lg shadow-xl w-full max-w-3xl max-h-[90vh] overflow-y-auto">
                        <div class="p-6">
                            <div class="flex items-center justify-between mb-4">
                                <h2 class="text-xl font-semibold text-gray-900">{"Start Analysis"}</h2>
                                <button
                                    onclick={move |_| show_analysis_form.set(false)}
                                    class="text-gray-400 hover:text-gray-600"
                                >
                                    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                                    </svg>
                                </button>
                            </div>
                            <AnalysisRequestForm
                                project_id={project_id.clone()}
                                file_id={"dummy_file_id".to_string()} // TODO: Get from selected file
                                on_submit={on_submit_analysis}
                                on_cancel={on_cancel_analysis}
                            />
                        </div>
                    </div>
                </div>
            }
        </div>
    }
}