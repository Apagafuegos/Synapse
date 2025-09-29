use yew::prelude::*;
use web_sys::{DragEvent, FileList, HtmlInputElement};
use wasm_bindgen::{JsCast, closure::Closure};

use crate::services::{ApiService, WasmBridge};
use crate::types::LogFile;
use crate::services::wasm_bridge::FileStats;

#[derive(Properties, PartialEq)]
pub struct FileUploadProps {
    pub project_id: String,
    pub on_upload: Callback<LogFile>,
    pub on_error: Callback<String>,
}

#[function_component(FileUpload)]
pub fn file_upload(props: &FileUploadProps) -> Html {
    let drag_over = use_state(|| false);
    let uploading = use_state(|| false);
    let preview_visible = use_state(|| false);
    let file_content = use_state(|| String::new());
    let file_stats = use_state(|| None::<crate::services::wasm_bridge::FileStats>);

    let on_drag_over = {
        let drag_over = drag_over.clone();
        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            drag_over.set(true);
        })
    };

    let on_drag_leave = {
        let drag_over = drag_over.clone();
        Callback::from(move |_: DragEvent| {
            drag_over.set(false);
        })
    };

    let handle_files = {
        let project_id = props.project_id.clone();
        let on_upload = props.on_upload.clone();
        let on_error = props.on_error.clone();
        let uploading = uploading.clone();
        let preview_visible = preview_visible.clone();
        let file_content = file_content.clone();
        let file_stats = file_stats.clone();

        Callback::from(move |files: FileList| {
            if files.length() == 0 {
                return;
            }

            let file = files.get(0).unwrap();
            let project_id = project_id.clone();
            let on_upload = on_upload.clone();
            let on_error = on_error.clone();
            let uploading = uploading.clone();
            let preview_visible = preview_visible.clone();
            let file_content = file_content.clone();
            let file_stats = file_stats.clone();

            // Read file for preview
            let file_reader = web_sys::FileReader::new().unwrap();
            let file_reader_for_closure = file_reader.clone();
            let file_content_clone = file_content.clone();
            let file_stats_clone = file_stats.clone();
            let preview_visible_clone = preview_visible.clone();
            let on_error_for_closure = on_error.clone();

            let onload = Closure::wrap(Box::new(move |_: web_sys::ProgressEvent| {
                if let Ok(result) = file_reader_for_closure.result() {
                    if let Some(content) = result.as_string() {
                        // Validate and get stats using WASM
                        if let Ok(validation) = WasmBridge::new().validate_file(&content) {
                            let stats = FileStats {
                                total_lines: validation.line_count,
                                total_chars: content.len(),
                                avg_line_length: if validation.line_count > 0 { content.len() / validation.line_count } else { 0 },
                                estimated_processing_time: validation.estimated_processing_time,
                            };
                            file_stats_clone.set(Some(stats));
                            file_content_clone.set(content);
                            preview_visible_clone.set(true);
                        } else {
                            on_error_for_closure.emit("File doesn't appear to be a valid log file".to_string());
                        }
                    }
                }
            }) as Box<dyn FnMut(_)>);

            file_reader.set_onload(Some(onload.as_ref().unchecked_ref()));
            file_reader.read_as_text(&file).unwrap();
            onload.forget();

            // Upload file
            uploading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match ApiService::upload_log_file(&project_id, file).await {
                    Ok(log_file) => {
                        uploading.set(false);
                        on_upload.emit(log_file);
                    }
                    Err(e) => {
                        uploading.set(false);
                        on_error.emit(format!("Upload failed: {:?}", e));
                    }
                }
            });
        })
    };

    let on_drop = {
        let drag_over = drag_over.clone();
        let handle_files = handle_files.clone();
        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            drag_over.set(false);
            
            // Access files from drag event target
            if let Some(target) = e.target() {
                if let Ok(input) = target.dyn_into::<HtmlInputElement>() {
                    if let Some(files) = input.files() {
                        handle_files.emit(files);
                    }
                }
            }
        })
    };

    let on_file_select = {
        let handle_files = handle_files.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
            if let Some(files) = input.files() {
                handle_files.emit(files);
            }
        })
    };

    let close_preview = {
        let preview_visible = preview_visible.clone();
        Callback::from(move |_| {
            preview_visible.set(false);
        })
    };

    html! {
        <div class="space-y-4">
            // Upload area
            <div
                class={classes!(
                    "drop-zone",
                    "relative",
                    "border-2",
                    "border-dashed",
                    "rounded-lg",
                    "p-8",
                    "text-center",
                    "transition-all",
                    "duration-300",
                    if *drag_over { "drag-over" } else { "border-gray-300" },
                    if *uploading { "pointer-events-none opacity-50" } else { "hover:border-gray-400" }
                )}
                ondragover={on_drag_over}
                ondragleave={on_drag_leave}
                ondrop={on_drop}
            >
                if *uploading {
                    <div class="flex flex-col items-center">
                        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                        <p class="mt-2 text-sm text-gray-600">{"Uploading and processing..."}</p>
                    </div>
                } else {
                    <>
                        <svg class="mx-auto h-12 w-12 text-gray-400" stroke="currentColor" fill="none" viewBox="0 0 48 48">
                            <path d="M28 8H12a4 4 0 00-4 4v20m32-12v8m0 0v8a4 4 0 01-4 4H12a4 4 0 01-4-4v-4m32-4l-3.172-3.172a4 4 0 00-5.656 0L28 28M8 32l9.172-9.172a4 4 0 015.656 0L28 28m0 0l4 4m4-24h8m-4-4v8m-12 4h.02" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
                        </svg>
                        <div class="mt-4">
                            <label for="file-upload" class="cursor-pointer">
                                <span class="mt-2 block text-sm font-medium text-gray-900">
                                    {"Drop log files here, or "}
                                    <span class="text-blue-600">{"browse"}</span>
                                </span>
                                <input id="file-upload" name="file-upload" type="file" class="sr-only" onchange={on_file_select} accept=".log,.txt" />
                            </label>
                            <p class="mt-1 text-xs text-gray-500">
                                {"Supports .log, .txt files up to 50MB"}
                            </p>
                        </div>
                    </>
                }
            </div>

            // File preview
            if *preview_visible {
                <div class="bg-white border border-gray-200 rounded-lg shadow-sm">
                    <div class="px-4 py-3 border-b border-gray-200 flex justify-between items-center">
                        <h3 class="text-lg font-medium text-gray-900">{"File Preview"}</h3>
                        <button 
                            class="text-gray-400 hover:text-gray-600"
                            onclick={close_preview}
                        >
                            <svg class="h-5 w-5" fill="currentColor" viewBox="0 0 20 20">
                                <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                            </svg>
                        </button>
                    </div>
                    <div class="p-4">
                        if let Some(stats) = file_stats.as_ref() {
                            <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-4">
                                <div class="text-center">
                                    <div class="text-2xl font-bold text-gray-900">{stats.total_lines}</div>
                                    <div class="text-sm text-gray-500">{"Lines"}</div>
                                </div>
                                <div class="text-center">
                                    <div class="text-2xl font-bold text-gray-900">{format!("{:.1}KB", stats.total_chars as f32 / 1024.0)}</div>
                                    <div class="text-sm text-gray-500">{"Size"}</div>
                                </div>
                                <div class="text-center">
                                    <div class="text-2xl font-bold text-gray-900">{stats.avg_line_length}</div>
                                    <div class="text-sm text-gray-500">{"Avg Length"}</div>
                                </div>
                                <div class="text-center">
                                    <div class="text-2xl font-bold text-gray-900">{format!("{}s", stats.estimated_processing_time)}</div>
                                    <div class="text-sm text-gray-500">{"Est. Time"}</div>
                                </div>
                            </div>
                        }
                        
                        <div class="bg-gray-50 rounded-lg p-3">
                            <div class="text-sm font-medium text-gray-700 mb-2">{"File Preview (first 10 lines):"}</div>
                            <pre class="log-entry text-xs text-gray-600 whitespace-pre-wrap overflow-x-auto">
                                {file_content.lines().take(10).collect::<Vec<_>>().join("\n")}
                            </pre>
                        </div>
                    </div>
                </div>
            }
        </div>
    }
}