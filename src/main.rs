use async_trait::async_trait;
use pg_mcp::config::{PermissionMode, load_config};
use pg_mcp::state::AppState;
use pg_mcp::tools::*;
use pg_mcp::*;
use rust_mcp_sdk::error::SdkResult;
use rust_mcp_sdk::mcp_server::{
    McpServerOptions, ServerHandler, ToMcpServerHandler, server_runtime,
};
use rust_mcp_sdk::schema::schema_utils::{CallToolError, SdkError, SdkErrorCodes};
use rust_mcp_sdk::schema::*;
use rust_mcp_sdk::{McpServer, StdioTransport, TransportOptions};
use std::sync::Arc;

#[derive(Debug)]
struct PgMcpHandler {
    state: AppState,
}

impl PgMcpHandler {
    fn new(state: AppState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl ServerHandler for PgMcpHandler {
    async fn handle_list_tools_request(
        &self,
        _request: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult {
            tools: vec![
                ExecuteSqlTool::tool(),
                ExecuteQueryTool::tool(),
                ListSchemasTool::tool(),
                ListTablesTool::tool(),
                ListViewsTool::tool(),
                ListMaterializedViewsTool::tool(),
                ListRoutinesTool::tool(),
                ListTriggersTool::tool(),
                ListIndexesTool::tool(),
                GetTableStructureTool::tool(),
                GetViewDefinitionTool::tool(),
                GetFunctionDefinitionTool::tool(),
                ExplainQueryTool::tool(),
                GetTableSizeTool::tool(),
                ListExtensionsTool::tool(),
                ListSequencesTool::tool(),
                GetTableRowCountTool::tool(),
                ListActiveQueriesTool::tool(),
                ListLocksTool::tool(),
            ],
            meta: None,
            next_cursor: None,
        })
    }

    async fn handle_call_tool_request(
        &self,
        params: CallToolRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        match params.name.as_str() {
            "execute_sql" => {
                if self.state.permission_mode != PermissionMode::Unrestricted {
                    return Err(CallToolError::unknown_tool(params.name));
                }
                let args = parse_args::<ExecuteSqlTool>(&params.arguments)?;
                handle_execute_sql(&self.state, &args).await
            }
            "execute_query" => {
                if self.state.permission_mode == PermissionMode::Restricted {
                    return Err(CallToolError::unknown_tool(params.name));
                }
                let args = parse_args::<ExecuteQueryTool>(&params.arguments)?;
                handle_execute_query(&self.state, &args).await
            }
            "list_schemas" => handle_list_schemas(&self.state).await,
            "list_tables" => {
                let args = parse_args::<ListTablesTool>(&params.arguments)?;
                handle_list_tables(&self.state, args.schema).await
            }
            "list_views" => {
                let args = parse_args::<ListViewsTool>(&params.arguments)?;
                handle_list_views(&self.state, args.schema).await
            }
            "list_materialized_views" => {
                let args = parse_args::<ListMaterializedViewsTool>(&params.arguments)?;
                handle_list_materialized_views(&self.state, args.schema).await
            }
            "list_routines" => {
                let args = parse_args::<ListRoutinesTool>(&params.arguments)?;
                handle_list_routines(&self.state, args.schema).await
            }
            "list_triggers" => {
                let args = parse_args::<ListTriggersTool>(&params.arguments)?;
                handle_list_triggers(&self.state, args.table, args.schema).await
            }
            "list_indexes" => {
                let args = parse_args::<ListIndexesTool>(&params.arguments)?;
                handle_list_indexes(&self.state, &args.table, args.schema).await
            }
            "get_table_structure" => {
                let args = parse_args::<GetTableStructureTool>(&params.arguments)?;
                handle_get_table_structure(&self.state, &args.table, args.schema).await
            }
            "get_view_definition" => {
                let args = parse_args::<GetViewDefinitionTool>(&params.arguments)?;
                handle_get_view_definition(&self.state, &args.view, args.schema).await
            }
            "get_function_definition" => {
                let args = parse_args::<GetFunctionDefinitionTool>(&params.arguments)?;
                handle_get_function_definition(&self.state, &args.function, args.schema).await
            }
            "explain_query" => {
                let args = parse_args::<ExplainQueryTool>(&params.arguments)?;
                handle_explain_query(&self.state, &args).await
            }
            "get_table_size" => {
                let args = parse_args::<GetTableSizeTool>(&params.arguments)?;
                handle_get_table_size(&self.state, &args.table, args.schema).await
            }
            "list_extensions" => handle_list_extensions(&self.state).await,
            "list_sequences" => {
                let args = parse_args::<ListSequencesTool>(&params.arguments)?;
                handle_list_sequences(&self.state, args.schema).await
            }
            "get_table_row_count" => {
                let args = parse_args::<GetTableRowCountTool>(&params.arguments)?;
                handle_get_table_row_count(
                    &self.state,
                    &args.table,
                    args.schema,
                    args.approximate.unwrap_or(false),
                )
                .await
            }
            "list_active_queries" => handle_list_active_queries(&self.state).await,
            "list_locks" => handle_list_locks(&self.state).await,
            _ => Err(CallToolError::unknown_tool(params.name)),
        }
    }
}

#[tokio::main]
async fn main() -> SdkResult<()> {
    let config =
        load_config().map_err(|e| SdkError::new(SdkErrorCodes::INVALID_PARAMS, e, None))?;
    let state = AppState::new(config).await.map_err(|e| {
        SdkError::new(
            SdkErrorCodes::INTERNAL_ERROR,
            format!("Failed to connect to PostgreSQL: {}", e),
            None,
        )
    })?;

    let server_info = InitializeResult {
        server_info: Implementation {
            name: "pg-mcp".into(),
            version: "0.1.0".into(),
            title: Some("PostgreSQL MCP Server".into()),
            description: Some(
                "MCP server for PostgreSQL schema introspection and query execution".into(),
            ),
            icons: vec![],
            website_url: None,
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default()
        },
        protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
        instructions: None,
        meta: None,
    };

    let transport = StdioTransport::new(TransportOptions::default())?;
    let handler = PgMcpHandler::new(state);
    let server = server_runtime::create_server(McpServerOptions {
        server_details: server_info,
        transport,
        handler: handler.to_mcp_server_handler(),
        task_store: None,
        client_task_store: None,
        message_observer: None,
    });

    server.start().await
}
