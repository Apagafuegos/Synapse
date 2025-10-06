#[cfg(all(test, feature = "project-management", feature = "mcp-server"))]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    use std::path::Path;
    use std::time::Duration;
    use tokio::time::sleep;
    
    use crate::project::{
        database::create_pool,
        init::initialize_project,
        queries::{create_analysis, get_analysis_by_id, query_analyses},
    };
    use crate::mcp_server::LogLensServer;
    use rmcp::model::{CallToolRequestParam, ListToolsResult};

    fn create_test_project() -> (TempDir, String) {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_str().unwrap().to_string();
        
        // Create test log file
        let log_content = r#"
[ERROR] Database connection failed: connection timeout
[WARN] Retrying database connection (attempt 1/3)
[ERROR] Database connection failed: connection timeout  
[INFO] Application started successfully
[ERROR] Database connection failed: connection timeout
[WARN] Retrying database connection (attempt 2/3)
"#;
        fs::write(temp_dir.path().join("test.log"), log_content).unwrap();
        
        // Initialize project
        initialize_project(&project_path).unwrap();
        
        (temp_dir, project_path)
    }

    #[tokio::test]
    async fn test_list_tools_includes_new_tools() {
        let server = LogLensServer::new();
        let tools_result = server.list_tools(None, Default::default()).await.unwrap();
        
        let tool_names: Vec<String> = tools_result.tools.iter()
            .map(|t| t.name.clone())
            .collect();
        
        assert!(tool_names.contains(&"analyze_logs".to_string()));
        assert!(tool_names.contains(&"parse_logs".to_string()));
        assert!(tool_names.contains(&"filter_logs".to_string()));
        assert!(tool_names.contains(&"add_log_file".to_string()));
        assert!(tool_names.contains(&"get_analysis".to_string()));
        assert!(tool_names.contains(&"query_analyses".to_string()));
    }

    #[tokio::test]
    async fn test_add_log_file_workflow() {
        let (_temp_dir, project_path) = create_test_project();
        let server = LogLensServer::new();
        
        // Call add_log_file via MCP
        let mut args = serde_json::Map::new();
        args.insert("project_path".to_string(), serde_json::Value::String(project_path.clone()));
        args.insert("log_file_path".to_string(), serde_json::Value::String("test.log".to_string()));
        args.insert("level".to_string(), serde_json::Value::String("ERROR".to_string()));
        args.insert("provider".to_string(), serde_json::Value::String("mock".to_string()));
        args.insert("auto_analyze".to_string(), serde_json::Value::Bool(false)); // Disable for test
        
        let request = CallToolRequestParam {
            name: "add_log_file".to_string(),
            arguments: Some(args),
        };
        
        let result = server.call_tool(request, Default::default()).await.unwrap();
        
        // Parse response
        let response: serde_json::Value = serde_json::from_str(&result.content[0].text).unwrap();
        assert!(response["success"].as_bool().unwrap());
        assert!(response["analysis_id"].is_string());
        assert_eq!(response["status"].as_str().unwrap(), "pending");
        
        let analysis_id = response["analysis_id"].as_str().unwrap();
        
        // Verify analysis was created in database
        let pool = create_pool(&Path::new(&project_path).join(".loglens")).await.unwrap();
        let db_result = get_analysis_by_id(&pool, analysis_id).await.unwrap();
        assert!(db_result.is_some());
        
        let (analysis, _) = db_result.unwrap();
        assert_eq!(analysis.status, crate::project::models::AnalysisStatus::Pending);
        assert_eq!(analysis.provider, "mock");
        assert_eq!(analysis.level, "ERROR");
    }

    #[tokio::test]
    async fn test_get_analysis_formats() {
        let (_temp_dir, project_path) = create_test_project();
        let server = LogLensServer::new();
        
        // Create a completed analysis with results
        let pool = create_pool(&Path::new(&project_path).join(".loglens")).await.unwrap();
        let project_id = crate::project::get_or_create_project(&pool, &project_path).await.unwrap();
        
        let analysis_id = create_analysis(
            &pool,
            project_id,
            "test.log".to_string(),
            "mock".to_string(),
            "ERROR".to_string(),
        ).await.unwrap();
        
        // Store test results
        let patterns = vec![
            crate::project::models::Pattern {
                pattern: "Database connection failed".to_string(),
                count: 3,
            }
        ];
        
        crate::project::store_analysis_results(
            &pool,
            &analysis_id,
            Some("Test summary".to_string()),
            Some("Test full report".to_string()),
            patterns,
            Some(3),
        ).await.unwrap();
        
        // Update status to completed
        crate::project::update_analysis_status(
            &pool,
            &analysis_id,
            crate::project::models::AnalysisStatus::Completed,
            Some(chrono::Utc::now()),
        ).await.unwrap();
        
        // Test summary format
        let mut args = serde_json::Map::new();
        args.insert("analysis_id".to_string(), serde_json::Value::String(analysis_id.clone()));
        args.insert("format".to_string(), serde_json::Value::String("summary".to_string()));
        
        let request = CallToolRequestParam {
            name: "get_analysis".to_string(),
            arguments: Some(args.clone()),
        };
        
        let result = server.call_tool(request, Default::default()).await.unwrap();
        let response: serde_json::Value = serde_json::from_str(&result.content[0].text).unwrap();
        
        assert!(response["success"].as_bool().unwrap());
        assert_eq!(response["analysis_id"].as_str().unwrap(), analysis_id);
        assert_eq!(response["summary"].as_str().unwrap(), "Test summary");
        assert_eq!(response["issues_found"].as_i64().unwrap(), 3);
        assert!(response["patterns"].as_array().unwrap().len() > 0);
        
        // Test full format
        args.insert("format".to_string(), serde_json::Value::String("full".to_string()));
        let request = CallToolRequestParam {
            name: "get_analysis".to_string(),
            arguments: Some(args.clone()),
        };
        
        let result = server.call_tool(request, Default::default()).await.unwrap();
        let response: serde_json::Value = serde_json::from_str(&result.content[0].text).unwrap();
        
        assert!(response["success"].as_bool().unwrap());
        assert!(response["analysis"].is_object());
        assert!(response["result"].is_object());
        
        // Test structured format
        args.insert("format".to_string(), serde_json::Value::String("structured".to_string()));
        let request = CallToolRequestParam {
            name: "get_analysis".to_string(),
            arguments: Some(args),
        };
        
        let result = server.call_tool(request, Default::default()).await.unwrap();
        let response: serde_json::Value = serde_json::from_str(&result.content[0].text).unwrap();
        
        assert!(response["success"].as_bool().unwrap());
        assert!(response["provider"].is_string());
        assert!(response["level"].is_string());
        assert!(response["full_report"].is_string());
    }

    #[tokio::test]
    async fn test_query_analyses_filters() {
        let (_temp_dir, project_path) = create_test_project();
        let server = LogLensServer::new();
        
        let pool = create_pool(&Path::new(&project_path).join(".loglens")).await.unwrap();
        let project_id = crate::project::get_or_create_project(&pool, &project_path).await.unwrap();
        
        // Create multiple analyses with different statuses
        let analysis1_id = create_analysis(
            &pool,
            project_id.clone(),
            "test1.log".to_string(),
            "mock".to_string(),
            "ERROR".to_string(),
        ).await.unwrap();
        
        let analysis2_id = create_analysis(
            &pool,
            project_id.clone(),
            "test2.log".to_string(),
            "mock".to_string(),
            "WARN".to_string(),
        ).await.unwrap();
        
        let analysis3_id = create_analysis(
            &pool,
            project_id.clone(),
            "test3.log".to_string(),
            "mock".to_string(),
            "INFO".to_string(),
        ).await.unwrap();
        
        // Update some to completed
        crate::project::update_analysis_status(
            &pool,
            &analysis2_id,
            crate::project::models::AnalysisStatus::Completed,
            Some(chrono::Utc::now()),
        ).await.unwrap();
        
        crate::project::update_analysis_status(
            &pool,
            &analysis3_id,
            crate::project::models::AnalysisStatus::Failed,
            Some(chrono::Utc::now()),
        ).await.unwrap();
        
        // Test query by project_path
        let mut args = serde_json::Map::new();
        args.insert("project_path".to_string(), serde_json::Value::String(project_path.clone()));
        
        let request = CallToolRequestParam {
            name: "query_analyses".to_string(),
            arguments: Some(args.clone()),
        };
        
        let result = server.call_tool(request, Default::default()).await.unwrap();
        let response: serde_json::Value = serde_json::from_str(&result.content[0].text).unwrap();
        
        assert!(response["success"].as_bool().unwrap());
        assert_eq!(response["count"].as_u64().unwrap(), 3);
        assert_eq!(response["analyses"].as_array().unwrap().len(), 3);
        
        // Test query by status filter
        args.insert("status".to_string(), serde_json::Value::String("pending".to_string()));
        let request = CallToolRequestParam {
            name: "query_analyses".to_string(),
            arguments: Some(args.clone()),
        };
        
        let result = server.call_tool(request, Default::default()).await.unwrap();
        let response: serde_json::Value = serde_json::from_str(&result.content[0].text).unwrap();
        
        assert_eq!(response["count"].as_u64().unwrap(), 1);
        
        // Test query by completed status
        args.insert("status".to_string(), serde_json::Value::String("completed".to_string()));
        let request = CallToolRequestParam {
            name: "query_analyses".to_string(),
            arguments: Some(args.clone()),
        };
        
        let result = server.call_tool(request, Default::default()).await.unwrap();
        let response: serde_json::Value = serde_json::from_str(&result.content[0].text).unwrap();
        
        assert_eq!(response["count"].as_u64().unwrap(), 1);
        
        // Test query with limit
        args.remove("status");
        args.insert("limit".to_string(), serde_json::Value::Number(serde_json::Number::from(2)));
        let request = CallToolRequestParam {
            name: "query_analyses".to_string(),
            arguments: Some(args),
        };
        
        let result = server.call_tool(request, Default::default()).await.unwrap();
        let response: serde_json::Value = serde_json::from_str(&result.content[0].text).unwrap();
        
        assert_eq!(response["count"].as_u64().unwrap(), 2);
        assert_eq!(response["analyses"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let server = LogLensServer::new();
        
        // Test invalid project path
        let mut args = serde_json::Map::new();
        args.insert("project_path".to_string(), serde_json::Value::String("/nonexistent/path".to_string()));
        args.insert("log_file_path".to_string(), serde_json::Value::String("test.log".to_string()));
        
        let request = CallToolRequestParam {
            name: "add_log_file".to_string(),
            arguments: Some(args),
        };
        
        let result = server.call_tool(request, Default::default()).await;
        assert!(result.is_err());
        
        // Test nonexistent analysis_id
        let mut args = serde_json::Map::new();
        args.insert("analysis_id".to_string(), serde_json::Value::String("nonexistent-id".to_string()));
        
        let request = CallToolRequestParam {
            name: "get_analysis".to_string(),
            arguments: Some(args),
        };
        
        let result = server.call_tool(request, Default::default()).await;
        assert!(result.is_err());
        
        // Test invalid analysis_id format
        let mut args = serde_json::Map::new();
        args.insert("analysis_id".to_string(), serde_json::Value::String("".to_string()));
        
        let request = CallToolRequestParam {
            name: "get_analysis".to_string(),
            arguments: Some(args),
        };
        
        let result = server.call_tool(request, Default::default()).await;
        assert!(result.is_err());
        
        // Test invalid format
        let mut args = serde_json::Map::new();
        args.insert("analysis_id".to_string(), serde_json::Value::String("some-id".to_string()));
        args.insert("format".to_string(), serde_json::Value::String("invalid".to_string()));
        
        let request = CallToolRequestParam {
            name: "get_analysis".to_string(),
            arguments: Some(args),
        };
        
        let result = server.call_tool(request, Default::default()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_project_path_validation() {
        let server = LogLensServer::new();
        
        // Test with relative path
        let mut args = serde_json::Map::new();
        args.insert("project_path".to_string(), serde_json::Value::String(".".to_string()));
        args.insert("log_file_path".to_string(), serde_json::Value::String("test.log".to_string()));
        
        // This should fail because current directory doesn't have .loglens
        let request = CallToolRequestParam {
            name: "add_log_file".to_string(),
            arguments: Some(args),
        };
        
        let result = server.call_tool(request, Default::default()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_log_file_path_resolution() {
        let (_temp_dir, project_path) = create_test_project();
        let server = LogLensServer::new();
        
        // Test with relative log file path
        let mut args = serde_json::Map::new();
        args.insert("project_path".to_string(), serde_json::Value::String(project_path.clone()));
        args.insert("log_file_path".to_string(), serde_json::Value::String("test.log".to_string()));
        args.insert("auto_analyze".to_string(), serde_json::Value::Bool(false));
        
        let request = CallToolRequestParam {
            name: "add_log_file".to_string(),
            arguments: Some(args),
        };
        
        let result = server.call_tool(request, Default::default()).await.unwrap();
        let response: serde_json::Value = serde_json::from_str(&result.content[0].text).unwrap();
        assert!(response["success"].as_bool().unwrap());
        
        // Test with absolute log file path
        let absolute_log_path = Path::new(&project_path).join("test.log").to_str().unwrap().to_string();
        let mut args = serde_json::Map::new();
        args.insert("project_path".to_string(), serde_json::Value::String(project_path.clone()));
        args.insert("log_file_path".to_string(), serde_json::Value::String(absolute_log_path));
        args.insert("auto_analyze".to_string(), serde_json::Value::Bool(false));
        
        let request = CallToolRequestParam {
            name: "add_log_file".to_string(),
            arguments: Some(args),
        };
        
        let result = server.call_tool(request, Default::default()).await.unwrap();
        let response: serde_json::Value = serde_json::from_str(&result.content[0].text).unwrap();
        assert!(response["success"].as_bool().unwrap());
        
        // Test with nonexistent log file
        let mut args = serde_json::Map::new();
        args.insert("project_path".to_string(), serde_json::Value::String(project_path));
        args.insert("log_file_path".to_string(), serde_json::Value::String("nonexistent.log".to_string()));
        
        let request = CallToolRequestParam {
            name: "add_log_file".to_string(),
            arguments: Some(args),
        };
        
        let result = server.call_tool(request, Default::default()).await;
        assert!(result.is_err());
    }
}