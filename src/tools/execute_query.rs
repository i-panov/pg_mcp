use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "execute_query",
    description = "Executes a read-only SQL query against PostgreSQL using a READ ONLY transaction. Safe for SELECT queries."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ExecuteQueryTool {
    /// The SQL query to execute (must be read-only, e.g., SELECT)
    pub sql: String,
}
