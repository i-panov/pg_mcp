pub mod config;
pub mod state;
pub mod tools;

use rust_mcp_sdk::schema::schema_utils::CallToolError;
use rust_mcp_sdk::schema::*;
use sqlx::{Column, Row, TypeInfo};
use state::AppState;
use std::str::FromStr;
use std::sync::LazyLock;
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

fn row_to_json_value(
    row: &sqlx::postgres::PgRow,
    i: usize,
    col: &sqlx::postgres::PgColumn,
) -> serde_json::Value {
    use sqlx::ValueRef;

    let raw = match row.try_get_raw(i) {
        Ok(v) => v,
        Err(_) => return serde_json::Value::Null,
    };
    if raw.is_null() {
        return serde_json::Value::Null;
    }

    let type_name = col.type_info().name().to_uppercase();
    let type_str = type_name.as_str();

    fn str_val(row: &sqlx::postgres::PgRow, i: usize) -> serde_json::Value {
        row.try_get::<String, _>(i)
            .ok()
            .map(serde_json::Value::String)
            .unwrap_or(serde_json::Value::Null)
    }

    match type_str {
        "BOOL" => row
            .try_get::<bool, _>(i)
            .ok()
            .map(serde_json::Value::from)
            .unwrap_or(serde_json::Value::Null),
        "INT2" => row
            .try_get::<i16, _>(i)
            .ok()
            .map(|v| serde_json::Value::from(v as i64))
            .unwrap_or(serde_json::Value::Null),
        "INT4" => row
            .try_get::<i32, _>(i)
            .ok()
            .map(|v| serde_json::Value::from(v as i64))
            .unwrap_or(serde_json::Value::Null),
        "INT8" => row
            .try_get::<i64, _>(i)
            .ok()
            .map(serde_json::Value::from)
            .unwrap_or(serde_json::Value::Null),
        "FLOAT4" => row
            .try_get::<f32, _>(i)
            .ok()
            .map(|v| serde_json::Value::from(v as f64))
            .unwrap_or(serde_json::Value::Null),
        "FLOAT8" => row
            .try_get::<f64, _>(i)
            .ok()
            .map(serde_json::Value::from)
            .unwrap_or(serde_json::Value::Null),
        "NUMERIC" => row
            .try_get::<String, _>(i)
            .ok()
            .map(|s| {
                serde_json::Number::from_str(&s)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::String(s))
            })
            .unwrap_or(serde_json::Value::Null),
        "JSON" | "JSONB" => row
            .try_get::<serde_json::Value, _>(i)
            .ok()
            .unwrap_or(serde_json::Value::Null),
        "UUID" => row
            .try_get::<uuid::Uuid, _>(i)
            .ok()
            .map(|v| serde_json::Value::String(v.to_string()))
            .unwrap_or(serde_json::Value::Null),
        "BYTEA" => row
            .try_get::<Vec<u8>, _>(i)
            .ok()
            .map(|v| serde_json::Value::String(format!("<bytea {} bytes>", v.len())))
            .unwrap_or(serde_json::Value::Null),
        "DATE" | "TIME" | "TIMESTAMP" | "TIMESTAMPTZ" | "INTERVAL" | "MONEY" | "INET" | "CIDR"
        | "MACADDR" | "MACADDR8" | "TSVECTOR" | "TSQUERY" | "POINT" | "LINE" | "LSEG" | "BOX"
        | "PATH" | "POLYGON" | "CIRCLE" | "INT4RANGE" | "INT8RANGE" | "NUMRANGE" | "TSRANGE"
        | "TSTZRANGE" | "DATERANGE" | "XML" | "BIT" | "VARBIT" => str_val(row, i),
        "_BOOL" => row
            .try_get::<Vec<Option<bool>>, _>(i)
            .ok()
            .map(|arr| {
                serde_json::Value::Array(
                    arr.into_iter()
                        .map(|opt| {
                            opt.map(serde_json::Value::from)
                                .unwrap_or(serde_json::Value::Null)
                        })
                        .collect(),
                )
            })
            .unwrap_or(serde_json::Value::Null),
        "_INT2" => row
            .try_get::<Vec<Option<i16>>, _>(i)
            .ok()
            .map(|arr| {
                serde_json::Value::Array(
                    arr.into_iter()
                        .map(|opt| {
                            opt.map(|v| serde_json::Value::from(v as i64))
                                .unwrap_or(serde_json::Value::Null)
                        })
                        .collect(),
                )
            })
            .unwrap_or(serde_json::Value::Null),
        "_INT4" => row
            .try_get::<Vec<Option<i32>>, _>(i)
            .ok()
            .map(|arr| {
                serde_json::Value::Array(
                    arr.into_iter()
                        .map(|opt| {
                            opt.map(|v| serde_json::Value::from(v as i64))
                                .unwrap_or(serde_json::Value::Null)
                        })
                        .collect(),
                )
            })
            .unwrap_or(serde_json::Value::Null),
        "_INT8" => row
            .try_get::<Vec<Option<i64>>, _>(i)
            .ok()
            .map(|arr| {
                serde_json::Value::Array(
                    arr.into_iter()
                        .map(|opt| {
                            opt.map(serde_json::Value::from)
                                .unwrap_or(serde_json::Value::Null)
                        })
                        .collect(),
                )
            })
            .unwrap_or(serde_json::Value::Null),
        "_FLOAT4" => row
            .try_get::<Vec<Option<f32>>, _>(i)
            .ok()
            .map(|arr| {
                serde_json::Value::Array(
                    arr.into_iter()
                        .map(|opt| {
                            opt.map(|v| serde_json::Value::from(v as f64))
                                .unwrap_or(serde_json::Value::Null)
                        })
                        .collect(),
                )
            })
            .unwrap_or(serde_json::Value::Null),
        "_FLOAT8" => row
            .try_get::<Vec<Option<f64>>, _>(i)
            .ok()
            .map(|arr| {
                serde_json::Value::Array(
                    arr.into_iter()
                        .map(|opt| {
                            opt.map(serde_json::Value::from)
                                .unwrap_or(serde_json::Value::Null)
                        })
                        .collect(),
                )
            })
            .unwrap_or(serde_json::Value::Null),
        "_JSON" | "_JSONB" => row
            .try_get::<Vec<Option<serde_json::Value>>, _>(i)
            .ok()
            .map(|arr| {
                serde_json::Value::Array(
                    arr.into_iter()
                        .map(|opt| opt.unwrap_or(serde_json::Value::Null))
                        .collect(),
                )
            })
            .unwrap_or(serde_json::Value::Null),
        _ if type_str.starts_with('_') => row
            .try_get::<Vec<Option<String>>, _>(i)
            .ok()
            .map(|arr| {
                serde_json::Value::Array(
                    arr.into_iter()
                        .map(|opt| {
                            opt.map(serde_json::Value::from)
                                .unwrap_or(serde_json::Value::Null)
                        })
                        .collect(),
                )
            })
            .unwrap_or(serde_json::Value::Null),
        _ => row
            .try_get::<Option<String>, _>(i)
            .ok()
            .flatten()
            .map(serde_json::Value::from)
            .unwrap_or(serde_json::Value::Null),
    }
}

