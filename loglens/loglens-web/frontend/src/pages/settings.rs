use yew::prelude::*;
use web_sys::HtmlInputElement;
use wasm_bindgen_futures::spawn_local;

use crate::services::api::ApiService;
use crate::types::{Settings, ModelInfo, ModelsRequest};

#[function_component(SettingsPage)]
pub fn settings_page() -> Html {
    let settings = use_state(|| None::<Settings>);
    let available_models = use_state(|| Vec::<ModelInfo>::new());
    let loading = use_state(|| true);
    let saving = use_state(|| false);
    let saved = use_state(|| false);
    let error = use_state(|| None::<String>);

    // Load settings on mount
    {
        let settings = settings.clone();
        let loading = loading.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                match ApiService::get_settings().await {
                    Ok(settings_data) => {
                        settings.set(Some(settings_data));
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load settings: {:?}", e)));
                        loading.set(false);
                    }
                }
            });
            || ()
        });
    }

    let on_provider_change = {
        let settings = settings.clone();
        let available_models = available_models.clone();

        move |e: Event| {
            let select: HtmlInputElement = e.target_unchecked_into();
            let new_provider = select.value();

            if let Some(current_settings) = (*settings).clone() {
                let mut updated_settings = current_settings;
                updated_settings.default_provider = new_provider.clone();
                updated_settings.selected_model = None; // Reset model when provider changes
                settings.set(Some(updated_settings.clone()));

                // Load models for new provider
                let available_models = available_models.clone();
                let api_key = updated_settings.api_key.clone();

                if !api_key.is_empty() {
                    spawn_local(async move {
                        let models_request = ModelsRequest {
                            provider: new_provider,
                            api_key,
                            force_refresh: Some(false),
                        };

                        match ApiService::get_available_models(models_request).await {
                            Ok(response) => {
                                available_models.set(response.models);
                            }
                            Err(_) => {
                                // Ignore errors for model loading
                                available_models.set(Vec::new());
                            }
                        }
                    });
                }
            }
        }
    };

    let on_api_key_change = {
        let settings = settings.clone();

        move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let new_api_key = input.value();

            if let Some(current_settings) = (*settings).clone() {
                let mut updated_settings = current_settings;
                updated_settings.api_key = new_api_key;
                settings.set(Some(updated_settings));
            }
        }
    };

    let on_model_change = {
        let settings = settings.clone();

        move |e: Event| {
            let select: HtmlInputElement = e.target_unchecked_into();
            let new_model = if select.value().is_empty() {
                None
            } else {
                Some(select.value())
            };

            if let Some(current_settings) = (*settings).clone() {
                let mut updated_settings = current_settings;
                updated_settings.selected_model = new_model;
                settings.set(Some(updated_settings));
            }
        }
    };

    let on_timeout_change = {
        let settings = settings.clone();

        move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let timeout_value = input.value().parse::<i32>().unwrap_or(300);

            if let Some(current_settings) = (*settings).clone() {
                let mut updated_settings = current_settings;
                updated_settings.analysis_timeout_seconds = Some(timeout_value);
                settings.set(Some(updated_settings));
            }
        }
    };

    let on_max_lines_change = {
        let settings = settings.clone();

        move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let max_lines_value = input.value().parse::<u32>().unwrap_or(1000);

            if let Some(current_settings) = (*settings).clone() {
                let mut updated_settings = current_settings;
                updated_settings.max_lines = max_lines_value;
                settings.set(Some(updated_settings));
            }
        }
    };

    let load_models = {
        let settings = settings.clone();
        let available_models = available_models.clone();

        Callback::from(move |_| {
            if let Some(current_settings) = (*settings).clone() {
                if !current_settings.api_key.is_empty() {
                    let available_models = available_models.clone();
                    let provider = current_settings.default_provider.clone();
                    let api_key = current_settings.api_key.clone();

                    spawn_local(async move {
                        let models_request = ModelsRequest {
                            provider,
                            api_key,
                            force_refresh: Some(true),
                        };

                        match ApiService::get_available_models(models_request).await {
                            Ok(response) => {
                                available_models.set(response.models);
                            }
                            Err(_) => {
                                available_models.set(Vec::new());
                            }
                        }
                    });
                }
            }
        })
    };

    let on_save = {
        let settings = settings.clone();
        let saving = saving.clone();
        let saved = saved.clone();
        let error = error.clone();

        move |e: SubmitEvent| {
            e.prevent_default();

            if let Some(current_settings) = (*settings).clone() {
                saving.set(true);
                let settings = settings.clone();
                let saving = saving.clone();
                let saved = saved.clone();
                let error = error.clone();

                spawn_local(async move {
                    match ApiService::update_settings(current_settings).await {
                        Ok(updated_settings) => {
                            settings.set(Some(updated_settings));
                            saved.set(true);
                            error.set(None);

                            // Clear saved indicator after 3 seconds
                            let saved = saved.clone();
                            gloo::timers::callback::Timeout::new(3000, move || {
                                saved.set(false);
                            }).forget();
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to save settings: {:?}", e)));
                        }
                    }
                    saving.set(false);
                });
            }
        }
    };

    if *loading {
        return html! {
            <div class="max-w-4xl mx-auto space-y-8">
                <div class="flex items-center justify-center h-64">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                    <span class="ml-2 text-gray-600">{"Loading settings..."}</span>
                </div>
            </div>
        };
    }

    let current_settings = match (*settings).clone() {
        Some(s) => s,
        None => return html! {
            <div class="max-w-4xl mx-auto space-y-8">
                <div class="text-center text-red-600">
                    {"Failed to load settings"}
                </div>
            </div>
        },
    };

    html! {
        <div class="max-w-4xl mx-auto space-y-8">
            // Header
            <div>
                <h1 class="text-2xl font-bold text-gray-900">{"Settings"}</h1>
                <p class="text-gray-600">{"Configure your LogLens preferences"}</p>
            </div>

            if let Some(error_msg) = (*error).clone() {
                <div class="bg-red-50 border border-red-200 rounded-md p-4">
                    <div class="flex">
                        <div class="ml-3">
                            <h3 class="text-sm font-medium text-red-800">{"Error"}</h3>
                            <div class="mt-2 text-sm text-red-700">
                                <p>{error_msg}</p>
                            </div>
                        </div>
                    </div>
                </div>
            }

            <form onsubmit={on_save}>
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
                    // AI Provider Settings
                    <div class="bg-white rounded-lg shadow p-6">
                        <h2 class="text-lg font-semibold text-gray-900 mb-4">{"AI Provider Configuration"}</h2>

                        <div class="space-y-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    {"Provider"}
                                </label>
                                <select
                                    value={current_settings.default_provider.clone()}
                                    onchange={on_provider_change}
                                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                >
                                    <option value="openrouter">{"OpenRouter"}</option>
                                    <option value="openai">{"OpenAI"}</option>
                                    <option value="claude">{"Anthropic Claude"}</option>
                                    <option value="gemini">{"Google Gemini"}</option>
                                </select>
                            </div>

                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    {"API Key"}
                                </label>
                                <input
                                    type="password"
                                    value={current_settings.api_key.clone()}
                                    onchange={on_api_key_change}
                                    placeholder="Enter your API key"
                                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                />
                                <p class="text-xs text-gray-500 mt-1">
                                    {"Your API key is stored securely and never shared"}
                                </p>
                            </div>

                            // Model Selection
                            <div>
                                <div class="flex items-center justify-between mb-2">
                                    <label class="block text-sm font-medium text-gray-700">
                                        {"Selected Model"}
                                    </label>
                                    <button
                                        type="button"
                                        onclick={load_models}
                                        class="text-xs text-blue-600 hover:text-blue-800"
                                    >
                                        {"Refresh Models"}
                                    </button>
                                </div>
                                <select
                                    value={current_settings.selected_model.clone().unwrap_or_default()}
                                    onchange={on_model_change}
                                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                    disabled={available_models.is_empty()}
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
                                    if available_models.is_empty() {
                                        html! {
                                            <p class="text-xs text-gray-500 mt-1">
                                                {"Enter API key and click 'Refresh Models' to load available models"}
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
                        </div>
                    </div>

                    // Analysis Settings
                    <div class="bg-white rounded-lg shadow p-6">
                        <h2 class="text-lg font-semibold text-gray-900 mb-4">{"Analysis Settings"}</h2>

                        <div class="space-y-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    {"Analysis Timeout (seconds)"}
                                </label>
                                <input
                                    type="number"
                                    value={current_settings.analysis_timeout_seconds.unwrap_or(300).to_string()}
                                    onchange={on_timeout_change}
                                    min="60"
                                    max="1800"
                                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                />
                                <p class="text-xs text-gray-500 mt-1">
                                    {"Timeout for analysis operations (60-1800 seconds)"}
                                </p>
                            </div>

                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    {"Max Lines to Analyze"}
                                </label>
                                <input
                                    type="number"
                                    value={current_settings.max_lines.to_string()}
                                    onchange={on_max_lines_change}
                                    min="100"
                                    max="10000"
                                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                />
                                <p class="text-xs text-gray-500 mt-1">
                                    {"Maximum number of log lines to analyze (100-10000)"}
                                </p>
                            </div>

                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    {"Default Log Level"}
                                </label>
                                <select
                                    value={current_settings.default_level.clone()}
                                    onchange={move |e: Event| {
                                        let select: HtmlInputElement = e.target_unchecked_into();
                                        if let Some(current_settings) = (*settings).clone() {
                                            let mut updated_settings = current_settings;
                                            updated_settings.default_level = select.value();
                                            settings.set(Some(updated_settings));
                                        }
                                    }}
                                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                >
                                    <option value="ERROR">{"ERROR"}</option>
                                    <option value="WARN">{"WARN"}</option>
                                    <option value="INFO">{"INFO"}</option>
                                    <option value="DEBUG">{"DEBUG"}</option>
                                </select>
                            </div>
                        </div>
                    </div>
                </div>

                // Save Button
                <div class="mt-8 flex justify-end space-x-4">
                    <button
                        type="submit"
                        disabled={*saving}
                        class="px-6 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors font-medium disabled:opacity-50"
                    >
                        {
                            if *saving {
                                "Saving..."
                            } else if *saved {
                                "✓ Saved"
                            } else {
                                "Save Settings"
                            }
                        }
                    </button>
                </div>
            </form>
        </div>
    }
}