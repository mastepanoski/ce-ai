# Exploration: Technical Investigation for Claude Code Adapter

## Evaluated Options for MCP Configuration Path
1. **Option A: User Scope `~/.claude.json` / `~/.claude/settings.json`**
   - Pros: Native user-wide scope supported by `claude mcp add --scope user`. Works across all terminals and projects.
   - Cons: Must preserve existing top-level fields in `~/.claude.json`.
2. **Option B: Project Scope `.mcp.json`**
   - Pros: Repository local.
   - Cons: Not suitable for global CLI sidecar installation (`ce-ai install --harness claude`).

**Decision**: Option A. For global harness installation (`ce-ai install --harness claude`), target `~/.claude.json` (or `~/.claude/settings.json`) using `mcpServers` stdio schema, preserving all existing top-level JSON fields and extra server properties.

## Rule Location Exploration
Claude Code checks:
1. `~/.claude/CLAUDE.md` (user memory)
2. `./CLAUDE.md` or `.claude/CLAUDE.md` (project directives)

**Decision**: `init_prj` will update `.claude/CLAUDE.md` or `./CLAUDE.md` with demarcated `CE-AI MANAGED BLOCK`.