static URL_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(?i)(postgres://|postgresql://)[^:]+:[^@]+@").unwrap());

fn sanitize_sql_error(e: &sqlx::Error) -> String {
    let msg = e.to_string();
    URL_RE.replace_all(&msg, "$1<user>:<password>@").to_string()
}

fn is_valid_identifier(name: &str) -> bool {
    !name.is_empty()
        && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        && name
            .bytes()
            .next()
            .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct ArgsError(String);

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct SqlError(String);

pub async fn handle_execute_sql(
    state: &AppState,
    args: &ExecuteSqlTool,
) -> std::result::Result<CallToolResult, CallToolError> {
    let trimmed = args.sql.trim_start();
    let is_select =
        trimmed.to_uppercase().starts_with("SELECT") || trimmed.to_uppercase().starts_with("WITH");

    if is_select {
        let rows = sqlx::query(&args.sql)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| CallToolError::new(SqlError(sanitize_sql_error(&e))))?;

        if rows.is_empty() {
            return Ok(CallToolResult::text_content(vec!["0 rows returned".into()]));
        }

        let mut results: Vec<serde_json::Value> = Vec::new();
        for row in &rows {
            let mut map = serde_json::Map::new();
            let columns = row.columns();
            for (i, col) in columns.iter().enumerate() {
                let value = row_to_json_value(row, i, col);
                map.insert(col.name().to_string(), value);
            }
            results.push(serde_json::Value::Object(map));
        }
        let json = serde_json::to_string_pretty(&results).unwrap_or_default();
        Ok(CallToolResult::text_content(vec![json.into()]))
    } else {
        let result = sqlx::query(&args.sql)
            .execute(&state.pool)
            .await
            .map_err(|e| CallToolError::new(SqlError(sanitize_sql_error(&e))))?;
        Ok(CallToolResult::text_content(vec![
            format!(
                "Query executed successfully. Rows affected: {}",
                result.rows_affected()
            )
            .into(),
        ]))
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

    let rows = match sqlx::query(&args.sql).fetch_all(&mut *tx).await {
        Ok(rows) => {
            if let Err(e) = tx.rollback().await {
                eprintln!("Warning: failed to rollback read-only transaction: {}", e);
            }
            rows
        }
        Err(e) => {
            if let Err(rb_err) = tx.rollback().await {
                eprintln!(
                    "Warning: failed to rollback transaction after error: {}",
                    rb_err
                );
            }
            return Err(CallToolError::new(SqlError(sanitize_sql_error(&e))));
        }
    };

    if rows.is_empty() {
        return Ok(CallToolResult::text_content(vec!["0 rows returned".into()]));
    }

    let mut results: Vec<serde_json::Value> = Vec::new();
    for row in &rows {
        let mut map = serde_json::Map::new();
        let columns = row.columns();
        for (i, col) in columns.iter().enumerate() {
            let value = row_to_json_value(row, i, col);
            map.insert(col.name().to_string(), value);
        }
        results.push(serde_json::Value::Object(map));
    }
    let json = serde_json::to_string_pretty(&results).unwrap_or_default();
    Ok(CallToolResult::text_content(vec![json.into()]))
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
    let json = serde_json::to_string_pretty(&names).unwrap_or_default();
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
    let json = serde_json::to_string_pretty(&names).unwrap_or_default();
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
    let json = serde_json::to_string_pretty(&names).unwrap_or_default();
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
    let json = serde_json::to_string_pretty(&names).unwrap_or_default();
    Ok(CallToolResult::text_content(vec![json.into()]))
}

pub async fn handle_list_routines(
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
    let json = serde_json::to_string_pretty(&names).unwrap_or_default();
    Ok(CallToolResult::text_content(vec![json.into()]))
}

pub async fn handle_list_triggers(
    state: &AppState,
    table: Option<String>,
    schema: Option<String>,
) -> std::result::Result<CallToolResult, CallToolError> {
    let schema = schema.unwrap_or_else(|| state.default_schema.clone());
    let rows = sqlx::query(
        "SELECT t.tgname AS trigger_name, c.relname AS table_name \
         FROM pg_catalog.pg_trigger t \
         JOIN pg_catalog.pg_class c ON t.tgrelid = c.oid \
         JOIN pg_catalog.pg_namespace n ON c.relnamespace = n.oid \
         WHERE NOT t.tgisinternal \
           AND n.nspname = $1 \
           AND ($2::text IS NULL OR c.relname = $2) \
         ORDER BY t.tgname",
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
                "trigger_name": r.try_get::<String, _>(0).ok(),
                "table": r.try_get::<String, _>(1).ok(),
            })
        })
        .collect();
    let json = serde_json::to_string_pretty(&result).unwrap_or_default();
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

    if rows.is_empty() {
        return Ok(CallToolResult::text_content(vec![
            format!("No indexes found for table '{}.{}'", schema, table).into(),
        ]));
    }

    let result: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "name": r.try_get::<String, _>(0).ok(),
                "definition": r.try_get::<String, _>(1).ok(),
            })
        })
        .collect();
    let json = serde_json::to_string_pretty(&result).unwrap_or_default();
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

    if columns.is_empty() {
        return Ok(CallToolResult::text_content(vec![
            format!("Table '{}.{}' not found", schema, table).into(),
        ]));
    }

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
        "SELECT tc.constraint_name, \
                array_agg(kcu.column_name ORDER BY kcu.ordinal_position) AS columns, \
                ccu.table_name AS foreign_table, \
                array_agg(ccu.column_name ORDER BY kcu.ordinal_position) AS foreign_columns \
         FROM information_schema.table_constraints tc \
         JOIN information_schema.key_column_usage kcu \
           ON tc.constraint_name = kcu.constraint_name \
           AND tc.table_schema = kcu.table_schema \
         JOIN information_schema.constraint_column_usage ccu \
           ON ccu.constraint_name = tc.constraint_name \
         WHERE tc.constraint_type = 'FOREIGN KEY' \
           AND tc.table_schema = $1 AND tc.table_name = $2 \
         GROUP BY tc.constraint_name, ccu.table_name \
         ORDER BY tc.constraint_name",
    )
    .bind(&schema)
    .bind(table)
    .fetch_all(&state.pool)
    .await
    .map_err(CallToolError::new)?;

    let foreign_keys: Vec<serde_json::Value> = fks
        .iter()
        .map(|fk| {
            let columns: Vec<String> = fk.try_get::<Vec<String>, _>(1).unwrap_or_default();
            let foreign_columns: Vec<String> = fk.try_get::<Vec<String>, _>(3).unwrap_or_default();
            serde_json::json!({
                "constraint": fk.try_get::<String, _>(0).ok(),
                "columns": columns,
                "references_table": fk.try_get::<String, _>(2).ok(),
                "references_columns": foreign_columns,
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
    let json = serde_json::to_string_pretty(&result).unwrap_or_default();
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
    let rows = sqlx::query(
        "SELECT p.oid, \
                pg_get_function_identity_arguments(p.oid) AS args, \
                pg_get_functiondef(p.oid) AS definition \
         FROM pg_proc p \
         JOIN pg_namespace n ON p.pronamespace = n.oid \
         WHERE n.nspname = $1 AND p.proname = $2 \
         ORDER BY p.oid",
    )
    .bind(&schema)
    .bind(function)
    .fetch_all(&state.pool)
    .await
    .map_err(CallToolError::new)?;

    if rows.is_empty() {
        return Ok(CallToolResult::text_content(vec![
            format!("Function '{}' not found in schema '{}'", function, schema).into(),
        ]));
    }

    let definitions: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            let args: String = r
                .try_get::<Option<String>, _>(1)
                .ok()
                .flatten()
                .unwrap_or_default();
            let def: String = r
                .try_get::<Option<String>, _>(2)
                .ok()
                .flatten()
                .unwrap_or_else(|| "(could not read definition)".to_string());
            serde_json::json!({
                "arguments": args,
                "definition": def,
            })
        })
        .collect();

    let result = serde_json::json!({
        "function": function,
        "schema": schema,
        "overloads": definitions,
    });
    let json = serde_json::to_string_pretty(&result).unwrap_or_default();
    Ok(CallToolResult::text_content(vec![json.into()]))
}

pub async fn handle_explain_query(
    state: &AppState,
    args: &ExplainQueryTool,
) -> std::result::Result<CallToolResult, CallToolError> {
    let explain_sql = format!("EXPLAIN (FORMAT JSON) {}", args.sql);
    let rows = sqlx::query(&explain_sql)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| CallToolError::new(SqlError(sanitize_sql_error(&e))))?;

    let plans: Vec<serde_json::Value> = rows
        .iter()
        .filter_map(|r| r.try_get::<serde_json::Value, _>(0).ok())
        .collect();

    let json = serde_json::to_string_pretty(&plans).unwrap_or_default();
    Ok(CallToolResult::text_content(vec![json.into()]))
}

