# feishu-mcp-server

A production-ready [Feishu (Lark)](https://www.feishu.cn/) MCP Server built in Rust, enabling AI Agents (Claude Code, Cursor, Cline) to natively read and write Feishu data via the Model Context Protocol.

## Features

- **Documents**: search, get, create, update, list Feishu cloud documents
- **Tasks**: list, get, create, update status, complete tasks
- **Messages**: send, get, search messages in chats

## Architecture

```
AI Agent → MCP Protocol → Tool Handlers → FeishuClient → Feishu Open API
```

- **Framework**: FastMCP (Model Context Protocol)
- **Auth**: OAuth2 `tenant_access_token` with AES-256-GCM encrypted SQLite storage
- **Rate Limiting**: Sliding window (100 req/min per user)
- **Error Handling**: Unified FeishuError → ToolError mapping with automatic token refresh

## Quick Start

### Prerequisites

- Rust 1.75+
- A Feishu open platform application ([apply here](https://open.feishu.cn/))

### Build

```bash
git clone https://github.com/YOUR_HANDLE/feishu-mcp.git
cd feishu-mcp
cargo build --release
```

### Configuration

```bash
mkdir -p ~/.feishu-mcp
cp config.example.yaml ~/.feishu-mcp/config.yaml
vim ~/.feishu-mcp/config.yaml
# Fill in app_id and app_secret
```

Required environment variables:
- `FEISHU_APP_SECRET` — your Feishu app secret (takes precedence over config file)
- `FEISHU_ENCRYPTION_KEY` — 32-byte key for token encryption (optional, auto-generated if not set)

### Run

```bash
./target/release/feishu-mcp-server --port 3000
```

### Required App Permissions

Enable these in your Feishu app settings:
- `doc:readonly`, `doc:write`
- `task:readonly`, `task:write`
- `im:message:send_as_bot`

## Claude Code Integration

```bash
# Add to Claude Code MCP servers
claude mcp add feishu -- docker run --rm -i ghcr.io/YOUR_HANDLE/feishu-mcp:latest
```

Or local:
```
# In Claude Code settings, add MCP server URL:
# http://localhost:3000/mcp
```

## Available Tools

### Documents

| Tool | Description |
|------|-------------|
| `search_documents` | Full-text search across Feishu cloud documents |
| `get_document` | Get document content and metadata by ID |
| `create_document` | Create a new document in a folder |
| `update_document` | Overwrite document content |
| `list_documents` | List documents in a folder |

### Tasks

| Tool | Description |
|------|-------------|
| `list_tasks` | List all tasks under a goal |
| `get_task` | Get task details by ID |
| `create_task` | Create a new task |
| `update_task_status` | Update task status (todo/in_progress/done) |
| `complete_task` | Mark task as completed |

### Messages

| Tool | Description |
|------|-------------|
| `send_message` | Send a message to a chat |
| `get_messages` | Get recent messages from a chat |
| `search_messages` | Search messages by keyword |

## Docker

```bash
docker build -t feishu-mcp .
docker run -p 3000:3000 feishu-mcp
```

## Project Structure

```
feishu-mcp/
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Library exports
│   ├── config.rs            # Config loading
│   ├── auth/
│   │   ├── oauth.rs         # OAuth2 token management
│   │   └── token_store.rs   # SQLite + AES-256-GCM storage
│   ├── feishu/
│   │   ├── client.rs        # Feishu API HTTP client
│   │   ├── error.rs        # Unified error types
│   │   └── types.rs        # API response types
│   ├── tools/
│   │   ├── documents.rs     # Document tools
│   │   ├── tasks.rs        # Task tools
│   │   └── messages.rs     # Message tools
│   └── middleware/
│       ├── rate_limit.rs    # Sliding window rate limiter
│       └── error_handler.rs # Error conversion
├── Cargo.toml
├── Dockerfile
├── config.example.yaml
└── README.md
```

## Testing

```bash
# Unit tests (WireMock mocks Feishu API, no real credentials needed)
cargo test -- --test-threads=1

# Integration tests (requires real credentials)
cargo test test_oauth_e2e -- --ignored
```

## License

MIT
