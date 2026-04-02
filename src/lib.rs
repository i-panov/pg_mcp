pub mod config;
pub mod error;
pub mod state;
pub mod tools;

use rust_mcp_sdk::schema::schema_utils::CallToolError;
use rust_mcp_sdk::schema::*;
use sqlx::{Column, Row};
use state::AppState;
use tools::*;

pub fn parse_args<T: for<'de> serde::Deserialize<'de>>(
    arguments: &Option<serde_json::Map<String, serde_json::Value>>,
) -> std::result::Result<T, CallToolError> {
    let args = arguments
        .as_ref()
        .ok_or_else(|| CallToolError::new(ArgsError("Missing arguments".to_string())))?;
    serde_json::from_value(serde_json::Value::Object(args.clone()))
        .map_err(|e| CallToolError::new(ArgsError(format!("Invalid arguments: {}", e))))
}

#[derive(Debug)]
struct ArgsError(String);

impl std::fmt::Display for ArgsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ArgsError {}

pub async fn handle_execute_sql(
    state: &AppState,
    args: &ExecuteSqlTool,
) -> std::result::Result<CallToolResult, CallToolError> {
    match sqlx::query(&args.sql).execute(&state.pool).await {
        Ok(result) => Ok(CallToolResult::text_content(vec![
            format!(
                "Query executed successfully. Rows affected: {}",
                result.rows_affected()
            )
            .into(),
        ])),
        Err(e) => Ok(CallToolResult::text_content(vec![
            format!("Error: {}", e).into(),
        ])),
    }
}

pub async fn handle_execute_query(
    state: &AppState,
    args: &ExecuteQueryTool,
) -> std::result::Result<CallToolResult, CallToolError> {
    let mut tx = state.pool.begin().await.map_err(CallToolError::new)?;

    sqlx::query("SET TRANSACTION READ ONLY")
        .execute(&mut *tx)
        .await
        .map_err(CallToolError::new)?;

    match sqlx::query(&args.sql).fetch_all(&mut *tx).await {
        Ok(rows) => {
            let _ = tx.rollback().await;
            if rows.is_empty() {
                return Ok(CallToolResult::text_content(vec!["0 rows returned".into()]));
            }
            let mut results: Vec<serde_json::Value> = Vec::new();
            for row in &rows {
                let mut map = serde_json::Map::new();
                let columns = row.columns();
                for (i, col) in columns.iter().enumerate() {
                    let value: serde_json::Value = row
                        .try_get::<Option<String>, _>(i)
                        .ok()
                        .flatten()
                        .map(serde_json::Value::from)
                        .unwrap_or(serde_json::Value::Null);
                    map.insert(col.name().to_string(), value);
                }
                results.push(serde_json::Value::Object(map));
            }
            let json = serde_json::to_string_pretty(&results).unwrap_or_default();
            Ok(CallToolResult::text_content(vec![json.into()]))
        }
        Err(e) => {
            let _ = tx.rollback().await;
            Ok(CallToolResult::text_content(vec![
                format!("Query error: {}", e).into(),
            ]))
        }
    }
}

pub async fn handle_list_schemas(
    state: &AppState,
) -> std::result::Result<CallToolResult, CallToolError> {
    let rows =
        sqlx::query("SELECT schema_name FROM information_schema.schemata ORDER BY schema_name")
            .fetch_all(&state.pool)
            .await
            .map_err(CallToolError::new)?;

    let names: Vec<String> = rows
        .iter()
        .filter_map(|r| r.try_get::<String, _>(0).ok())
        .collect();
    let json = serde_json::to_string_pretty(&names).unwrap();
    Ok(CallToolResult::text_content(vec![json.into()]))
}

pub async fn handle_list_tables(
    state: &AppState,
    schema: Option<String>,
) -> std::result::Result<CallToolResult, CallToolError> {
    let schema = schema.unwrap_or_else(|| state.default_schema.clone());
    let rows = sqlx::query(
        "SELECT table_name FROM information_schema.tables WHERE table_schema = $1 AND table_type = 'BASE TABLE' ORDER BY table_name"
    )
    .bind(&schema)
    .fetch_all(&state.pool)
    .await
    .map_err(CallToolError::new)?;

    let names: Vec<String> = rows
        .iter()
        .filter_map(|r| r.try_get::<String, _>(0).ok())
        .collect();
    let json = serde_json::to_string_pretty(&names).unwrap();
    Ok(CallToolResult::text_content(vec![json.into()]))
}

