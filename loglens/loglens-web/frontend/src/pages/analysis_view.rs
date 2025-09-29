use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen_futures::spawn_local;
use gloo_net::http::Request;
use crate::router::Route;
use crate::types::{Analysis, AnalysisStatus};

#[derive(Properties, PartialEq)]
pub struct AnalysisViewProps {
    pub project_id: String,
    pub analysis_id: String,
}

#[function_component(AnalysisView)]
pub fn analysis_view(props: &AnalysisViewProps) -> Html {
    let project_id = props.project_id.clone();
    let analysis_id = props.analysis_id.clone();
    
    let analysis = use_state(|| None::<Analysis>);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let raw_result = use_state(|| None::<String>);
    
    let navigator = use_navigator();

    // Load analysis data
    {
        let analysis = analysis.clone();
        let loading = loading.clone();
        let error = error.clone();
        let raw_result = raw_result.clone();
        let analysis_id = analysis_id.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                loading.set(true);
                error.set(None);
                
                // Fetch analysis data from API
                let url = format!("/api/analyses/{}", analysis_id);
                match Request::get(&url).send().await {
                    Ok(response) => {
                        if response.ok() {
                            match response.json::<Analysis>().await {
                                Ok(analysis_data) => {
                                    // Extract and parse the result if available
                                    if let Some(result) = &analysis_data.result {
                                        raw_result.set(Some(result.clone()));
                                    }
                                    analysis.set(Some(analysis_data));
                                }
                                Err(e) => {
                                    error.set(Some(format!("Failed to parse analysis data: {}", e)));
                                }
                            }
                        } else {
                            error.set(Some(format!("Failed to fetch analysis: {}", response.status())));
                        }
                    }
                    Err(e) => {
                        error.set(Some(format!("Network error: {}", e)));
                    }
                }
                loading.set(false);
            });
        });
    }

    let on_back = {
        let navigator = navigator.clone();
        move |_| {
            if let Some(navigator) = &navigator {
                navigator.push(&Route::ProjectDetail { id: project_id.clone() });
            }
        }
    };

    let on_export_html = {
        let analysis_id = analysis_id.clone();
        move |_| {
            // Trigger HTML export download
            let url = format!("/api/analyses/{}/export/html", analysis_id);
            if let Some(window) = web_sys::window() {
                let _ = window.open_with_url(&url);
            }
        }
    };

    let on_export_json = {
        let analysis_id = analysis_id.clone();
        move |_| {
            // Trigger JSON export download
            let url = format!("/api/analyses/{}/export/json", analysis_id);
            if let Some(window) = web_sys::window() {
                let _ = window.open_with_url(&url);
            }
        }
    };

    if *loading {
        return html! {
            <div class="flex items-center justify-center min-h-screen">
                <div class="text-center">
                    <div class="animate-spin rounded-full h-32 w-32 border-b-2 border-blue-600 mx-auto"></div>
                    <p class="mt-4 text-gray-600">{"Loading analysis..."}</p>
                </div>
            </div>
        };
    }

    if let Some(error_msg) = error.as_ref() {
        return html! {
            <div class="min-h-screen flex items-center justify-center">
                <div class="text-center">
                    <div class="text-red-500 mb-4">
                        <svg class="w-16 h-16 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                        </svg>
                    </div>
                    <h3 class="text-lg font-medium text-gray-900 mb-2">{"Failed to load analysis"}</h3>
                    <p class="text-gray-600 mb-4">{error_msg}</p>
                    <button
                        onclick={on_back.clone()}
                        class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                    >
                        {"← Back to Project"}
                    </button>
                </div>
            </div>
        };
    }

    let analysis_data = analysis.as_ref().unwrap();

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
                        <h1 class="text-2xl font-bold text-gray-900">{"Analysis Results"}</h1>
                        <p class="text-gray-600">{format!("Analysis ID: {}", analysis_id)}</p>
                    </div>
                </div>
                <div class="flex space-x-3">
                    <button
                        onclick={on_export_html}
                        class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors font-medium"
                    >
                        {"📄 Export HTML"}
                    </button>
                    <button
                        onclick={on_export_json}
                        class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-medium"
                    >
                        {"📊 Export JSON"}
                    </button>
                </div>
            </div>

            // Analysis Info
            <div class="bg-white rounded-lg shadow p-6">
                <div class="grid grid-cols-1 md:grid-cols-4 gap-6">
                    <div>
                        <label class="text-sm font-medium text-gray-500">{"Status"}</label>
                        <div class="mt-1">
                            {render_status_badge(&analysis_data.status)}
                        </div>
                    </div>
                    <div>
                        <label class="text-sm font-medium text-gray-500">{"Provider"}</label>
                        <p class="mt-1 text-sm text-gray-900">{&analysis_data.provider}</p>
                    </div>
                    <div>
                        <label class="text-sm font-medium text-gray-500">{"Level Filter"}</label>
                        <p class="mt-1 text-sm text-gray-900">{&analysis_data.level_filter}</p>
                    </div>
                    <div>
                        <label class="text-sm font-medium text-gray-500">{"Started"}</label>
                        <p class="mt-1 text-sm text-gray-900">{format_timestamp(&analysis_data.started_at)}</p>
                    </div>
                </div>
                
                if let Some(completed_at) = &analysis_data.completed_at {
                    <div class="mt-4">
                        <label class="text-sm font-medium text-gray-500">{"Completed"}</label>
                        <p class="mt-1 text-sm text-gray-900">{format_timestamp(completed_at)}</p>
                    </div>
                }
                
                if let Some(error_msg) = &analysis_data.error_message {
                    <div class="mt-4">
                        <label class="text-sm font-medium text-red-500">{"Error"}</label>
                        <p class="mt-1 text-sm text-red-600 bg-red-50 p-3 rounded-md">{error_msg}</p>
                    </div>
                }
            </div>

            // Analysis Results
            if let Some(result) = raw_result.as_ref() {
                <div class="bg-white rounded-lg shadow">
                    <div class="px-6 py-4 border-b border-gray-200">
                        <h2 class="text-lg font-semibold text-gray-900">{"Analysis Results"}</h2>
                    </div>
                    <div class="p-6">
                        {render_analysis_result(result)}
                    </div>
                </div>
            } else if analysis_data.status == AnalysisStatus::Completed {
                <div class="bg-white rounded-lg shadow p-6">
                    <div class="text-center py-8">
                        <div class="text-gray-400 mb-4">
                            <svg class="w-16 h-16 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
                            </svg>
                        </div>
                        <h3 class="text-lg font-medium text-gray-900 mb-2">{"No Results Available"}</h3>
                        <p class="text-gray-600">{"Analysis completed but no results were generated."}</p>
                    </div>
                </div>
            } else if analysis_data.status == AnalysisStatus::Running {
                <div class="bg-white rounded-lg shadow p-6">
                    <div class="text-center py-8">
                        <div class="animate-spin rounded-full h-16 w-16 border-b-2 border-blue-600 mx-auto mb-4"></div>
                        <h3 class="text-lg font-medium text-gray-900 mb-2">{"Analysis in Progress"}</h3>
                        <p class="text-gray-600">{"Please wait while the analysis is being processed..."}</p>
                    </div>
                </div>
            }
        </div>
    }
}

