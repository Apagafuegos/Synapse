use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{query, SqlitePool};
use uuid::Uuid;

use crate::project::models::{Analysis, AnalysisResult, AnalysisStatus, Pattern, Project};

/// Create new analysis record
pub async fn create_analysis(
    pool: &SqlitePool,
    project_id: String,
    log_file_path: String,
    provider: String,
    level: String,
) -> Result<String> {
    let analysis_id = Uuid::new_v4().to_string();
    let status_str = AnalysisStatus::Pending.to_string();
    let created_at = Utc::now();

    query!(
        r#"
        INSERT INTO analyses (id, project_id, log_file_path, provider, level, status, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
        analysis_id,
        project_id,
        log_file_path,
        provider,
        level,
        status_str,
        created_at
    )
    .execute(pool)
    .await?;

    Ok(analysis_id)
}

/// Retrieve analysis with optional results
pub async fn get_analysis_by_id(
    pool: &SqlitePool,
    analysis_id: &str,
) -> Result<Option<(Analysis, Option<AnalysisResult>)>> {
    // First get the analysis (using query_as instead of query_as! to use custom FromRow impl)
    let analysis = sqlx::query_as::<_, Analysis>(
        "SELECT id, project_id, log_file_path, provider, level, status,
               created_at, started_at, completed_at, metadata
        FROM analyses
        WHERE id = ?1"
    )
    .bind(analysis_id)
    .fetch_optional(pool)
    .await?;

    if let Some(analysis) = analysis {
        // Try to get the analysis result (using query_as instead of query_as! to use custom FromRow impl)
        let result = sqlx::query_as::<_, AnalysisResult>(
            "SELECT analysis_id, summary, full_report, patterns_detected, issues_found, metadata
            FROM analysis_results
            WHERE analysis_id = ?1"
        )
        .bind(analysis_id)
        .fetch_optional(pool)
        .await?;

        Ok(Some((analysis, result)))
    } else {
        Ok(None)
    }
}

/// Query analyses with filters
pub async fn query_analyses(
    pool: &SqlitePool,
    project_id: Option<&str>,
    status: Option<AnalysisStatus>,
    limit: Option<i64>,
    since: Option<DateTime<Utc>>,
) -> Result<Vec<Analysis>> {
    let mut query_builder = sqlx::QueryBuilder::new(
        "SELECT id, project_id, log_file_path, provider, level, status, \
         created_at, started_at, completed_at, metadata \
         FROM analyses \
         WHERE 1=1"
    );

    if let Some(project_id) = project_id {
        query_builder.push(" AND project_id = ");
        query_builder.push_bind(project_id);
    }

    if let Some(status) = status {
        query_builder.push(" AND status = ");
        query_builder.push_bind(status.to_string());
    }

    if let Some(since) = since {
        query_builder.push(" AND created_at >= ");
        query_builder.push_bind(since);
    }

    query_builder.push(" ORDER BY created_at DESC");

    if let Some(limit) = limit {
        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
    }

    let analyses = query_builder
        .build_query_as::<Analysis>()
        .fetch_all(pool)
        .await?;

    Ok(analyses)
}

/// Store analysis results
pub async fn store_analysis_results(
    pool: &SqlitePool,
    analysis_id: &str,
    summary: Option<String>,
    full_report: Option<String>,
    patterns: Vec<Pattern>,
    issues_found: Option<i64>,
) -> Result<()> {
    let patterns_json = serde_json::to_value(patterns)?;
    
    query!(
        r#"
        INSERT INTO analysis_results (analysis_id, summary, full_report, patterns_detected, issues_found)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(analysis_id) DO UPDATE SET
            summary = excluded.summary,
            full_report = excluded.full_report,
            patterns_detected = excluded.patterns_detected,
            issues_found = excluded.issues_found
        "#,
        analysis_id,
        summary,
        full_report,
        patterns_json,
        issues_found
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Update analysis status
pub async fn update_analysis_status(
    pool: &SqlitePool,
    analysis_id: &str,
    status: AnalysisStatus,
    completed_at: Option<DateTime<Utc>>,
) -> Result<()> {
    let status_str = status.to_string();
    query!(
        r#"
        UPDATE analyses
        SET status = ?1, completed_at = ?2
        WHERE id = ?3
        "#,
        status_str,
        completed_at,
        analysis_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Resolve project by path
pub async fn get_project_by_path(
    pool: &SqlitePool,
    root_path: &str,
) -> Result<Option<Project>> {
    let project = sqlx::query_as::<_, Project>(
        "SELECT id, name, root_path, description, created_at, updated_at, metadata
        FROM projects
        WHERE root_path = ?1"
    )
    .bind(root_path)
    .fetch_optional(pool)
    .await?;

    Ok(project)
}

/// Create or get project by path
pub async fn get_or_create_project(
    pool: &SqlitePool,
    root_path: &str,
) -> Result<String> {
    // Try to get existing project
    if let Some(project) = get_project_by_path(pool, root_path).await? {
        return Ok(project.id);
    }

    // Create new project
    let project_id = Uuid::new_v4().to_string();
    let name = std::path::Path::new(root_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let now = Utc::now();
    query!(
        r#"
        INSERT INTO projects (id, name, root_path, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
        project_id,
        name,
        root_path,
        now,
        now
    )
    .execute(pool)
    .await?;

    Ok(project_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use tempfile::TempDir;
    use std::path::Path;

    async fn setup_test_db() -> (SqlitePool, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());
        
        let pool = SqlitePool::connect(&db_url).await.unwrap();
        
        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .unwrap();
        
        (pool, temp_dir)
    }

    #[tokio::test]
    async fn test_create_analysis() {
        let (pool, _temp) = setup_test_db().await;
        
        let project_id = "test-project".to_string();
        let log_file_path = "/test/path.log".to_string();
        let provider = "openrouter".to_string();
        let level = "ERROR".to_string();

        let analysis_id = create_analysis(
            &pool,
            project_id.clone(),
            log_file_path.clone(),
            provider.clone(),
            level.clone(),
        ).await.unwrap();

        assert!(!analysis_id.is_empty());

        // Verify the analysis was created
        let analysis = query_as!(
            Analysis,
            "SELECT * FROM analyses WHERE id = ?1",
            analysis_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(analysis.project_id, project_id);
        assert_eq!(analysis.log_file_path, log_file_path);
        assert_eq!(analysis.provider, provider);
        assert_eq!(analysis.level, level);
        assert_eq!(analysis.status, AnalysisStatus::Pending.to_string());
    }

    #[tokio::test]
    async fn test_get_analysis_by_id() {
        let (pool, _temp) = setup_test_db().await;
        
        // Create an analysis first
        let analysis_id = create_analysis(
            &pool,
            "test-project".to_string(),
            "/test/path.log".to_string(),
            "openrouter".to_string(),
            "ERROR".to_string(),
        ).await.unwrap();

        // Get the analysis
        let result = get_analysis_by_id(&pool, &analysis_id).await.unwrap();
        assert!(result.is_some());
        
        let (analysis, result_opt) = result.unwrap();
        assert_eq!(analysis.id, analysis_id);
        assert_eq!(analysis.project_id, "test-project");
        assert!(result_opt.is_none()); // No results yet

        // Test non-existent analysis
        let result = get_analysis_by_id(&pool, "non-existent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_query_analyses_with_filters() {
        let (pool, _temp) = setup_test_db().await;
        
        let project_id1 = "project1".to_string();
        let project_id2 = "project2".to_string();

        // Create multiple analyses
        let _id1 = create_analysis(&pool, project_id1.clone(), "/test1.log".to_string(), "openrouter".to_string(), "ERROR".to_string()).await.unwrap();
        let id2 = create_analysis(&pool, project_id1.clone(), "/test2.log".to_string(), "openai".to_string(), "WARN".to_string()).await.unwrap();
        let _id3 = create_analysis(&pool, project_id2.clone(), "/test3.log".to_string(), "claude".to_string(), "INFO".to_string()).await.unwrap();

        // Update one analysis to completed
        update_analysis_status(&pool, &id2, AnalysisStatus::Completed, Some(Utc::now())).await.unwrap();

        // Test query by project_id
        let analyses = query_analyses(&pool, Some(&project_id1), None, None, None).await.unwrap();
        assert_eq!(analyses.len(), 2);

        // Test query by status
        let analyses = query_analyses(&pool, None, Some(AnalysisStatus::Pending), None, None).await.unwrap();
        assert_eq!(analyses.len(), 2);

        let analyses = query_analyses(&pool, None, Some(AnalysisStatus::Completed), None, None).await.unwrap();
        assert_eq!(analyses.len(), 1);

        // Test limit
        let analyses = query_analyses(&pool, None, None, Some(1), None).await.unwrap();
        assert_eq!(analyses.len(), 1);
    }

    #[tokio::test]
    async fn test_store_analysis_results() {
        let (pool, _temp) = setup_test_db().await;
        
        let analysis_id = create_analysis(
            &pool,
            "test-project".to_string(),
            "/test/path.log".to_string(),
            "openrouter".to_string(),
            "ERROR".to_string(),
        ).await.unwrap();

        let summary = Some("Test summary".to_string());
        let full_report = Some("Test full report".to_string());
        let patterns = vec![
            Pattern {
                pattern: "Error pattern".to_string(),
                count: 5,
                examples: vec!["Error example 1".to_string()],
                severity: "high".to_string(),
                confidence: 0.9,
            }
        ];
        let issues_found = Some(3);

        store_analysis_results(
            &pool,
            &analysis_id,
            summary.clone(),
            full_report.clone(),
            patterns.clone(),
            issues_found,
        ).await.unwrap();

        // Verify results were stored
        let result = query_as!(
            AnalysisResult,
            "SELECT * FROM analysis_results WHERE analysis_id = ?1",
            analysis_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(result.summary, summary);
        assert_eq!(result.full_report, full_report);
        assert_eq!(result.issues_found, issues_found);
        
        let stored_patterns: Vec<Pattern> = serde_json::from_value(result.patterns_detected).unwrap();
        assert_eq!(stored_patterns.len(), 1);
        assert_eq!(stored_patterns[0].pattern, "Error pattern");
    }

    #[tokio::test]
    async fn test_update_analysis_status() {
        let (pool, _temp) = setup_test_db().await;
        
        let analysis_id = create_analysis(
            &pool,
            "test-project".to_string(),
            "/test/path.log".to_string(),
            "openrouter".to_string(),
            "ERROR".to_string(),
        ).await.unwrap();

        // Update to completed
        let completed_at = Utc::now();
        update_analysis_status(&pool, &analysis_id, AnalysisStatus::Completed, Some(completed_at)).await.unwrap();

        // Verify status was updated
        let analysis = query_as!(
            Analysis,
            "SELECT * FROM analyses WHERE id = ?1",
            analysis_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(analysis.status, AnalysisStatus::Completed.to_string());
        assert!(analysis.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_get_or_create_project() {
        let (pool, _temp) = setup_test_db().await;
        
        let root_path = "/test/project".to_string();

        // Create new project
        let project_id1 = get_or_create_project(&pool, &root_path).await.unwrap();
        assert!(!project_id1.is_empty());

        // Get existing project
        let project_id2 = get_or_create_project(&pool, &root_path).await.unwrap();
        assert_eq!(project_id1, project_id2);

        // Verify only one project exists
        let count = query_as!(
            (i64,),
            "SELECT COUNT(*) as count FROM projects WHERE root_path = ?1",
            root_path
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        
        assert_eq!(count.0, 1);
    }
}