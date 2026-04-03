use pg_mcp::config::{Config, PermissionMode};
use pg_mcp::state::AppState;
use pg_mcp::tools::*;
use pg_mcp::*;
use std::process::{Command, Stdio};
use std::time::Duration;

struct TestContainer {
    id: String,
    url: String,
}

impl TestContainer {
    fn new() -> Self {
        // Generate unique container name
        let name = format!("pg_mcp_test_{}", uuid::Uuid::new_v4().simple());

        // Pick a random port in high range
        let port = 50000
            + (uuid::Uuid::new_v4().as_bytes()[0..2]
                .iter()
                .enumerate()
                .map(|(i, b)| (*b as u16) << (8 * i))
                .sum::<u16>()
                % 10000);

        // Pull and run postgres container
        let output = Command::new("podman")
            .args([
                "run",
                "-d",
                "--name",
                &name,
                "-e",
                "POSTGRES_PASSWORD=postgres",
                "-p",
                &format!("{}:5432", port),
                "docker.io/postgres:15-alpine",
            ])
            .stderr(Stdio::inherit())
            .output()
            .expect("Failed to run podman container");

        if !output.status.success() {
            panic!(
                "Failed to start container: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let id = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // Wait for postgres to be ready
        let mut ready = false;
        for _ in 0..30 {
            std::thread::sleep(Duration::from_secs(1));
            let check = Command::new("podman")
                .args(["exec", &id, "pg_isready", "-U", "postgres"])
                .output();
            if let Ok(result) = check {
                if result.status.success() {
                    ready = true;
                    break;
                }
            }
        }
        if !ready {
            panic!(
                "PostgreSQL container {} did not become ready within 30 seconds",
                id
            );
        }

        let url = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", port);

        Self { id, url }
    }
}

impl Drop for TestContainer {
    fn drop(&mut self) {
        let _ = Command::new("podman").args(["rm", "-f", &self.id]).output();
    }
}

async fn create_app_state(url: &str, mode: PermissionMode) -> AppState {
    let config = Config {
        database_url: url.to_string(),
        default_schema: "public".to_string(),
        permission_mode: mode,
        max_connections: 5,
    };
    AppState::new(config).await.unwrap()
}

#[tokio::test]
async fn test_execute_sql_insert_select() {
    let container = TestContainer::new();
    let state = create_app_state(&container.url, PermissionMode::Unrestricted).await;

    // Create table
    let args = ExecuteSqlTool {
        sql: "CREATE TABLE test_users (id SERIAL PRIMARY KEY, name TEXT NOT NULL)".to_string(),
    };
    let result = handle_execute_sql(&state, &args).await;
    assert!(result.is_ok());

    // Insert row
    let args = ExecuteSqlTool {
        sql: "INSERT INTO test_users (name) VALUES ('Alice')".to_string(),
    };
    let result = handle_execute_sql(&state, &args).await;
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(text.contains("Rows affected: 1"));

    // Select
    let args = ExecuteQueryTool {
        sql: "SELECT name FROM test_users".to_string(),
    };
    let result = handle_execute_query(&state, &args).await;
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(text.contains("Alice"));
}

#[tokio::test]
async fn test_execute_query_readonly_error() {
    let container = TestContainer::new();
    let state = create_app_state(&container.url, PermissionMode::Unrestricted).await;

    // Create table via execute_sql
    let args = ExecuteSqlTool {
        sql: "CREATE TABLE test_insert (id SERIAL PRIMARY KEY)".to_string(),
    };
    handle_execute_sql(&state, &args).await.unwrap();

    // Try INSERT via execute_query (should fail due to READ ONLY)
    let args = ExecuteQueryTool {
        sql: "INSERT INTO test_insert DEFAULT VALUES".to_string(),
    };
    let result = handle_execute_query(&state, &args).await;
    assert!(result.is_err()); // READ ONLY transaction rejects writes
}

#[tokio::test]
async fn test_permission_mode_restricted() {
    let container = TestContainer::new();
    let state = create_app_state(&container.url, PermissionMode::Restricted).await;

    // Permission enforcement is in PgMcpHandler, not in handle_* functions.
    // Here we just verify that schema tools work with any permission mode.
    let result = handle_list_tables(&state, None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_permission_mode_readonly() {
    let container = TestContainer::new();
    let state = create_app_state(&container.url, PermissionMode::Readonly).await;

    // execute_query should work in readonly mode
    let args = ExecuteQueryTool {
        sql: "SELECT 1".to_string(),
    };
    let result = handle_execute_query(&state, &args).await;
    assert!(result.is_ok());

    // list_tables should work
    let result = handle_list_tables(&state, None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_schemas() {
    let container = TestContainer::new();
    let state = create_app_state(&container.url, PermissionMode::Unrestricted).await;

    let result = handle_list_schemas(&state).await;
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(text.contains("public"));
}

#[tokio::test]
async fn test_list_tables() {
    let container = TestContainer::new();
    let state = create_app_state(&container.url, PermissionMode::Unrestricted).await;

    // Create table first
    let args = ExecuteSqlTool {
        sql: "CREATE TABLE test_list (id SERIAL PRIMARY KEY)".to_string(),
    };
    handle_execute_sql(&state, &args).await.unwrap();

    let result = handle_list_tables(&state, None).await;
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(text.contains("test_list"));
}

#[tokio::test]
async fn test_get_table_structure() {
    let container = TestContainer::new();
    let state = create_app_state(&container.url, PermissionMode::Unrestricted).await;

    // Create table with columns
    let args = ExecuteSqlTool {
        sql: "CREATE TABLE test_struct (id SERIAL PRIMARY KEY, name TEXT NOT NULL, email TEXT UNIQUE)".to_string(),
    };
    handle_execute_sql(&state, &args).await.unwrap();

    let result = handle_get_table_structure(&state, "test_struct", None).await;
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(text.contains("id"));
    assert!(text.contains("name"));
    assert!(text.contains("email"));
}

#[tokio::test]
async fn test_get_view_definition() {
    let container = TestContainer::new();
    let state = create_app_state(&container.url, PermissionMode::Unrestricted).await;

    // Create table and view
    let args = ExecuteSqlTool {
        sql: "CREATE TABLE test_view_src (id SERIAL PRIMARY KEY, val TEXT)".to_string(),
    };
    handle_execute_sql(&state, &args).await.unwrap();

    let args = ExecuteSqlTool {
        sql: "CREATE VIEW test_view AS SELECT * FROM test_view_src".to_string(),
    };
    handle_execute_sql(&state, &args).await.unwrap();

    let result = handle_get_view_definition(&state, "test_view", None).await;
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(text.contains("test_view_src"));
}

#[tokio::test]
async fn test_list_triggers() {
    let container = TestContainer::new();
    let state = create_app_state(&container.url, PermissionMode::Unrestricted).await;

    // Create table with trigger
    let args = ExecuteSqlTool {
        sql: "CREATE TABLE test_trig (id SERIAL PRIMARY KEY, val TEXT)".to_string(),
    };
    handle_execute_sql(&state, &args).await.unwrap();

    let args = ExecuteSqlTool {
        sql: "CREATE FUNCTION test_trig_fn() RETURNS TRIGGER AS $$ BEGIN RETURN NEW; END; $$ LANGUAGE plpgsql".to_string(),
    };
    handle_execute_sql(&state, &args).await.unwrap();

    let args = ExecuteSqlTool {
        sql: "CREATE TRIGGER test_trigger BEFORE INSERT ON test_trig FOR EACH ROW EXECUTE FUNCTION test_trig_fn()".to_string(),
    };
    handle_execute_sql(&state, &args).await.unwrap();

    let result = handle_list_triggers(&state, None, None).await;
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(text.contains("test_trigger"));
}

#[tokio::test]
async fn test_list_indexes() {
    let container = TestContainer::new();
    let state = create_app_state(&container.url, PermissionMode::Unrestricted).await;

    // Create table with index
    let args = ExecuteSqlTool {
        sql: "CREATE TABLE test_idx (id SERIAL PRIMARY KEY, email TEXT UNIQUE)".to_string(),
    };
    handle_execute_sql(&state, &args).await.unwrap();

    let result = handle_list_indexes(&state, "test_idx", None).await;
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(text.contains("test_idx_pkey") || text.contains("test_idx_email_key"));
}

fn extract_text(result: &rust_mcp_sdk::schema::CallToolResult) -> String {
    result
        .content
        .first()
        .map(|c| {
            if let rust_mcp_sdk::schema::ContentBlock::TextContent(t) = c {
                t.text.clone()
            } else {
                String::new()
            }
        })
        .unwrap_or_default()
}

#[tokio::test]
async fn test_get_view_definition_not_found() {
    let container = TestContainer::new();
    let state = create_app_state(&container.url, PermissionMode::Unrestricted).await;

    let result = handle_get_view_definition(&state, "nonexistent_view", None).await;
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(text.contains("not found"));
}

#[tokio::test]
async fn test_get_function_definition_not_found() {
    let container = TestContainer::new();
    let state = create_app_state(&container.url, PermissionMode::Unrestricted).await;

    let result = handle_get_function_definition(&state, "nonexistent_function", None).await;
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(text.contains("not found"));
}

#[tokio::test]
async fn test_get_table_structure_not_found() {
    let container = TestContainer::new();
    let state = create_app_state(&container.url, PermissionMode::Unrestricted).await;

    let result = handle_get_table_structure(&state, "nonexistent_table", None).await;
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(text.contains("nonexistent_table"));
}

#[tokio::test]
async fn test_list_indexes_not_found() {
    let container = TestContainer::new();
    let state = create_app_state(&container.url, PermissionMode::Unrestricted).await;

    let result = handle_list_indexes(&state, "nonexistent_table", None).await;
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(text == "[]" || text.is_empty());
}

#[tokio::test]
async fn test_execute_query_empty_result() {
    let container = TestContainer::new();
    let state = create_app_state(&container.url, PermissionMode::Unrestricted).await;

    let args = ExecuteSqlTool {
        sql: "CREATE TABLE test_empty (id SERIAL PRIMARY KEY)".to_string(),
    };
    handle_execute_sql(&state, &args).await.unwrap();

    let args = ExecuteQueryTool {
        sql: "SELECT * FROM test_empty".to_string(),
    };
    let result = handle_execute_query(&state, &args).await;
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(text.contains("0 rows returned") || text == "[]");
}

#[tokio::test]
async fn test_list_tables_empty() {
    let container = TestContainer::new();
    let state = create_app_state(&container.url, PermissionMode::Unrestricted).await;

    let result = handle_list_tables(&state, None).await;
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(text.contains("[]") || text.is_empty() || text.contains("pg_"));
}

#[tokio::test]
async fn test_uuid_column() {
    let container = TestContainer::new();
    let state = create_app_state(&container.url, PermissionMode::Unrestricted).await;

    let args = ExecuteSqlTool {
        sql: "CREATE TABLE test_uuid (id UUID PRIMARY KEY DEFAULT gen_random_uuid())".to_string(),
    };
    handle_execute_sql(&state, &args).await.unwrap();

    let args = ExecuteSqlTool {
        sql: "INSERT INTO test_uuid (id) VALUES (gen_random_uuid())".to_string(),
    };
    handle_execute_sql(&state, &args).await.unwrap();

    let args = ExecuteQueryTool {
        sql: "SELECT id FROM test_uuid".to_string(),
    };
    let result = handle_execute_query(&state, &args).await;
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(text.contains("uuid"));
}
