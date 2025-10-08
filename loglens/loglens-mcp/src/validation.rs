use serde_json::Value;
use anyhow::{Result, anyhow};

/// Validate input parameters against the expected schema
pub fn validate_tool_params(tool_name: &str, params: &Value) -> Result<()> {
    match tool_name {
        "list_projects" => validate_list_projects(params),
        "get_project" => validate_get_project(params),
        "list_analyses" => validate_list_analyses(params),
        "get_analysis" => validate_get_analysis(params),
        "get_analysis_status" => validate_get_analysis_status(params),
        "analyze_file" => validate_analyze_file(params),
        _ => Err(anyhow!("Unknown tool: {}", tool_name)),
    }
}

fn validate_list_projects(params: &Value) -> Result<()> {
    if let Value::Object(map) = params {
        if let Some(names) = map.get("names") {
            if let Value::Array(names_array) = names {
                for (i, name) in names_array.iter().enumerate() {
                    if !name.is_string() {
                        return Err(anyhow!("names[{}] must be a string, found: {}", i, name));
                    }
                }
            } else {
                return Err(anyhow!("names must be an array"));
            }
        }
    }
    Ok(())
}

fn validate_get_project(params: &Value) -> Result<()> {
    if let Value::Object(map) = params {
        let project_id = map.get("project_id")
            .ok_or_else(|| anyhow!("project_id is required"))?;
        
        if !project_id.is_string() {
            return Err(anyhow!("project_id must be a string"));
        }
        
        if project_id.as_str().unwrap().is_empty() {
            return Err(anyhow!("project_id cannot be empty"));
        }
    } else {
        return Err(anyhow!("Parameters must be an object"));
    }
    Ok(())
}

fn validate_list_analyses(params: &Value) -> Result<()> {
    if let Value::Object(map) = params {
        // Validate project_id (required)
        let project_id = map.get("project_id")
            .ok_or_else(|| anyhow!("project_id is required"))?;
        
        if !project_id.is_string() {
            return Err(anyhow!("project_id must be a string"));
        }
        
        if project_id.as_str().unwrap().is_empty() {
            return Err(anyhow!("project_id cannot be empty"));
        }
        
        // Validate limit (optional)
        if let Some(limit) = map.get("limit") {
            if let Some(limit_num) = limit.as_i64() {
                if limit_num < 1 || limit_num > 200 {
                    return Err(anyhow!("limit must be between 1 and 200"));
                }
            } else {
                return Err(anyhow!("limit must be an integer"));
            }
        }
        
        // Validate offset (optional)
        if let Some(offset) = map.get("offset") {
            if let Some(offset_num) = offset.as_i64() {
                if offset_num < 0 {
                    return Err(anyhow!("offset must be non-negative"));
                }
            } else {
                return Err(anyhow!("offset must be an integer"));
            }
        }
    } else {
        return Err(anyhow!("Parameters must be an object"));
    }
    Ok(())
}

fn validate_get_analysis(params: &Value) -> Result<()> {
    if let Value::Object(map) = params {
        let analysis_id = map.get("analysis_id")
            .ok_or_else(|| anyhow!("analysis_id is required"))?;
        
        if !analysis_id.is_string() {
            return Err(anyhow!("analysis_id must be a string"));
        }
        
        if analysis_id.as_str().unwrap().is_empty() {
            return Err(anyhow!("analysis_id cannot be empty"));
        }
    } else {
        return Err(anyhow!("Parameters must be an object"));
    }
    Ok(())
}

fn validate_get_analysis_status(params: &Value) -> Result<()> {
    if let Value::Object(map) = params {
        let analysis_id = map.get("analysis_id")
            .ok_or_else(|| anyhow!("analysis_id is required"))?;
        
        if !analysis_id.is_string() {
            return Err(anyhow!("analysis_id must be a string"));
        }
        
        if analysis_id.as_str().unwrap().is_empty() {
            return Err(anyhow!("analysis_id cannot be empty"));
        }
    } else {
        return Err(anyhow!("Parameters must be an object"));
    }
    Ok(())
}

fn validate_analyze_file(params: &Value) -> Result<()> {
    if let Value::Object(map) = params {
        // Validate project_id (required)
        let project_id = map.get("project_id")
            .ok_or_else(|| anyhow!("project_id is required"))?;
        
        if !project_id.is_string() {
            return Err(anyhow!("project_id must be a string"));
        }
        
        if project_id.as_str().unwrap().is_empty() {
            return Err(anyhow!("project_id cannot be empty"));
        }
        
        // Validate file_id (required)
        let file_id = map.get("file_id")
            .ok_or_else(|| anyhow!("file_id is required"))?;
        
        if !file_id.is_string() {
            return Err(anyhow!("file_id must be a string"));
        }
        
        if file_id.as_str().unwrap().is_empty() {
            return Err(anyhow!("file_id cannot be empty"));
        }
        
        // Validate provider (optional)
        if let Some(provider) = map.get("provider") {
            if let Some(provider_str) = provider.as_str() {
                let valid_providers = ["openrouter", "openai", "claude", "gemini"];
                if !valid_providers.contains(&provider_str) {
                    return Err(anyhow!(
                        "provider must be one of: {}", 
                        valid_providers.join(", ")
                    ));
                }
            } else {
                return Err(anyhow!("provider must be a string"));
            }
        }
    } else {
        return Err(anyhow!("Parameters must be an object"));
    }
    Ok(())
}