use serde_json::{json, Map, Value};

/// JSON schema for list_projects tool
pub fn list_projects_schema() -> Map<String, Value> {
    let mut schema = Map::new();
    schema.insert("type".to_string(), json!("object"));
    
    let mut properties = Map::new();
    let mut names = Map::new();
    names.insert("type".to_string(), json!("array"));
    names.insert("items".to_string(), json!({"type": "string"}));
    names.insert("description".to_string(), json!("Optional list of project names to filter by"));
    properties.insert("names".to_string(), Value::Object(names));
    
    schema.insert("properties".to_string(), Value::Object(properties));
    schema
}

/// JSON schema for get_project tool
pub fn get_project_schema() -> Map<String, Value> {
    let mut schema = Map::new();
    schema.insert("type".to_string(), json!("object"));
    
    let mut properties = Map::new();
    properties.insert("project_id".to_string(), json!({"type": "string"}));
    
    schema.insert("properties".to_string(), Value::Object(properties));
    schema.insert("required".to_string(), json!(["project_id"]));
    schema
}

/// JSON schema for list_analyses tool
pub fn list_analyses_schema() -> Map<String, Value> {
    let mut schema = Map::new();
    schema.insert("type".to_string(), json!("object"));
    
    let mut properties = Map::new();
    properties.insert("project_id".to_string(), json!({"type": "string"}));
    properties.insert("limit".to_string(), json!({
        "type": "integer",
        "default": 50,
        "maximum": 200,
        "minimum": 1
    }));
    properties.insert("offset".to_string(), json!({
        "type": "integer",
        "default": 0,
        "minimum": 0
    }));
    
    schema.insert("properties".to_string(), Value::Object(properties));
    schema.insert("required".to_string(), json!(["project_id"]));
    schema
}

/// JSON schema for get_analysis tool
pub fn get_analysis_schema() -> Map<String, Value> {
    let mut schema = Map::new();
    schema.insert("type".to_string(), json!("object"));
    
    let mut properties = Map::new();
    properties.insert("analysis_id".to_string(), json!({"type": "string"}));
    
    schema.insert("properties".to_string(), Value::Object(properties));
    schema.insert("required".to_string(), json!(["analysis_id"]));
    schema
}

/// JSON schema for get_analysis_status tool
pub fn get_analysis_status_schema() -> Map<String, Value> {
    let mut schema = Map::new();
    schema.insert("type".to_string(), json!("object"));
    
    let mut properties = Map::new();
    properties.insert("analysis_id".to_string(), json!({"type": "string"}));
    
    schema.insert("properties".to_string(), Value::Object(properties));
    schema.insert("required".to_string(), json!(["analysis_id"]));
    schema
}

/// JSON schema for analyze_file tool
pub fn analyze_file_schema() -> Map<String, Value> {
    let mut schema = Map::new();
    schema.insert("type".to_string(), json!("object"));
    
    let mut properties = Map::new();
    properties.insert("project_id".to_string(), json!({"type": "string"}));
    properties.insert("file_id".to_string(), json!({"type": "string"}));
    properties.insert("provider".to_string(), json!({
        "type": "string",
        "enum": ["openrouter", "openai", "claude", "gemini"],
        "default": "openrouter"
    }));
    
    schema.insert("properties".to_string(), Value::Object(properties));
    schema.insert("required".to_string(), json!(["project_id", "file_id"]));
    schema
}