pub async fn handle_get_table_size(
    state: &AppState,
    table: &str,
    schema: Option<String>,
) -> std::result::Result<CallToolResult, CallToolError> {
    let schema = schema.unwrap_or_else(|| state.default_schema.clone());
    if !is_valid_identifier(&schema) || !is_valid_identifier(table) {
        return Err(CallToolError::new(ArgsError(format!(
            "Invalid schema or table name: {}.{}",
            schema, table
        ))));
    }
    let qualified = format!("{}.{}", schema, table);
    let row = sqlx::query(
        "SELECT \
            pg_relation_size($1::regclass) AS table_size, \
            pg_indexes_size($1::regclass) AS indexes_size, \
            pg_total_relation_size($1::regclass) AS total_size, \
            pg_size_pretty(pg_relation_size($1::regclass)) AS table_size_pretty, \
            pg_size_pretty(pg_indexes_size($1::regclass)) AS indexes_size_pretty, \
            pg_size_pretty(pg_total_relation_size($1::regclass)) AS total_size_pretty",
    )
    .bind(&qualified)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| CallToolError::new(SqlError(sanitize_sql_error(&e))))?;

    let result = serde_json::json!({
        "table": table,
        "schema": schema,
        "table_size_bytes": row.try_get::<Option<i64>, _>(0).ok().flatten().unwrap_or(0),
        "indexes_size_bytes": row.try_get::<Option<i64>, _>(1).ok().flatten().unwrap_or(0),
        "total_size_bytes": row.try_get::<Option<i64>, _>(2).ok().flatten().unwrap_or(0),
        "table_size": row.try_get::<Option<String>, _>(3).ok().flatten().unwrap_or_default(),
        "indexes_size": row.try_get::<Option<String>, _>(4).ok().flatten().unwrap_or_default(),
        "total_size": row.try_get::<Option<String>, _>(5).ok().flatten().unwrap_or_default(),
    });
    let json = serde_json::to_string_pretty(&result).unwrap_or_default();
    Ok(CallToolResult::text_content(vec![json.into()]))
}

