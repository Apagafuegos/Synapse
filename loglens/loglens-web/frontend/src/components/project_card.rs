use yew::prelude::*;
use yew_router::prelude::*;
use crate::router::Route;
use crate::types::Project;
use std::rc::Rc;

#[derive(Properties, PartialEq)]
pub struct ProjectCardProps {
    pub project: Project,
    pub file_count: Option<u32>,
    pub analysis_count: Option<u32>,
    pub on_delete: Option<Callback<String>>,
}

#[function_component(ProjectCard)]
pub fn project_card(props: &ProjectCardProps) -> Html {
    let project = &props.project;
    let navigator = use_navigator().unwrap();

    let project_id = Rc::new(project.id.clone());
    let on_click = {
        let navigator = navigator.clone();
        let project_id = project_id.clone();
        Callback::from(move |_: MouseEvent| {
            navigator.push(&Route::ProjectDetail { id: (*project_id).clone() });
        })
    };

    let on_delete = props.on_delete.clone();
    let delete_project_id = Rc::new(project.id.clone());
    let on_delete_click = {
        let on_delete = on_delete.clone();
        let delete_project_id = delete_project_id.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            if let Some(callback) = on_delete.as_ref() {
                callback.emit((*delete_project_id).clone());
            }
        })
    };

    html! {
        <div
            class="bg-white rounded-lg shadow-md p-6 hover:shadow-lg transition-all cursor-pointer transform hover:scale-105"
            onclick={on_click}
        >
            <div class="flex items-start justify-between mb-4">
                <div class="flex-1">
                    <h3 class="text-xl font-semibold text-gray-900 mb-2">
                        {&project.name}
                    </h3>
                    if let Some(desc) = &project.description {
                        <p class="text-gray-600 text-sm line-clamp-2">
                            {desc}
                        </p>
                    }
                </div>
                if props.on_delete.is_some() {
                    <button
                        onclick={on_delete_click}
                        class="text-gray-400 hover:text-red-600 transition-colors p-1"
                        title="Delete project"
                    >
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path>
                        </svg>
                    </button>
                }
            </div>

            <div class="flex items-center justify-between text-sm text-gray-500 mb-4">
                <span>{"Created "}{project.created_at.format("%Y-%m-%d")}</span>
                <span>{"Updated "}{project.updated_at.format("%Y-%m-%d")}</span>
            </div>

            <div class="grid grid-cols-2 gap-4 mb-4">
                <div class="text-center p-3 bg-blue-50 rounded-lg">
                    <div class="text-2xl font-bold text-blue-600">
                        {props.file_count.unwrap_or(0)}
                    </div>
                    <div class="text-xs text-blue-800 uppercase tracking-wide">
                        {"Log Files"}
                    </div>
                </div>
                <div class="text-center p-3 bg-green-50 rounded-lg">
                    <div class="text-2xl font-bold text-green-600">
                        {props.analysis_count.unwrap_or(0)}
                    </div>
                    <div class="text-xs text-green-800 uppercase tracking-wide">
                        {"Analyses"}
                    </div>
                </div>
            </div>

            <div class="flex items-center justify-between">
                <div class="flex space-x-2">
                    <span class="px-2 py-1 bg-gray-100 text-gray-700 rounded-full text-xs">
                        {"Active"}
                    </span>
                </div>
                <div class="text-blue-600 hover:text-blue-800 font-medium text-sm">
                    {"View Details →"}
                </div>
            </div>
        </div>
    }
}