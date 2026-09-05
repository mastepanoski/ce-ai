# Design: Skills Resolve Positional Query Support

## 1. CLI Contract & Subcommand Definition

In `src/commands/skills.rs`:

```rust
#[derive(Subcommand, Debug, Clone)]
pub enum Action {
    List {
        #[arg(long, default_value = "opencode")]
        harness: String,

        #[arg(long, default_value_t = false)]
        json: bool,
    },
    Resolve {
        #[arg(long, default_value = "opencode")]
        harness: String,

        /// Search query (positional)
        #[arg(value_name = "QUERY")]
        query_pos: Option<String>,

        /// Search query (flag)
        #[arg(long)]
        query: Option<String>,

        #[arg(long, default_value_t = false)]
        json: bool,
    },
}
```

## 2. Parameter Resolution Logic

In `src/commands/skills.rs`:

```rust
Action::Resolve {
    harness,
    query_pos,
    query,
    json,
} => {
    let search_query = query_pos
        .as_deref()
        .or(query.as_deref())
        .unwrap_or("")
        .trim();
    if search_query.is_empty() {
        return Err(CeError::Usage(
            "Missing search query. Provide either a positional query or --query <QUERY>."
                .to_string(),
        ));
    }
    // Proceed with resolution using search_query
    ...
}
```

If neither `query_pos` nor `query` is provided (or if both are empty/whitespace), return `CeError::Usage` (exit code 2) with a descriptive error message.

## 3. Backward Compatibility
- Commands such as `ce-ai skills resolve --query sequential-thinking` continue to parse into `query: Some("sequential-thinking".into())` and `query_pos: None`.
- Commands such as `ce-ai skills resolve sequential-thinking` parse into `query_pos: Some("sequential-thinking".into())` and `query: None`.
- Commands combining positional and flags (e.g. `ce-ai skills resolve sequential-thinking --query custom-flag`) prioritize the positional query or fall back cleanly.
- JSON output `--json` remains unaffected and outputs matching schema.
