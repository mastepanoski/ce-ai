# Design
Pure builders in src/tui.rs: status_args, install_cmd_args(harness,dry),
models_list_args, sync_cmd_args(dry), workflow_status_args, upgrade_cmd_args,
doctor_cmd_args, uninstall_cmd_args(harness), init_prj_args. run_* wrappers
call capture_cli(&builder()). Contract test augments each module's Args with
globals and parses the verb-stripped vector; upgrade dead-flag rejection is
pinned separately (ErrorKind::UnknownArgument).
