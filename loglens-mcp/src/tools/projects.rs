use serde_json::Value;
use crate::Database;
use anyhow::Result;
use sqlx::Row;

/// List available projects with optional name filtering
pub async fn list_projects(db: &Database, params: Value) -> Result<Value> {
    let names: Option<Vec<String>> = params.get("names")
        .and_then(|v| serde_json::from_value(v.clone()).ok());

    let mut query = String::from(
        "SELECT p.id, p.name, p.description, p.created_at, p.updated_at,
         COUNT(DISTINCT f.id) as file_count,
         COUNT(DISTINCT a.id) as analysis_count,
         MAX(a.created_at) as last_analysis_date
         FROM projects p
         LEFT JOIN log_files f ON p.id = f.project_id
         LEFT JOIN analyses a ON p.id = a.project_id"
    );

    let mut binds = Vec::new();

    if let Some(names) = &names {
        let placeholders = names.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        query.push_str(&format!(" WHERE p.name IN ({})", placeholders));
        for name in names {
            binds.push(name.clone());
        }
    }

    query.push_str(" GROUP BY p.id, p.name, p.description, p.created_at, p.updated_at");

    let mut query_builder = sqlx::query(&query);
    for bind in &binds {
        query_builder = query_builder.bind(bind);
    }

    let rows = query_builder.fetch_all(&db.pool).await?;

    let projects: Vec<Value> = rows.into_iter().map(|row| {
        let id: String = row.get("id");
        let name: String = row.get("name");
        let description: Option<String> = row.get("description");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");
        let file_count: Option<i64> = row.get("file_count");
        let analysis_count: Option<i64> = row.get("analysis_count");
        let last_analysis_date: Option<chrono::DateTime<chrono::Utc>> = row.get("last_analysis_date");

        serde_json::json!({
            "id": id,
            "name": name,
            "description": description,
            "file_count": file_count.unwrap_or(0),
            "analysis_count": analysis_count.unwrap_or(0),
            "last_analysis_date": last_analysis_date,
            "created_at": created_at,
            "updated_at": updated_at
        })
    }).collect();

    Ok(serde_json::Value::Array(projects))
}

/// Get detailed project information
pub async fn get_project(db: &Database, params: Value) -> Result<Value> {
    let project_id: String = serde_json::from_value(params["project_id"].clone())
        .map_err(|_| anyhow::anyhow!("Invalid project_id parameter"))?;

    let row = sqlx::query(
        "SELECT p.id, p.name, p.description, p.created_at, p.updated_at,
         COUNT(DISTINCT f.id) as file_count,
         COUNT(DISTINCT a.id) as analysis_count,
         MAX(a.created_at) as last_analysis_date
         FROM projects p
         LEFT JOIN log_files f ON p.id = f.project_id
         LEFT JOIN analyses a ON p.id = a.project_id
         WHERE p.id = ?
         GROUP BY p.id, p.name, p.description, p.created_at, p.updated_at"
    )
    .bind(&project_id)
    .fetch_optional(&db.pool)
    .await?;

    match row {
        Some(row) => {
            let id: String = row.get("id");
            let name: String = row.get("name");
            let description: Option<String> = row.get("description");
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
            let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");
            let file_count: Option<i64> = row.get("file_count");
            let analysis_count: Option<i64> = row.get("analysis_count");
            let last_analysis_date: Option<chrono::DateTime<chrono::Utc>> = row.get("last_analysis_date");

            Ok(serde_json::json!({
                "id": id,
                "name": name,
                "description": description,
                "file_count": file_count.unwrap_or(0),
                "analysis_count": analysis_count.unwrap_or(0),
                "last_analysis_date": last_analysis_date,
                "created_at": created_at,
                "updated_at": updated_at
            }))
        }
        None => Err(anyhow::anyhow!("Project not found: {}", project_id))
    }
}