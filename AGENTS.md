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
  config.rs      — figment-based Config (DATABASE_URL, DEFAULT_SCHEMA, PERMISSION_MODE, MAX_CONNECTIONS, MAX_RESULT_ROWS)
  state.rs       — AppState { pool, default_schema, permission_mode, max_result_rows }
  tools/         — MCP tool definitions (one struct per tool via mcp_tool macro)
    mod.rs       — re-exports all tools
    analyze.rs   — explain, table row count, active queries, locks
    execute_sql.rs
    execute_query.rs
    schema.rs    — list_schemas, list_tables, list_views, list_materialized_views, list_routines,
                   list_triggers, list_indexes, get_table_structure, get_view_definition,
                   get_function_definition, get_table_size, list_extensions, list_sequences
    analyze.rs   — explain_query, get_table_row_count, list_active_queries, list_locks
tests/
  integration.rs — podman-based integration tests (each test spins up a fresh postgres:15-alpine container)
```

Handler logic lives in `lib.rs` so it's importable from integration tests.
`main.rs` only contains `PgMcpHandler` (permission gating + arg parsing) and server bootstrap.

## Permission Modes

The server supports 3 permission modes controlled by `PERMISSION_MODE` env var:

| Mode           | execute_sql | execute_query | Schema/analyze tools |
|----------------|-------------|---------------|----------------------|
| `unrestricted` | ✅           | ✅             | ✅                    |
| `readonly`     | ❌           | ✅             | ✅                    |
| `restricted`   | ❌           | ❌             | ✅                    |

Default: `restricted`.

Permission gating happens in `main.rs` `handle_call_tool_request` — not in handler functions.
When a tool is disabled by the current mode, the handler returns
`CallToolError::unknown_tool` (makes it invisible to the client).

## Code Style

- `cargo fmt` is the single source of truth for formatting
- Import order: std → external crates → local crate (alphabetized within groups)
- Derive `Debug, Deserialize, Serialize, JsonSchema` on all MCP tool structs
- Use `thiserror` for errors, never `anyhow` in library/tool code
- SQL errors are sanitized via `sanitize_sql_error` to strip connection strings (password leak prevention)

## Environment

Copy `.env.example` to `.env` and set:
```
DATABASE_URL=postgres://user:pass@localhost:5432/db
DEFAULT_SCHEMA=public
PERMISSION_MODE=restricted  # unrestricted | readonly | restricted
MAX_CONNECTIONS=5           # optional, default 5
MAX_RESULT_ROWS=1000
```

## Git

- No secrets in commits (`.env` is gitignored)
- Branch naming: `feat/`, `fix/`, `chore/` prefix
- Format: `type: description` (e.g. `feat: add permission modes`)
- Types: `feat`, `fix`, `chore`, `docs`, `test`, `refactor`
- Run `cargo fmt && cargo clippy` before every commit
