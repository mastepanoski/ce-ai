//! TUI tab definitions (KTD3, R5) — extracted from monolithic tui.rs.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuTab {
    Status,
    Workflow,
    Install,
    Models,
    Skills,
    Sync,
    Upgrade,
    Doctor,
    Backups,
    Tools,
    Usage,
    Audit,
    InitPrj,
    Uninstall,
    Exit,
}

impl MenuTab {
    pub fn all() -> Vec<Self> {
        vec![
            MenuTab::Status,
            MenuTab::Workflow,
            MenuTab::Install,
            MenuTab::Models,
            MenuTab::Skills,
            MenuTab::Sync,
            MenuTab::Upgrade,
            MenuTab::Doctor,
            MenuTab::Backups,
            MenuTab::Tools,
            MenuTab::Usage,
            MenuTab::Audit,
            MenuTab::InitPrj,
            MenuTab::Uninstall,
            MenuTab::Exit,
        ]
    }

    pub fn title(&self) -> &'static str {
        match self {
            MenuTab::Status => "📊  Status & Harnesses",
            MenuTab::Workflow => "🎮  Workflow (FSM)",
            MenuTab::Install => "📥  Install Plugin",
            MenuTab::Models => "🤖  Models & Profiles",
            MenuTab::Skills => "🧩  Skills Registry",
            MenuTab::Sync => "🔄  Sync & Reconcile",
            MenuTab::Upgrade => "🚀  Upgrade Release",
            MenuTab::Doctor => "🩺  Health Doctor",
            MenuTab::Backups => "💾  Backups & Restore",
            MenuTab::Tools => "🛠️  Tools & Sidecars",
            MenuTab::Usage => "📈  Usage Analytics",
            MenuTab::Audit => "🔍  Audit & Quality",
            MenuTab::InitPrj => "📁  Project Adopt",
            MenuTab::Uninstall => "🗑️   Uninstall Plugin",
            MenuTab::Exit => "❌  Quit Dashboard",
        }
    }
}
