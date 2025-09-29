use yew::prelude::*;
// use wasm_bindgen_futures::spawn_local;
// use crate::services::api::ApiService;
use crate::components::{AnalysisCard, ProjectCard};
use crate::types::{Analysis, Project};

#[function_component(Dashboard)]
pub fn dashboard() -> Html {
    let recent_projects = use_state(|| Vec::<Project>::new());
    let recent_analyses = use_state(|| Vec::<Analysis>::new());
    let loading = use_state(|| true);

    // Load dashboard data on component mount
    // TODO: Add data loading effect back when needed
    let recent_projects = use_state(|| Vec::<Project>::new());
    let recent_analyses = use_state(|| Vec::<Analysis>::new());
    let loading = use_state(|| true);

    // Callback for viewing analysis details
    let on_view_analysis = Callback::from(|analysis_id: String| {
        // TODO: Navigate to analysis detail page
        web_sys::console::log_1(&format!("View analysis: {}", analysis_id).into());
    });

    if *loading {
        return html! {
            <div class="flex items-center justify-center min-h-screen">
                <div class="animate-spin rounded-full h-32 w-32 border-b-2 border-blue-600"></div>
            </div>
        };
    }

    html! {
        <div class="space-y-8">
            // Header
            <div class="bg-gradient-to-r from-blue-600 to-purple-600 rounded-lg p-8 text-white">
                <h1 class="text-3xl font-bold mb-2">{"LogLens Dashboard"}</h1>
                <p class="text-blue-100">{"Intelligent log analysis and error detection"}</p>
            </div>

            // Quick Stats
            <div class="grid grid-cols-1 md:grid-cols-4 gap-6">
                <div class="bg-white rounded-lg shadow p-6">
                    <div class="flex items-center">
                        <div class="p-3 rounded-full bg-blue-100 text-blue-600">
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
                            </svg>
                        </div>
                        <div class="ml-4">
                            <p class="text-sm font-medium text-gray-600">{"Active Projects"}</p>
                            <p class="text-2xl font-semibold text-gray-900">{recent_projects.len()}</p>
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
                            <p class="text-sm font-medium text-gray-600">{"Total Analyses"}</p>
                            <p class="text-2xl font-semibold text-gray-900">{recent_analyses.len()}</p>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow p-6">
                    <div class="flex items-center">
                        <div class="p-3 rounded-full bg-yellow-100 text-yellow-600">
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                            </svg>
                        </div>
                        <div class="ml-4">
                            <p class="text-sm font-medium text-gray-600">{"Errors Found"}</p>
                            <p class="text-2xl font-semibold text-gray-900">{"127"}</p>
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
                            <p class="text-sm font-medium text-gray-600">{"AI Insights"}</p>
                            <p class="text-2xl font-semibold text-gray-900">{"43"}</p>
                        </div>
                    </div>
                </div>
            </div>

            // Recent Projects and Analyses
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
                // Recent Projects
                <div>
                    <div class="flex items-center justify-between mb-6">
                        <h2 class="text-xl font-semibold text-gray-900">{"Recent Projects"}</h2>
                        <a href="/projects" class="text-blue-600 hover:text-blue-800 text-sm font-medium">
                            {"View All →"}
                        </a>
                    </div>
                    <div class="space-y-4">
                        {for recent_projects.iter().take(3).map(|project| {
                            html! {
                                <ProjectCard
                                    project={project.clone()}
                                    file_count={Some(5)}
                                    analysis_count={Some(2)}
                                    on_delete={None as Option<Callback<String>>}
                                />
                            }
                        })}
                    </div>
                </div>

                // Recent Analyses
                <div>
                    <div class="flex items-center justify-between mb-6">
                        <h2 class="text-xl font-semibold text-gray-900">{"Recent Analyses"}</h2>
                        <a href="/analyses" class="text-blue-600 hover:text-blue-800 text-sm font-medium">
                            {"View All →"}
                        </a>
                    </div>
                    <div class="space-y-4">
                        {for recent_analyses.iter().take(3).map(|analysis| {
                            html! {
                                <AnalysisCard
                                    analysis={analysis.clone()}
                                    on_view={on_view_analysis.clone()}
                                />
                            }
                        })}
                    </div>
                </div>
            </div>

            // Quick Actions
            <div class="bg-white rounded-lg shadow p-6">
                <h2 class="text-xl font-semibold text-gray-900 mb-4">{"Quick Actions"}</h2>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <a href="/projects/new" class="p-4 border-2 border-dashed border-gray-300 rounded-lg hover:border-blue-400 hover:bg-blue-50 transition-colors text-center">
                        <div class="text-gray-400 mb-2">
                            <svg class="w-8 h-8 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6"></path>
                            </svg>
                        </div>
                        <p class="text-sm font-medium text-gray-700">{"Create New Project"}</p>
                    </a>

                    <a href="/upload" class="p-4 border-2 border-dashed border-gray-300 rounded-lg hover:border-green-400 hover:bg-green-50 transition-colors text-center">
                        <div class="text-gray-400 mb-2">
                            <svg class="w-8 h-8 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"></path>
                            </svg>
                        </div>
                        <p class="text-sm font-medium text-gray-700">{"Upload Log Files"}</p>
                    </a>

                    <a href="/analyze" class="p-4 border-2 border-dashed border-gray-300 rounded-lg hover:border-purple-400 hover:bg-purple-50 transition-colors text-center">
                        <div class="text-gray-400 mb-2">
                            <svg class="w-8 h-8 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"></path>
                            </svg>
                        </div>
                        <p class="text-sm font-medium text-gray-700">{"Start Analysis"}</p>
                    </a>
                </div>
            </div>
        </div>
    }
}