pub async fn handle_list_views(
    state: &AppState,
    schema: Option<String>,
) -> std::result::Result<CallToolResult, CallToolError> {
    let schema = schema.unwrap_or_else(|| state.default_schema.clone());
    let rows = sqlx::query(
        "SELECT table_name FROM information_schema.views WHERE table_schema = $1 ORDER BY table_name",
    )
    .bind(&schema)
    .fetch_all(&state.pool)
    .await
    .map_err(CallToolError::new)?;

    let names: Vec<String> = rows
        .iter()
        .filter_map(|r| r.try_get::<String, _>(0).ok())
        .collect();
    let json = serde_json::to_string_pretty(&names).unwrap();
    Ok(CallToolResult::text_content(vec![json.into()]))
}

pub async fn handle_list_materialized_views(
    state: &AppState,
    schema: Option<String>,
) -> std::result::Result<CallToolResult, CallToolError> {
    let schema = schema.unwrap_or_else(|| state.default_schema.clone());
    let rows = sqlx::query(
        "SELECT matviewname FROM pg_matviews WHERE schemaname = $1 ORDER BY matviewname",
    )
    .bind(&schema)
    .fetch_all(&state.pool)
    .await
    .map_err(CallToolError::new)?;

    let names: Vec<String> = rows
        .iter()
        .filter_map(|r| r.try_get::<String, _>(0).ok())
        .collect();
    let json = serde_json::to_string_pretty(&names).unwrap();
    Ok(CallToolResult::text_content(vec![json.into()]))
}

pub async fn handle_list_procedures(
    state: &AppState,
    schema: Option<String>,
) -> std::result::Result<CallToolResult, CallToolError> {
    let schema = schema.unwrap_or_else(|| state.default_schema.clone());
    let rows = sqlx::query(
        "SELECT routine_name FROM information_schema.routines WHERE routine_schema = $1 ORDER BY routine_name",
    )
    .bind(&schema)
    .fetch_all(&state.pool)
    .await
    .map_err(CallToolError::new)?;

    let names: Vec<String> = rows
        .iter()
        .filter_map(|r| r.try_get::<String, _>(0).ok())
        .collect();
    let json = serde_json::to_string_pretty(&names).unwrap();
    Ok(CallToolResult::text_content(vec![json.into()]))
}

pub async fn handle_list_triggers(
    state: &AppState,
    table: Option<String>,
    schema: Option<String>,
) -> std::result::Result<CallToolResult, CallToolError> {
    let schema = schema.unwrap_or_else(|| state.default_schema.clone());
    let rows = if let Some(ref table) = table {
        sqlx::query(
            "SELECT trigger_name, event_object_table FROM information_schema.triggers WHERE trigger_schema = $1 AND event_object_table = $2 ORDER BY trigger_name"
        )
        .bind(&schema)
        .bind(table)
        .fetch_all(&state.pool)
        .await
    } else {
        sqlx::query(
            "SELECT trigger_name, event_object_table FROM information_schema.triggers WHERE trigger_schema = $1 ORDER BY trigger_name"
        )
        .bind(&schema)
        .fetch_all(&state.pool)
        .await
    }
    .map_err(CallToolError::new)?;

    let result: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "trigger_name": r.try_get::<String, _>(0).ok(),
                "table": r.try_get::<String, _>(1).ok(),
            })
        })
        .collect();
    let json = serde_json::to_string_pretty(&result).unwrap();
    Ok(CallToolResult::text_content(vec![json.into()]))
}

pub async fn handle_list_indexes(
    state: &AppState,
    table: &str,
    schema: Option<String>,
) -> std::result::Result<CallToolResult, CallToolError> {
    let schema = schema.unwrap_or_else(|| state.default_schema.clone());
    let rows = sqlx::query(
        "SELECT indexname, indexdef FROM pg_indexes WHERE schemaname = $1 AND tablename = $2 ORDER BY indexname",
    )
    .bind(&schema)
    .bind(table)
    .fetch_all(&state.pool)
    .await
    .map_err(CallToolError::new)?;

    let result: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "name": r.try_get::<String, _>(0).ok(),
                "definition": r.try_get::<String, _>(1).ok(),
            })
        })
        .collect();
    let json = serde_json::to_string_pretty(&result).unwrap();
    Ok(CallToolResult::text_content(vec![json.into()]))
}

