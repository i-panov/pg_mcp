use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "explain_query",
    description = "Returns the execution plan of a SQL query using EXPLAIN (FORMAT JSON). Safe in all permission modes as it never executes or modifies data."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ExplainQueryTool {
    /// The SQL query to analyze
    pub sql: String,
}

#[mcp_tool(
    name = "get_table_row_count",
    description = "Returns the number of rows in a table. Can use approximate count for speed."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetTableRowCountTool {
    /// Table name
    pub table: String,
    /// Schema name (uses DEFAULT_SCHEMA if not specified)
    pub schema: Option<String>,
    /// If true, uses approximate count from pg_class.reltuples (fast). If false, runs COUNT(*) (accurate but slower). Defaults to false.
    pub approximate: Option<bool>,
}

#[mcp_tool(
    name = "list_active_queries",
    description = "Returns currently running queries from pg_stat_activity (idle queries excluded)."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListActiveQueriesTool {}

#[mcp_tool(
    name = "list_locks",
    description = "Returns active locks, focusing on granted and conflicting locks. Useful for debugging deadlocks."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListLocksTool {}
