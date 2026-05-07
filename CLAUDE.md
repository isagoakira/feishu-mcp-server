# Feishu MCP Server

> **Status**: Archived — Project completed 2026-05-06

This project was developed using a multi-agent workflow. The implementation has been moved to the `feishu-mcp/` subdirectory.

For the actual project code, see **`feishu-mcp/`**.

---

## Development Artifacts (Not for Commit)

The following files are development metadata and should NOT be committed:

```
.claude/              # Harness state, VCS snapshots
dev-plan.md           # Development task plan
harness-spec.md       # Technical specifications
main-log.md           # Activity log
lessons-learned.md    # Engineering lessons
TODO.md               # Task tracking
02-飞书MCP-Server-项目计划书.md  # Original Chinese project brief
```

---

## Quick Start

```bash
cd feishu-mcp
cargo build --release
./target/release/feishu-mcp-server --port 3000
```

See `feishu-mcp/README.md` for full documentation.
