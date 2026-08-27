//! TUI application state (KTD3, R5) — extracted from monolithic tui.rs.

use crate::commands::Context;
use crate::harness::HarnessKind;
use crate::state::state::State;

use super::tabs::MenuTab;

pub struct App {
    pub selected_tab: usize,
    pub dry_run: bool,
    pub harnesses: Vec<(String, String, String)>,
    pub detected_harnesses: Vec<HarnessKind>,
    pub model_scope: Vec<(String, String)>,
    pub model_slots: Vec<String>,
    pub selected_model_idx: usize,
    pub model_picker_open: bool,
    pub picker_items: Vec<String>,
    pub picker_selected: usize,
    pub output_modal: Option<(String, Vec<String>)>,
    pub output_scroll: usize,
    pub selected_harness_idx: usize,
    pub harness_targets: Vec<String>,
    pub selected_backup_idx: usize,
    pub backups: Vec<crate::state::backups::BackupEntry>,
    pub state_error: Option<String>,
}

impl App {
    pub fn new(ctx: &Context) -> Self {
        let mut app = Self {
            selected_tab: 0,
            dry_run: ctx.dry_run,
            harnesses: Vec::new(),
            detected_harnesses: Vec::new(),
            model_scope: Vec::new(),
            model_slots: Vec::new(),
            selected_model_idx: 0,
            model_picker_open: false,
            picker_items: Vec::new(),
            picker_selected: 0,
            output_modal: None,
            output_scroll: 0,
            selected_harness_idx: 0,
            selected_backup_idx: 0,
            backups: Vec::new(),
            state_error: None,
            harness_targets: vec![
                "all".into(),
                "opencode".into(),
                "claude".into(),
                "pi".into(),
                "cursor".into(),
                "copilot".into(),
                "codex".into(),
                "grok".into(),
                "kimi".into(),
                "agy".into(),
                "fx".into(),
                "custom".into(),
            ],
        };
        app.reload_state(ctx);
        app
    }

    pub fn selected_harness_target(&self) -> &str {
        self.harness_targets
            .get(self.selected_harness_idx)
            .map(|s| s.as_str())
            .unwrap_or("all")
    }

    pub fn next_harness(&mut self) {
        if self.selected_harness_idx + 1 < self.harness_targets.len() {
            self.selected_harness_idx += 1;
        } else {
            self.selected_harness_idx = 0;
        }
    }

    pub fn prev_harness(&mut self) {
        if self.selected_harness_idx > 0 {
            self.selected_harness_idx -= 1;
        } else {
            self.selected_harness_idx = self.harness_targets.len() - 1;
        }
    }

    pub fn reload_state(&mut self, ctx: &Context) {
        self.harnesses.clear();
        self.detected_harnesses.clear();
        self.model_scope.clear();
        self.model_slots.clear();

        if let Ok(home) = std::env::var("HOME") {
            self.detected_harnesses =
                HarnessKind::detect_installed_harnesses(std::path::Path::new(&home));
        }

        let mut seen = std::collections::HashSet::new();

        let state_path = ctx.config_dir.join("state.json");
        match State::load(&state_path) {
            Ok(state) => {
                for h in &state.installed_harnesses {
                    let name = h["name"].as_str().unwrap_or("unknown").to_string();
                    let version = h["version"].as_str().unwrap_or("unknown").to_string();
                    let source = h["source"]["kind"].as_str().unwrap_or("local").to_string();
                    seen.insert(name.clone());
                    self.harnesses.push((name, version, source));
                }
                self.state_error = None;
            }
            Err(e) => {
                self.state_error = Some(format!("{e}"));
            }
        }

        let scope = self.selected_harness_target().to_string();
        self.model_scope = crate::commands::models::config_assignments(ctx, &scope);

        for slot in crate::harness::agents::CE_AGENT_SLOTS {
            self.model_slots.push(slot.to_string());
        }
        for (slot, _) in &self.model_scope.clone() {
            if !self.model_slots.contains(slot) {
                self.model_slots.push(slot.clone());
            }
        }
        if self.selected_model_idx >= self.model_slots.len() {
            self.selected_model_idx = self.model_slots.len().saturating_sub(1);
        }

        if let Ok(home) = std::env::var("HOME") {
            let home_path = std::path::Path::new(&home);
            for h in HarnessKind::detect_ce_installed_harnesses(home_path) {
                let name = h.to_string();
                if !seen.contains(&name) {
                    seen.insert(name.clone());
                    self.harnesses
                        .push((name, "host-detected".to_string(), "local".to_string()));
                }
            }
        }

        let backups_dir = ctx.config_dir.join("backups");
        let filter = self.selected_harness_target();
        self.backups =
            crate::state::backups::list_backups(&backups_dir, Some(filter)).unwrap_or_default();
        if self.selected_backup_idx >= self.backups.len() && !self.backups.is_empty() {
            self.selected_backup_idx = self.backups.len() - 1;
        }
    }

    pub fn current_tab(&self) -> MenuTab {
        MenuTab::all()[self.selected_tab]
    }
}
