# Feishu MCP Server

A Model Context Protocol (MCP) server implementation for Feishu (飞书) collaboration platform.

Built with the official Rust MCP SDK (`rmcp`), this server exposes Feishu APIs as MCP tools, enabling AI assistants like Claude Desktop, Claude Code, and Cursor to interact with Feishu documents, tasks, and messages.

## Quick Start

### 1. Prerequisites

- Rust 1.75+
- A Feishu application with appropriate permissions

### 2. Configuration

Create `~/.feishu-mcp/config.yaml`:

```yaml
feishu:
  app_id: "your_app_id"
  app_secret: "your_app_secret"
  base_url: "https://open.feishu.cn/open-apis"

server:
  host: "0.0.0.0"
  port: 3000

rate_limit:
  max_requests_per_minute: 100

logging:
  level: "info"
  redact_tokens: true
```

**Environment Variables** (override config file):

| Variable | Description |
|----------|-------------|
| `FEISHU_APP_SECRET` | Overrides `feishu.app_secret` |
| `FEISHU_ENCRYPTION_KEY` | 32-byte key for token store encryption |
| `FEISHU_TOKEN_DB` | Path to token database (default: `~/.feishu-mcp/tokens.db`) |

### 3. Run (stdio transport -- default)

```bash
cargo build --release
./target/release/feishu-mcp-server
```

Or with custom config:

```bash
./target/release/feishu-mcp-server --config /path/to/config.yaml --verbose
```

### 4. Run (HTTP transport -- experimental)

```bash
./target/release/feishu-mcp-server --http --port 3000
```

### Docker

```bash
docker build -t feishu-mcp .
docker run -p 3000:3000 \
  -v ~/.feishu-mcp/config.yaml:/app/config.yaml \
  -e FEISHU_APP_SECRET=your_secret \
  -e FEISHU_ENCRYPTION_KEY=your_32_byte_key \
  feishu-mcp
```

## Feishu Application Setup

### Required Permissions

Enable these permissions in your Feishu app:

**Documents**
- `doc:readonly` -- Read documents
- `doc:write` -- Create/edit documents

**Tasks**
- `task:readonly` -- Read tasks
- `task:write` -- Create/update tasks

**Messages**
- `im:message:readonly` -- Read messages
- `im:message:send_as_bot` -- Send messages

### Getting Credentials

1. Go to [Feishu Open Platform](https://open.feishu.cn/)
2. Create an application
3. Obtain `App ID` and `App Secret` from the credentials page
4. Configure the required permissions
5. Publish the application

## MCP Client Configuration

The server uses **stdio transport** by default (the standard MCP transport mode).

### Claude for Desktop

Add to your Claude Desktop config (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "feishu": {
      "command": "/absolute/path/to/feishu-mcp-server"
    }
  }
}
```

### Claude Code

Add to your Claude Code config (`~/.claude/settings.json`):

```json
{
  "mcpServers": {
    "feishu": {
      "command": "/absolute/path/to/feishu-mcp-server"
    }
  }
}
```

### Cursor

Add to Cursor's MCP settings:

```json
{
  "mcpServers": {
    "feishu": {
      "command": "/absolute/path/to/feishu-mcp-server"
    }
  }
}
```

For remote usage (HTTP transport), add the `--http` flag:

```json
{
  "mcpServers": {
    "feishu": {
      "command": "/absolute/path/to/feishu-mcp-server",
      "args": ["--http", "--port", "3000"]
    }
  }
}
```

## Available Tools (13 total)

The server exposes 13 MCP tools organized into three categories.

### Documents (5)

| Tool | Description | Read-Only | Idempotent |
|------|-------------|:---------:|:----------:|
| `search_documents` | Search documents by query string | Yes | Yes |
| `get_document` | Get a document by ID | Yes | Yes |
| `create_document` | Create a new document | No | No |
| `update_document` | Update document content | No | No |
| `list_documents` | List documents in a folder | Yes | Yes |

### Tasks (5)

| Tool | Description | Read-Only | Idempotent |
|------|-------------|:---------:|:----------:|
| `list_tasks` | List tasks by goal ID | Yes | Yes |
| `get_task` | Get task by ID | Yes | Yes |
| `create_task` | Create a new task | No | No |
| `update_task_status` | Update task status (todo/in_progress/done) | No | Yes |
| `complete_task` | Mark task as completed | No | Yes |

### Messages (3)

| Tool | Description | Read-Only | Idempotent |
|------|-------------|:---------:|:----------:|
| `send_message` | Send a message to a chat | No | No |
| `get_messages` | Get messages from a chat | Yes | Yes |
| `search_messages` | Search messages by query | Yes | Yes |

## Testing with MCP Inspector

You can test the server using the [MCP Inspector](https://github.com/modelcontextprotocol/inspector):

```bash
npx @modelcontextprotocol/inspector ./target/release/feishu-mcp-server
```

Or with arguments:

```bash
npx @modelcontextprotocol/inspector "./target/release/feishu-mcp-server --verbose"
```

Once connected, the Inspector will show all 13 registered tools with their schemas. You can invoke individual tools to verify they respond correctly.

## Command Line Options

| Option | Description |
|--------|-------------|
| `-p, --port <PORT>` | Port for HTTP transport (default: 3000) |
| `-c, --config <PATH>` | Path to config file (default: `~/.feishu-mcp/config.yaml`) |
| `--http` | Use HTTP transport instead of stdio |
| `-v, --verbose` | Enable debug logging |
| `--help` | Show help |
| `--version` | Show version |

## Architecture

```
                     MCP Client
            (Claude Desktop / Cursor / etc.)
                        |
                MCP Protocol (JSON-RPC)
                   (stdio / HTTP)
                        |
               FeishuMcpServer (rmcp)
            +-- tool_router (13 tools) --+
            |                           |
      Documents Tools             Tasks Tools
      Messages Tools           FeishuClient API
                        |
            +-----------+-----------+
            |           |           |
      Feishu API     OAuth 2.0   Token Store
      (REST)        (Tenant)    (AES-256-GCM)
                        |
                   Rate Limiter
```

## Security Notes

- **Never commit credentials** to version control
- App secrets are redacted in logs (`redact_tokens: true`)
- Tokens are encrypted at rest using AES-256-GCM
- Use environment variables for sensitive configuration
- The encryption key must be exactly 32 bytes
- Rate limiting protects against API abuse (configurable)

## Development

### Build

```bash
cargo build --release
```

### Test

```bash
cargo test          # Unit + integration tests
cargo test --lib    # Unit tests only
```

All tests use isolated database instances and mock HTTP servers for deterministic testing.

## Configuration Precedence

1. Environment variables (highest priority)
2. `--config` flag value
3. `~/.feishu-mcp/config.yaml`
4. Default values (port 3000, rate limit 100/min)

## License

MIT