fn render_status_badge(status: &AnalysisStatus) -> Html {
    let (color_class, text) = match status {
        AnalysisStatus::Pending => ("bg-yellow-100 text-yellow-800", "Pending"),
        AnalysisStatus::Running => ("bg-blue-100 text-blue-800", "Running"),
        AnalysisStatus::Completed => ("bg-green-100 text-green-800", "Completed"),
        AnalysisStatus::Failed => ("bg-red-100 text-red-800", "Failed"),
    };

    html! {
        <span class={format!("inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {}", color_class)}>
            {text}
        </span>
    }
}

fn format_timestamp(timestamp: &chrono::DateTime<chrono::Utc>) -> String {
    timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

fn render_analysis_result(result: &str) -> Html {
    // Try to parse as JSON for structured display
    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(result) {
        render_json_result(&json_value)
    } else {
        // Fallback to plain text with basic formatting
        render_text_result(result)
    }
}

fn render_json_result(json: &serde_json::Value) -> Html {
    match json {
        serde_json::Value::Object(map) => {
            html! {
                <div class="space-y-6">
                    {for map.iter().map(|(key, value)| {
                        html! {
                            <div class="border-l-4 border-blue-500 pl-4">
                                <h3 class="text-lg font-semibold text-gray-900 mb-2">{key}</h3>
                                <div class="text-gray-700">
                                    {render_json_value(value)}
                                </div>
                            </div>
                        }
                    })}
                </div>
            }
        }
        _ => render_json_value(json)
    }
}

fn render_json_value(value: &serde_json::Value) -> Html {
    match value {
        serde_json::Value::String(s) => html! { <p class="whitespace-pre-wrap">{s}</p> },
        serde_json::Value::Array(arr) => {
            html! {
                <ul class="list-disc list-inside space-y-1">
                    {for arr.iter().map(|item| {
                        html! { <li>{render_json_value(item)}</li> }
                    })}
                </ul>
            }
        }
        serde_json::Value::Object(obj) => {
            html! {
                <div class="space-y-2">
                    {for obj.iter().map(|(key, val)| {
                        html! {
                            <div>
                                <span class="font-medium text-gray-800">{format!("{}: ", key)}</span>
                                {render_json_value(val)}
                            </div>
                        }
                    })}
                </div>
            }
        }
        _ => html! { <span>{value.to_string()}</span> }
    }
}

fn render_text_result(text: &str) -> Html {
    html! {
        <div class="prose max-w-none">
            <pre class="whitespace-pre-wrap bg-gray-50 p-4 rounded-lg text-sm text-gray-800 overflow-x-auto">
                {text}
            </pre>
        </div>
    }
}