pub async fn handle_list_extensions(
    state: &AppState,
) -> std::result::Result<CallToolResult, CallToolError> {
    let rows = sqlx::query(
        "SELECT extname, extversion, extrelocatable, \
                extnamespace::regnamespace AS schema_name \
         FROM pg_extension ORDER BY extname",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(CallToolError::new)?;

    let result: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "name": r.try_get::<String, _>(0).ok(),
                "version": r.try_get::<String, _>(1).ok(),
                "relocatable": r.try_get::<Option<bool>, _>(2).ok().flatten().unwrap_or(false),
                "schema": r.try_get::<String, _>(3).ok(),
            })
        })
        .collect();
    let json = serde_json::to_string_pretty(&result).unwrap_or_default();
    Ok(CallToolResult::text_content(vec![json.into()]))
}

pub async fn handle_list_sequences(
    state: &AppState,
    schema: Option<String>,
) -> std::result::Result<CallToolResult, CallToolError> {
    let schema = schema.unwrap_or_else(|| state.default_schema.clone());
    let rows = sqlx::query(
        "SELECT sequence_schema, sequence_name, data_type, \
                start_value, minimum_value, maximum_value, increment \
         FROM information_schema.sequences \
         WHERE sequence_schema = $1 \
         ORDER BY sequence_name",
    )
    .bind(&schema)
    .fetch_all(&state.pool)
    .await
    .map_err(CallToolError::new)?;

    let result: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "schema": r.try_get::<String, _>(0).ok(),
                "name": r.try_get::<String, _>(1).ok(),
                "data_type": r.try_get::<String, _>(2).ok(),
                "start_value": r.try_get::<Option<String>, _>(3).ok().flatten(),
                "min_value": r.try_get::<Option<String>, _>(4).ok().flatten(),
                "max_value": r.try_get::<Option<String>, _>(5).ok().flatten(),
                "increment": r.try_get::<Option<String>, _>(6).ok().flatten(),
            })
        })
        .collect();
    let json = serde_json::to_string_pretty(&result).unwrap_or_default();
    Ok(CallToolResult::text_content(vec![json.into()]))
}

