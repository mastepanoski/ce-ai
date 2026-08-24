# Specification: Vercel Labs fx Adapter Audit Refinements

## Requirements

### R1: Deterministic Path Resolution
WHEN `FxAdapter::default_config_path` is called with a directory path,
THEN path resolution SHALL NOT query the filesystem with `.exists()`, relying strictly on basename matching.

### R2: Error Propagation on Unregistration File Removal
WHEN `unregister_fx_mcp_server` removes an empty configuration file,
THEN IO errors other than `ErrorKind::NotFound` SHALL NOT be silenced and SHALL be propagated as `CeError::Io`.

### R3: Environment Variable Resolution Precedence
WHEN `FX_HOME` environment variable is set and non-empty,
THEN `FxAdapter` SHALL resolve its harness directory to `$FX_HOME` instead of `~/.fx`.

### R4: Managed Collision Extra Map Cleanup
WHEN `register_fx_mcp_server` registers a server entry over an existing entry,
THEN stale `type` fields SHALL be removed from `extra` map before setting `"type": "local"`.
