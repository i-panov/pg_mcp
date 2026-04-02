# pg_mcp — Agent Guidelines

## Build / Lint / Test

```bash
cargo build --release          # production build
cargo build                    # dev build
cargo test                     # all tests
cargo test test_name           # single test by name
cargo test --test integration  # specific integration test file
cargo clippy -- -D warnings    # strict lint (deny warnings)
cargo fmt                      # format all code
```

Pre-commit: always run `cargo fmt && cargo clippy` before committing.

## Architecture

Rust edition 2024. MCP server for PostgreSQL using `rust-mcp-sdk` (stdio transport)
and `sqlx` with `PgPool`. Configuration via `figment` from `.env` or environment variables.

```
src/
  lib.rs         — public library: handler functions, parse_args, row_to_json_value, sanitize_sql_error
  main.rs        — entry point, PgMcpHandler (ServerHandler impl), MCP server bootstrap
  config.rs      — figment-based Config (DATABASE_URL, DEFAULT_SCHEMA, PERMISSION_MODE)
  error.rs       — thiserror AppError enum (currently unused by handlers)
  state.rs       — AppState { pool, default_schema, permission_mode }
  tools/         — MCP tool definitions (one struct per tool via mcp_tool macro)
    mod.rs       — re-exports all tools
    execute_sql.rs
    execute_query.rs
    schema.rs    — all schema-introspection tools
tests/
  integration.rs — podman-based integration tests
```

Handler logic lives in `lib.rs` so it's importable from integration tests.
`main.rs` only contains `PgMcpHandler` (permission gating + arg parsing) and server bootstrap.

## Permission Modes

The server supports 3 permission modes controlled by `PERMISSION_MODE` env var:

| Mode           | execute_sql | execute_query | Schema tools |
|----------------|-------------|---------------|--------------|
| `unrestricted` | ✅           | ✅             | ✅            |
| `readonly`     | ❌           | ✅             | ✅            |
| `restricted`   | ❌           | ❌             | ✅            |

Default: `restricted`.

Schema tools = `list_schemas`, `list_tables`, `list_views`, `list_materialized_views`,
`list_procedures`, `list_triggers`, `list_indexes`, `get_table_structure`,
`get_view_definition`, `get_function_definition`.

When a tool is disabled by the current mode, the handler returns
`CallToolError::unknown_tool` (makes it invisible to the client).

## Code Style

### Imports
Group in order: (1) std, (2) external crates, (3) local crate.
Blank line between groups. Alphabetize within groups.

```rust
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

use crate::config::Config;
use crate::error::AppError;
```

### Naming
- Structs: `PascalCase` (e.g. `ExecuteSqlTool`, `AppState`)
- Functions/methods: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Tool names in MCP: `snake_case` (e.g. `execute_sql`)

### Types
- Prefer owned `String` in public API (tool params are `String`, not `&str`)
- Use `Option<String>` for optional tool parameters
- Derive `Debug, Deserialize, Serialize, JsonSchema` on all tool structs

### MCP Tool Pattern
Each tool is a struct annotated with `#[mcp_tool(...)]`. Description is in the
macro; field-level docs (`/// comment`) describe parameters.

```rust
#[mcp_tool(name = "my_tool", description = "Does X")]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct MyTool {
    /// Param description
    pub param: String,
    pub optional: Option<String>,
}
```

### Error Handling
- Use `thiserror` for `AppError` enum; never `anyhow` in library/tool code
- `#[from]` for automatic conversion from `sqlx::Error`
- Handler errors go through `CallToolError::new` with local error types (`ArgsError`, `SqlError`)
- MCP tool visibility gated by `CallToolError::unknown_tool`
- SQL errors are sanitized via `sanitize_sql_error` to strip connection strings (password leak prevention)

### Async
- All I/O functions are `async fn`
- `tokio` is used for `#[tokio::main]` only; sqlx provides the async runtime
- Use `sqlx::query!` macros when schema is known; fall back to `sqlx::query` for dynamic SQL
- Always import `sqlx::Row` to use `.try_get()` on query results

### Config
- `figment` reads from env vars (`.env` or process env)
- `Config` struct derives `Deserialize`; fields use `#[serde(default)]`
- Never `panic!` on missing config — return `Err` early
- `PERMISSION_MODE` is case-insensitive (accepts `restricted`, `RESTRICTED`, `Unrestricted`, etc.)

### Formatting
- 4-space indent, no tabs
- Max line length ~100 chars (soft)
- `cargo fmt` is the single source of truth

## Environment

Copy `.env.example` to `.env` and set:
```
DATABASE_URL=postgres://user:pass@localhost:5432/db
DEFAULT_SCHEMA=public
PERMISSION_MODE=restricted  # unrestricted | readonly | restricted
```

## Git

- No secrets in commits (`.env` is gitignored)
- Branch naming: `feat/`, `fix/`, `chore/` prefix
- After completing each task or feature, commit with a descriptive message
- Format: `type: description` (e.g. `feat: add permission modes`)
- Types: `feat`, `fix`, `chore`, `docs`, `test`, `refactor`
- Run `cargo fmt && cargo clippy` before every commit
