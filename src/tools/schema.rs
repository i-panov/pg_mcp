use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "list_schemas",
    description = "Returns all schemas in the database"
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListSchemasTool {}

#[mcp_tool(
    name = "list_tables",
    description = "Returns tables in a schema (or all schemas if schema not specified)"
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListTablesTool {
    /// Schema name (uses DEFAULT_SCHEMA if not specified)
    pub schema: Option<String>,
}

#[mcp_tool(
    name = "list_views",
    description = "Returns ordinary views in a schema"
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListViewsTool {
    /// Schema name (uses DEFAULT_SCHEMA if not specified)
    pub schema: Option<String>,
}

#[mcp_tool(
    name = "list_materialized_views",
    description = "Returns materialized views in a schema"
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListMaterializedViewsTool {
    /// Schema name (uses DEFAULT_SCHEMA if not specified)
    pub schema: Option<String>,
}

#[mcp_tool(
    name = "list_routines",
    description = "Returns functions and procedures in a schema"
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListRoutinesTool {
    /// Schema name (uses DEFAULT_SCHEMA if not specified)
    pub schema: Option<String>,
}

#[mcp_tool(
    name = "list_triggers",
    description = "Returns triggers. Can be filtered by table name"
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListTriggersTool {
    /// Table name to filter by (optional)
    pub table: Option<String>,
    /// Schema name (uses DEFAULT_SCHEMA if not specified)
    pub schema: Option<String>,
}

#[mcp_tool(name = "list_indexes", description = "Returns indexes for a table")]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListIndexesTool {
    /// Table name
    pub table: String,
    /// Schema name (uses DEFAULT_SCHEMA if not specified)
    pub schema: Option<String>,
}

#[mcp_tool(
    name = "get_table_structure",
    description = "Returns detailed structure of a table: columns, types, constraints, foreign keys"
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetTableStructureTool {
    /// Table name
    pub table: String,
    /// Schema name (uses DEFAULT_SCHEMA if not specified)
    pub schema: Option<String>,
}

#[mcp_tool(
    name = "get_view_definition",
    description = "Returns SQL definition of a view"
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetViewDefinitionTool {
    /// View name
    pub view: String,
    /// Schema name (uses DEFAULT_SCHEMA if not specified)
    pub schema: Option<String>,
}

#[mcp_tool(
    name = "get_function_definition",
    description = "Returns SQL definition of a function or procedure"
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetFunctionDefinitionTool {
    /// Function name
    pub function: String,
    /// Schema name (uses DEFAULT_SCHEMA if not specified)
    pub schema: Option<String>,
}