pub async fn handle_get_table_row_count(
    state: &AppState,
    table: &str,
    schema: Option<String>,
    approximate: bool,
) -> std::result::Result<CallToolResult, CallToolError> {
    let schema = schema.unwrap_or_else(|| state.default_schema.clone());

    if approximate {
        let row = sqlx::query(
            "SELECT reltuples::bigint AS estimate \
             FROM pg_class c \
             JOIN pg_namespace n ON c.relnamespace = n.oid \
             WHERE n.nspname = $1 AND c.relname = $2",
        )
        .bind(&schema)
        .bind(table)
        .fetch_optional(&state.pool)
        .await
        .map_err(CallToolError::new)?;

        match row {
            Some(r) => {
                let estimate: Option<i64> = r.try_get::<Option<i64>, _>(0).ok().flatten();
                let result = serde_json::json!({
                    "table": table,
                    "schema": schema,
                    "approximate": true,
                    "count": estimate.unwrap_or(0),
                });
                let json = serde_json::to_string_pretty(&result).unwrap_or_default();
                Ok(CallToolResult::text_content(vec![json.into()]))
            }
            None => Ok(CallToolResult::text_content(vec![
                format!("Table '{}.{}' not found", schema, table).into(),
            ])),
        }
    } else {
        if !is_valid_identifier(&schema) || !is_valid_identifier(table) {
            return Err(CallToolError::new(ArgsError(format!(
                "Invalid schema or table name: {}.{}",
                schema, table
            ))));
        }
        let row = sqlx::query(&format!(
            "SELECT COUNT(*) AS count FROM {}.{}",
            schema, table
        ))
        .fetch_one(&state.pool)
        .await
        .map_err(|e| CallToolError::new(SqlError(sanitize_sql_error(&e))))?;

        let count: i64 = row.try_get::<i64, _>(0).unwrap_or(0);
        let result = serde_json::json!({
            "table": table,
            "schema": schema,
            "approximate": false,
            "count": count,
        });
        let json = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::text_content(vec![json.into()]))
    }
}

