use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "execute_sql",
    description = "Executes an arbitrary SQL query against PostgreSQL. Supports both SELECT and DML statements (INSERT, UPDATE, DELETE, etc.)."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ExecuteSqlTool {
    /// The SQL query to execute
    pub sql: String,
}
