# Design
- harness::registration::{McpRegistrar, RegistrationSpec, registration_spec,
  copy_managed_skills} is the single source of truth; sync dispatch calls
  spec.register_companions(target_config); install adds the uniform manifest
  write after registrar+skills steps.
- Cursor spec: {register_mcp: Some(cursor registrar), skills_subpath: None}.
- Context gains workspace_root (git rev-parse --show-toplevel); readers of
  model assignments resolve via State::load_with_workspace_overrides(path,
  ctx.workspace_root.as_deref()). Writers keep writing the global state file.
- Deleted: AuditStatus::Fail(+fail_count), CodexMcpServer, CursorAdapter,
  OpencodeAdapter (+module), cursor::strip_managed_block,
  opencode::config::apply_model_assignment, profiles::list_snapshots,
  State::{default_model_assignments,merge_overrides stays—wired,
  load_with_workspace_overrides stays—wired}, TUI LOGO,
  HarnessAdapter::{canonical_instruction_file,derived_stub_files}.