pub async fn handle_list_active_queries(
    state: &AppState,
) -> std::result::Result<CallToolResult, CallToolError> {
    let rows = sqlx::query(
        "SELECT pid, usename, datname, state, query, backend_start, query_start \
         FROM pg_stat_activity \
         WHERE state != 'idle' AND pid != pg_backend_pid() \
         ORDER BY query_start",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(CallToolError::new)?;

    let result: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "pid": r.try_get::<Option<i32>, _>(0).ok().flatten(),
                "user": r.try_get::<Option<String>, _>(1).ok().flatten(),
                "database": r.try_get::<Option<String>, _>(2).ok().flatten(),
                "state": r.try_get::<Option<String>, _>(3).ok().flatten(),
                "query": r.try_get::<Option<String>, _>(4).ok().flatten(),
                "backend_start": r.try_get::<Option<String>, _>(5).ok().flatten(),
                "query_start": r.try_get::<Option<String>, _>(6).ok().flatten(),
            })
        })
        .collect();

    if result.is_empty() {
        return Ok(CallToolResult::text_content(vec![
            "No active queries".into(),
        ]));
    }

    let json = serde_json::to_string_pretty(&result).unwrap_or_default();
    Ok(CallToolResult::text_content(vec![json.into()]))
}

pub async fn handle_list_locks(
    state: &AppState,
) -> std::result::Result<CallToolResult, CallToolError> {
    let rows = sqlx::query(
        "SELECT l.pid, l.mode, l.granted, l.locktype, \
                c.relname AS table_name, a.query, a.usename, a.wait_event_type, a.wait_event \
         FROM pg_locks l \
         LEFT JOIN pg_class c ON l.relation = c.oid \
         LEFT JOIN pg_stat_activity a ON l.pid = a.pid \
         ORDER BY l.granted, l.pid",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(CallToolError::new)?;

    let result: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "pid": r.try_get::<Option<i32>, _>(0).ok().flatten(),
                "mode": r.try_get::<Option<String>, _>(1).ok().flatten(),
                "granted": r.try_get::<Option<bool>, _>(2).ok().flatten(),
                "locktype": r.try_get::<Option<String>, _>(3).ok().flatten(),
                "table": r.try_get::<Option<String>, _>(4).ok().flatten(),
                "query": r.try_get::<Option<String>, _>(5).ok().flatten(),
                "user": r.try_get::<Option<String>, _>(6).ok().flatten(),
                "wait_event_type": r.try_get::<Option<String>, _>(7).ok().flatten(),
                "wait_event": r.try_get::<Option<String>, _>(8).ok().flatten(),
            })
        })
        .collect();

    if result.is_empty() {
        return Ok(CallToolResult::text_content(vec!["No locks found".into()]));
    }

    let json = serde_json::to_string_pretty(&result).unwrap_or_default();
    Ok(CallToolResult::text_content(vec![json.into()]))
}