pub async fn handle_get_table_structure(
    state: &AppState,
    table: &str,
    schema: Option<String>,
) -> std::result::Result<CallToolResult, CallToolError> {
    let schema = schema.unwrap_or_else(|| state.default_schema.clone());

    let columns = sqlx::query(
        "SELECT column_name, data_type, is_nullable, column_default FROM information_schema.columns WHERE table_schema = $1 AND table_name = $2 ORDER BY ordinal_position"
    )
    .bind(&schema)
    .bind(table)
    .fetch_all(&state.pool)
    .await
    .map_err(CallToolError::new)?;

    let cols: Vec<serde_json::Value> = columns
        .iter()
        .map(|c| {
            serde_json::json!({
                "name": c.try_get::<String, _>(0).ok(),
                "type": c.try_get::<String, _>(1).ok(),
                "nullable": c.try_get::<String, _>(2).ok().map(|v| v == "YES"),
                "default": c.try_get::<Option<String>, _>(3).ok().flatten(),
            })
        })
        .collect();

    let constraints = sqlx::query(
        "SELECT constraint_name, constraint_type FROM information_schema.table_constraints WHERE table_schema = $1 AND table_name = $2 ORDER BY constraint_name"
    )
    .bind(&schema)
    .bind(table)
    .fetch_all(&state.pool)
    .await
    .map_err(CallToolError::new)?;

    let cons: Vec<serde_json::Value> = constraints
        .iter()
        .map(|c| {
            serde_json::json!({
                "name": c.try_get::<String, _>(0).ok(),
                "type": c.try_get::<String, _>(1).ok(),
            })
        })
        .collect();

    let fks = sqlx::query(
        "SELECT tc.constraint_name, kcu.column_name, ccu.table_name AS foreign_table, ccu.column_name AS foreign_column \
         FROM information_schema.table_constraints tc \
         JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name \
         JOIN information_schema.constraint_column_usage ccu ON ccu.constraint_name = tc.constraint_name \
         WHERE tc.constraint_type = 'FOREIGN KEY' AND tc.table_schema = $1 AND tc.table_name = $2"
    )
    .bind(&schema)
    .bind(table)
    .fetch_all(&state.pool)
    .await
    .map_err(CallToolError::new)?;

    let foreign_keys: Vec<serde_json::Value> = fks
        .iter()
        .map(|fk| {
            serde_json::json!({
                "constraint": fk.try_get::<String, _>(0).ok(),
                "column": fk.try_get::<String, _>(1).ok(),
                "references_table": fk.try_get::<String, _>(2).ok(),
                "references_column": fk.try_get::<String, _>(3).ok(),
            })
        })
        .collect();

    let result = serde_json::json!({
        "table": table,
        "schema": schema,
        "columns": cols,
        "constraints": cons,
        "foreign_keys": foreign_keys,
    });
    let json = serde_json::to_string_pretty(&result).unwrap();
    Ok(CallToolResult::text_content(vec![json.into()]))
}

pub async fn handle_get_view_definition(
    state: &AppState,
    view: &str,
    schema: Option<String>,
) -> std::result::Result<CallToolResult, CallToolError> {
    let schema = schema.unwrap_or_else(|| state.default_schema.clone());
    let row = sqlx::query(
        "SELECT view_definition FROM information_schema.views WHERE table_schema = $1 AND table_name = $2",
    )
    .bind(&schema)
    .bind(view)
    .fetch_optional(&state.pool)
    .await
    .map_err(CallToolError::new)?;

    match row {
        Some(r) => {
            let def: String = r
                .try_get::<Option<String>, _>(0)
                .ok()
                .flatten()
                .unwrap_or_else(|| "(could not read definition)".to_string());
            Ok(CallToolResult::text_content(vec![def.into()]))
        }
        None => Ok(CallToolResult::text_content(vec![
            format!("View '{}' not found in schema '{}'", view, schema).into(),
        ])),
    }
}

pub async fn handle_get_function_definition(
    state: &AppState,
    function: &str,
    schema: Option<String>,
) -> std::result::Result<CallToolResult, CallToolError> {
    let schema = schema.unwrap_or_else(|| state.default_schema.clone());
    let row = sqlx::query(
        "SELECT pg_get_functiondef(p.oid) AS definition FROM pg_proc p JOIN pg_namespace n ON p.pronamespace = n.oid WHERE n.nspname = $1 AND p.proname = $2 LIMIT 1"
    )
    .bind(&schema)
    .bind(function)
    .fetch_optional(&state.pool)
    .await
    .map_err(CallToolError::new)?;

    match row {
        Some(r) => {
            let def: String = r
                .try_get::<Option<String>, _>(0)
                .ok()
                .flatten()
                .unwrap_or_else(|| "(could not read definition)".to_string());
            Ok(CallToolResult::text_content(vec![def.into()]))
        }
        None => Ok(CallToolResult::text_content(vec![
            format!("Function '{}' not found in schema '{}'", function, schema).into(),
        ])),
    }
}
