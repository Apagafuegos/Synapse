use yew::prelude::*;




#[derive(Properties, PartialEq)]
pub struct AnalysisCardProps {
    pub analysis: crate::types::Analysis,
    pub on_view: Callback<String>,
}

#[function_component(AnalysisCard)]
pub fn analysis_card(props: &AnalysisCardProps) -> Html {
    let analysis = &props.analysis;
    let on_view = props.on_view.clone();
    let analysis_id = analysis.id.clone();

    let status_class = match analysis.status {
        crate::types::AnalysisStatus::Completed => "bg-green-100 text-green-800",
        crate::types::AnalysisStatus::Failed => "bg-red-100 text-red-800",
        crate::types::AnalysisStatus::Running => "bg-yellow-100 text-yellow-800",
        crate::types::AnalysisStatus::Pending => "bg-gray-100 text-gray-800",
    };

    let status_icon = match analysis.status {
        crate::types::AnalysisStatus::Completed => "✓",
        crate::types::AnalysisStatus::Failed => "✗",
        crate::types::AnalysisStatus::Running => "⟳",
        crate::types::AnalysisStatus::Pending => "◦",
    };

    let duration = if let Some(completed) = analysis.completed_at {
        let duration = completed.signed_duration_since(analysis.started_at);
        Some(format!("{}s", duration.num_seconds()))
    } else {
        None
    };

    html! {
        <div class="bg-white rounded-lg shadow-md p-6 hover:shadow-lg transition-shadow">
            <div class="flex items-start justify-between mb-4">
                <div class="flex items-center space-x-3">
                    <span class="text-2xl">{status_icon}</span>
                    <div>
                        <h3 class="text-lg font-semibold text-gray-900">
                            {format!("Analysis {}", &analysis.id[..8])}
                        </h3>
                        <p class="text-sm text-gray-500">
                            {analysis.started_at.format("%Y-%m-%d %H:%M").to_string()}
                        </p>
                        <p class="text-sm text-gray-500">
                            {format!("Provider: {} | Level: {}", analysis.provider, analysis.level_filter)}
                        </p>
                    </div>
                </div>
                <span class={format!("px-2 py-1 rounded-full text-xs font-medium {}", status_class)}>
                    {analysis.status.as_str()}
                </span>
            </div>

            if let Some(dur) = duration {
                <div class="text-sm text-gray-600 mb-3">
                    <span class="font-medium">{"Duration:"}</span> {dur}
                </div>
            }

            if let Some(error) = &analysis.error_message {
                <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded-md">
                    <p class="text-sm text-red-700 font-medium">{"Error:"}</p>
                    <p class="text-sm text-red-600 mt-1">{error}</p>
                </div>
            }

            if analysis.status == crate::types::AnalysisStatus::Completed {
                <div class="flex justify-end">
                    <button
                        onclick={move |_| on_view.emit(analysis_id.clone())}
                        class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors text-sm font-medium"
                    >
                        {"View Results"}
                    </button>
                </div>
            } else if analysis.status == crate::types::AnalysisStatus::Running {
                <div class="mb-4">
                    <div class="w-full bg-gray-200 rounded-full h-2">
                        <div class="bg-blue-600 h-2 rounded-full animate-pulse" style="width: 45%"></div>
                    </div>
                    <p class="text-xs text-gray-500 mt-1">{"Processing logs..."}</p>
                </div>
            }
        </div>
    }
}