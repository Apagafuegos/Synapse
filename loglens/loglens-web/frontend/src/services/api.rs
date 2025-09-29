use gloo_net::http::Request;
use serde::Serialize;
use wasm_bindgen::JsValue;
use web_sys::{FormData, File};

use crate::types::*;

const API_BASE: &str = "/api/v1";

#[derive(Clone)]
pub struct ApiService;

impl ApiService {
    pub fn new() -> Self {
        Self
    }

    // Project management
    pub async fn get_projects() -> Result<Vec<Project>, JsValue> {
        let response = Request::get(&format!("{}/projects", API_BASE))
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            return Err(JsValue::from_str(&format!("HTTP {}", response.status())));
        }

        response
            .json::<Vec<Project>>()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub async fn create_project(request: CreateProjectRequest) -> Result<Project, JsValue> {
        let response = Request::post(&format!("{}/projects", API_BASE))
            .header("Content-Type", "application/json")
            .json(&request)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            return Err(JsValue::from_str(&format!("HTTP {}", response.status())));
        }

        response
            .json::<Project>()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub async fn get_project(id: &str) -> Result<Project, JsValue> {
        let response = Request::get(&format!("{}/projects/{}", API_BASE, id))
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            return Err(JsValue::from_str(&format!("HTTP {}", response.status())));
        }

        response
            .json::<Project>()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub async fn delete_project(id: &str) -> Result<(), JsValue> {
        let response = Request::delete(&format!("{}/projects/{}", API_BASE, id))
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            return Err(JsValue::from_str(&format!("HTTP {}", response.status())));
        }

        Ok(())
    }

    // File management
    pub async fn get_log_files(project_id: &str) -> Result<Vec<LogFile>, JsValue> {
        let response = Request::get(&format!("{}/projects/{}/files", API_BASE, project_id))
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            return Err(JsValue::from_str(&format!("HTTP {}", response.status())));
        }

        response
            .json::<Vec<LogFile>>()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub async fn upload_log_file(project_id: &str, file: File) -> Result<LogFile, JsValue> {
        let form_data = FormData::new()
            .map_err(|e| JsValue::from_str(&format!("Failed to create FormData: {:?}", e)))?;
        
        form_data
            .append_with_blob("file", &file)
            .map_err(|e| JsValue::from_str(&format!("Failed to append file: {:?}", e)))?;

        let response = Request::post(&format!("{}/projects/{}/files", API_BASE, project_id))
            .body(form_data)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            return Err(JsValue::from_str(&format!("HTTP {}", response.status())));
        }

        response
            .json::<LogFile>()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub async fn delete_log_file(project_id: &str, file_id: &str) -> Result<(), JsValue> {
        let response = Request::delete(&format!(
            "{}/projects/{}/files/{}",
            API_BASE, project_id, file_id
        ))
        .send()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            return Err(JsValue::from_str(&format!("HTTP {}", response.status())));
        }

        Ok(())
    }

    // Analysis management
    pub async fn start_analysis(
        project_id: &str,
        file_id: &str,
        request: AnalysisRequest,
    ) -> Result<Analysis, JsValue> {
        let response = Request::post(&format!(
            "{}/projects/{}/files/{}/analyze",
            API_BASE, project_id, file_id
        ))
        .header("Content-Type", "application/json")
        .json(&request)
        .map_err(|e| JsValue::from_str(&e.to_string()))?
        .send()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            return Err(JsValue::from_str(&format!("HTTP {}", response.status())));
        }

        response
            .json::<Analysis>()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub async fn get_analysis(id: &str) -> Result<Analysis, JsValue> {
        let response = Request::get(&format!("{}/analyses/{}", API_BASE, id))
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            return Err(JsValue::from_str(&format!("HTTP {}", response.status())));
        }

        response
            .json::<Analysis>()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub async fn list_analyses(project_id: &str) -> Result<AnalysisListResponse, JsValue> {
        let response = Request::get(&format!("{}/projects/{}/analyses", API_BASE, project_id))
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            return Err(JsValue::from_str(&format!("HTTP {}", response.status())));
        }

        response
            .json::<AnalysisListResponse>()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    // Settings management
    pub async fn get_settings() -> Result<Settings, JsValue> {
        let response = Request::get(&format!("{}/settings", API_BASE))
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            return Err(JsValue::from_str(&format!("HTTP {}", response.status())));
        }

        response
            .json::<Settings>()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub async fn update_settings(settings: Settings) -> Result<Settings, JsValue> {
        let response = Request::patch(&format!("{}/settings", API_BASE))
            .header("Content-Type", "application/json")
            .json(&settings)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            return Err(JsValue::from_str(&format!("HTTP {}", response.status())));
        }

        response
            .json::<Settings>()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    // Model management
    pub async fn get_available_models(request: ModelsRequest) -> Result<ModelListResponse, JsValue> {
        let response = Request::post(&format!("{}/models/available", API_BASE))
            .header("Content-Type", "application/json")
            .json(&request)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            return Err(JsValue::from_str(&format!("HTTP {}", response.status())));
        }

        response
            .json::<ModelListResponse>()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub async fn clear_models_cache(provider: Option<String>) -> Result<String, JsValue> {
        #[derive(Serialize)]
        struct ClearCacheRequest {
            provider: Option<String>,
        }

        let request = ClearCacheRequest { provider };

        let response = Request::post(&format!("{}/models/cache/clear", API_BASE))
            .header("Content-Type", "application/json")
            .json(&request)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        if !response.ok() {
            return Err(JsValue::from_str(&format!("HTTP {}", response.status())));
        }

        response
            .json::<String>()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}