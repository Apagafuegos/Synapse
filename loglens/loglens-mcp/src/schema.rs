use serde_json::json;

/// JSON schema for list_projects tool
pub fn list_projects_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "names": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Optional list of project names to filter by"
            }
        }
    })
}

/// JSON schema for get_project tool
pub fn get_project_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "project_id": { "type": "string" }
        },
        "required": ["project_id"]
    })
}

/// JSON schema for list_analyses tool
pub fn list_analyses_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "project_id": { "type": "string" },
            "limit": { 
                "type": "integer", 
                "default": 50, 
                "maximum": 200,
                "minimum": 1
            },
            "offset": { 
                "type": "integer", 
                "default": 0,
                "minimum": 0
            }
        },
        "required": ["project_id"]
    })
}

/// JSON schema for get_analysis tool
pub fn get_analysis_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "analysis_id": { "type": "string" }
        },
        "required": ["analysis_id"]
    })
}

/// JSON schema for get_analysis_status tool
pub fn get_analysis_status_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "analysis_id": { "type": "string" }
        },
        "required": ["analysis_id"]
    })
}

/// JSON schema for analyze_file tool
pub fn analyze_file_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "project_id": { "type": "string" },
            "file_id": { "type": "string" },
            "provider": { 
                "type": "string", 
                "enum": ["openrouter", "openai", "claude", "gemini"], 
                "default": "openrouter" 
            }
        },
        "required": ["project_id", "file_id"]
    })
}