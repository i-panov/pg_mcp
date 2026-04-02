# pg_mcp

MCP server for PostgreSQL. Provides tools for schema introspection and query execution via the [Model Context Protocol](https://modelcontextprotocol.io).

## Features

- Execute arbitrary SQL queries
- Execute read-only queries (safe for SELECT)
- Introspect schemas, tables, views, functions, triggers, indexes
- Three permission modes for security control

## Requirements

- Rust 1.75+ (edition 2024)
- PostgreSQL 15+
- Podman or Docker (for integration tests)

## Setup

```bash
cp .env.example .env
# Edit .env with your database credentials
```

```
DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres
DEFAULT_SCHEMA=public
PERMISSION_MODE=restricted
```

## Build & Run

```bash
cargo build --release
./target/release/pg_mcp
```

The server communicates over stdio (MCP protocol).

## Tools

| Tool | Description |
|------|-------------|
| `execute_sql` | Execute arbitrary SQL (INSERT, UPDATE, DELETE, SELECT, etc.) |
| `execute_query` | Execute read-only queries via READ ONLY transaction |
| `list_schemas` | List all schemas |
| `list_tables` | List tables in a schema |
| `list_views` | List ordinary views in a schema |
| `list_materialized_views` | List materialized views in a schema |
| `list_procedures` | List functions/procedures in a schema |
| `list_triggers` | List triggers (optional table filter) |
| `list_indexes` | List indexes for a table |
| `get_table_structure` | Columns, constraints, foreign keys |
| `get_view_definition` | SQL definition of a view |
| `get_function_definition` | SQL definition of a function |

## Permission Modes

| Mode | execute_sql | execute_query | Schema tools |
|------|-------------|---------------|--------------|
| `unrestricted` | ✅ | ✅ | ✅ |
| `readonly` | ❌ | ✅ | ✅ |
| `restricted` | ❌ | ❌ | ✅ |

Default: `restricted`. Set via `PERMISSION_MODE` env var.
