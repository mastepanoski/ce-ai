# OpenSpec Design: Sidecar Registration Contracts

- **Change:** `tools-install-real-sidecar-registration`
- **Issue:** #158 (P0)

---

## 📐 1. MCP Server Config Structures

For `context7`:
```json
{
  "mcpServers": {
    "context7": {
      "command": "npx",
      "args": ["-y", "@upstash/context7-mcp@latest"]
    }
  }
}
```

For `engram`:
```json
{
  "mcpServers": {
    "engram": {
      "command": "engram",
      "args": ["serve"]
    }
  }
}
```

For `rtk`:
```json
{
  "mcpServers": {
    "rtk": {
      "command": "rtk",
      "args": ["mcp"]
    }
  }
}
```

For `codegraph`:
Runs `gentle-ai codegraph init --cwd <repo_root>` if inside git repo, and registers `codegraph` MCP server entry.
