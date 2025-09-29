use yew::prelude::*;
use web_sys::HtmlInputElement;
use wasm_bindgen_futures::spawn_local;

use crate::services::api::ApiService;
use crate::types::{AnalysisRequest, Settings, ModelInfo, ModelsRequest};

#[derive(Properties, PartialEq)]
pub struct AnalysisRequestFormProps {
    pub project_id: String,
    pub file_id: String,
    pub on_submit: Callback<AnalysisRequest>,
    pub on_cancel: Callback<()>,
}

#[function_component(AnalysisRequestForm)]
pub fn analysis_request_form(props: &AnalysisRequestFormProps) -> Html {
    let provider = use_state(|| "openrouter".to_string());
    let level = use_state(|| "ERROR".to_string());
    let user_context = use_state(|| String::new());
    let selected_model = use_state(|| None::<String>);
    let timeout_seconds = use_state(|| None::<u32>);

    let available_models = use_state(|| Vec::<ModelInfo>::new());
    let settings = use_state(|| None::<Settings>);
    let loading_models = use_state(|| false);

    // Load settings and models on mount
    {
        let provider = provider.clone();
        let settings = settings.clone();
        let available_models = available_models.clone();
        let loading_models = loading_models.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                // Load settings first
                if let Ok(settings_data) = ApiService::get_settings().await {
                    provider.set(settings_data.default_provider.clone());
                    settings.set(Some(settings_data.clone()));

                    // Load models for the default provider if API key exists
                    if !settings_data.api_key.is_empty() {
                        loading_models.set(true);
                        let models_request = ModelsRequest {
                            provider: settings_data.default_provider,
                            api_key: settings_data.api_key,
                            force_refresh: Some(false),
                        };

                        if let Ok(models_response) = ApiService::get_available_models(models_request).await {
                            available_models.set(models_response.models);
                        }
                        loading_models.set(false);
                    }
                }
            });
            || ()
        });
    }

    let on_provider_change = {
        let provider = provider.clone();
        let available_models = available_models.clone();
        let settings = settings.clone();
        let loading_models = loading_models.clone();

        move |e: Event| {
            let select: HtmlInputElement = e.target_unchecked_into();
            let new_provider = select.value();
            provider.set(new_provider.clone());

            // Load models for new provider
            if let Some(current_settings) = (*settings).clone() {
                if !current_settings.api_key.is_empty() {
                    loading_models.set(true);
                    let available_models = available_models.clone();
                    let loading_models = loading_models.clone();

                    spawn_local(async move {
                        let models_request = ModelsRequest {
                            provider: new_provider,
                            api_key: current_settings.api_key,
                            force_refresh: Some(false),
                        };

                        if let Ok(models_response) = ApiService::get_available_models(models_request).await {
                            available_models.set(models_response.models);
                        }
                        loading_models.set(false);
                    });
                }
            }
        }
    };

    let on_submit = {
        let provider = provider.clone();
        let level = level.clone();
        let user_context = user_context.clone();
        let selected_model = selected_model.clone();
        let timeout_seconds = timeout_seconds.clone();
        let on_submit = props.on_submit.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let request = AnalysisRequest {
                provider: (*provider).clone(),
                level: (*level).clone(),
                user_context: if user_context.trim().is_empty() {
                    None
                } else {
                    Some((*user_context).clone())
                },
                selected_model: (*selected_model).clone(),
                timeout_seconds: (*timeout_seconds).clone(),
            };

            on_submit.emit(request);
        })
    };

    html! {
        <div class="bg-white rounded-lg shadow p-6">
            <h3 class="text-lg font-semibold text-gray-900 mb-4">{"Configure Analysis"}</h3>

            <form onsubmit={on_submit}>
                <div class="space-y-4">
                    // Provider Selection
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                            {"AI Provider"}
                        </label>
                        <select
                            value={(*provider).clone()}
                            onchange={on_provider_change}
                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                        >
                            <option value="openrouter">{"OpenRouter"}</option>
                            <option value="openai">{"OpenAI"}</option>
                            <option value="claude">{"Anthropic Claude"}</option>
                            <option value="gemini">{"Google Gemini"}</option>
                        </select>
                    </div>

                    // Model Selection
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                            {"Model"}
                        </label>
                        <select
                            value={(*selected_model).clone().unwrap_or_default()}
                            onchange={{
                                let selected_model = selected_model.clone();
                                move |e: Event| {
                                    let select: HtmlInputElement = e.target_unchecked_into();
                                    selected_model.set(if select.value().is_empty() {
                                        None
                                    } else {
                                        Some(select.value())
                                    });
                                }
                            }}
                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                            disabled={*loading_models || available_models.is_empty()}
                        >
                            <option value="">{"Auto-select (recommended)"}</option>
                            {
                                available_models.iter().map(|model| {
                                    html! {
                                        <option key={model.id.clone()} value={model.id.clone()}>
                                            {format!("{} ({})", model.name, model.pricing_tier.clone().unwrap_or("unknown".to_string()))}
                                        </option>
                                    }
                                }).collect::<Html>()
                            }
                        </select>
                        {
                            if *loading_models {
                                html! {
                                    <p class="text-xs text-gray-500 mt-1">
                                        {"Loading available models..."}
                                    </p>
                                }
                            } else if available_models.is_empty() {
                                html! {
                                    <p class="text-xs text-gray-500 mt-1">
                                        {"Configure API key in Settings to load models"}
                                    </p>
                                }
                            } else {
                                html! {
                                    <p class="text-xs text-gray-500 mt-1">
                                        {format!("{} models available", available_models.len())}
                                    </p>
                                }
                            }
                        }
                    </div>

                    // Log Level
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                            {"Log Level Filter"}
                        </label>
                        <select
                            value={(*level).clone()}
                            onchange={{
                                let level = level.clone();
                                move |e: Event| {
                                    let select: HtmlInputElement = e.target_unchecked_into();
                                    level.set(select.value());
                                }
                            }}
                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                        >
                            <option value="ERROR">{"ERROR - Critical errors only"}</option>
                            <option value="WARN">{"WARN - Warnings and errors"}</option>
                            <option value="INFO">{"INFO - General information and above"}</option>
                            <option value="DEBUG">{"DEBUG - All log levels"}</option>
                        </select>
                    </div>

                    // User Context
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                            {"Analysis Context"} <span class="text-gray-400 text-xs">{"(Optional)"}</span>
                        </label>
                        <textarea
                            value={(*user_context).clone()}
                            onchange={{
                                let user_context = user_context.clone();
                                move |e: Event| {
                                    let textarea: HtmlInputElement = e.target_unchecked_into();
                                    user_context.set(textarea.value());
                                }
                            }}
                            placeholder="Describe what you're looking for or any specific context about these logs..."
                            rows="3"
                            maxlength="2000"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
                        />
                        <p class="text-xs text-gray-500 mt-1">
                            {format!("{}/2000 characters", user_context.len())}
                        </p>
                    </div>

                    // Timeout Configuration
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                            {"Analysis Timeout"} <span class="text-gray-400 text-xs">{"(Optional)"}</span>
                        </label>
                        <div class="flex items-center space-x-2">
                            <input
                                type="number"
                                value={timeout_seconds.clone().unwrap_or_default().to_string()}
                                onchange={{
                                    let timeout_seconds = timeout_seconds.clone();
                                    move |e: Event| {
                                        let input: HtmlInputElement = e.target_unchecked_into();
                                        if input.value().is_empty() {
                                            timeout_seconds.set(None);
                                        } else if let Ok(value) = input.value().parse::<u32>() {
                                            if value >= 60 && value <= 1800 {
                                                timeout_seconds.set(Some(value));
                                            }
                                        }
                                    }
                                }}
                                min="60"
                                max="1800"
                                placeholder="300"
                                class="flex-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                            />
                            <span class="text-sm text-gray-500">{"seconds"}</span>
                        </div>
                        <p class="text-xs text-gray-500 mt-1">
                            {"Leave empty to use default timeout from settings (60-1800 seconds)"}
                        </p>
                    </div>
                </div>

                // Action Buttons
                <div class="flex justify-end space-x-3 mt-6 pt-4 border-t border-gray-200">
                    <button
                        type="button"
                        onclick={{
                            let on_cancel = props.on_cancel.clone();
                            move |_| on_cancel.emit(())
                        }}
                        class="px-4 py-2 text-gray-700 border border-gray-300 rounded-md hover:bg-gray-50 transition-colors"
                    >
                        {"Cancel"}
                    </button>
                    <button
                        type="submit"
                        class="px-6 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors font-medium"
                    >
                        {"Start Analysis"}
                    </button>
                </div>
            </form>
        </div>
    }
}