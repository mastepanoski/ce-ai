//! Automated fleet update scheduling across native operating system daemons.
//!
//! Supports macOS LaunchAgents (`launchd`), Linux systemd user timers / crontab,
//! and Windows Task Scheduler (`schtasks.exe`).

use std::path::{Path, PathBuf};

use crate::commands::Context;
use crate::error::CeError;

/// Scheduling interval frequency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleFrequency {
    Daily,
    Weekly,
}

impl ScheduleFrequency {
    pub fn parse(s: &str) -> Result<Self, CeError> {
        match s.trim().to_lowercase().as_str() {
            "daily" => Ok(Self::Daily),
            "weekly" => Ok(Self::Weekly),
            other => Err(CeError::Usage(format!(
                "invalid frequency '{other}': expected 'daily' or 'weekly'"
            ))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Daily => "daily",
            Self::Weekly => "weekly",
        }
    }
}

/// Normalized configuration for scheduling background fleet updates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleConfig {
    pub binary_path: PathBuf,
    pub log_path: PathBuf,
    pub frequency: ScheduleFrequency,
    pub hour: u32,
    pub minute: u32,
}

impl ScheduleConfig {
    pub fn new(
        binary_path: PathBuf,
        log_path: PathBuf,
        frequency_str: &str,
        time_str: &str,
    ) -> Result<Self, CeError> {
        let frequency = ScheduleFrequency::parse(frequency_str)?;
        let (hour, minute) = Self::parse_time(time_str)?;
        Ok(Self {
            binary_path,
            log_path,
            frequency,
            hour,
            minute,
        })
    }

    pub fn parse_time(time_str: &str) -> Result<(u32, u32), CeError> {
        let parts: Vec<&str> = time_str.trim().split(':').collect();
        if parts.len() != 2 {
            return Err(CeError::Usage(format!(
                "invalid time format '{time_str}': expected 'HH:MM' (24-hour clock, e.g. '09:00')"
            )));
        }
        let hour: u32 = parts[0]
            .trim()
            .parse()
            .map_err(|_| CeError::Usage(format!("invalid hour in time '{time_str}'")))?;
        let minute: u32 = parts[1]
            .trim()
            .parse()
            .map_err(|_| CeError::Usage(format!("invalid minute in time '{time_str}'")))?;
        if hour > 23 || minute > 59 {
            return Err(CeError::Usage(format!(
                "time out of range '{time_str}': hour must be 0-23, minute 0-59"
            )));
        }
        Ok((hour, minute))
    }
}

/// Generates a standard crontab entry for Unix / Linux / macOS.
pub fn generate_cron_line(config: &ScheduleConfig) -> String {
    let day_of_week = match config.frequency {
        ScheduleFrequency::Daily => "*",
        ScheduleFrequency::Weekly => "1", // Monday
    };
    format!(
        "{} {} * * {} \"{}\" update -q >> \"{}\" 2>&1",
        config.minute,
        config.hour,
        day_of_week,
        config.binary_path.display(),
        config.log_path.display()
    )
}

/// Generates a macOS `launchd` LaunchAgent XML plist.
pub fn generate_macos_launchd_plist(config: &ScheduleConfig) -> String {
    let weekday_block = match config.frequency {
        ScheduleFrequency::Daily => String::new(),
        ScheduleFrequency::Weekly => {
            "        <key>Weekday</key>\n        <integer>1</integer>\n".to_string()
        }
    };

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.ce-ai.fleet-update</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>update</string>
        <string>-q</string>
    </array>
    <key>StartCalendarInterval</key>
    <dict>
{weekday_block}        <key>Hour</key>
        <integer>{}</integer>
        <key>Minute</key>
        <integer>{}</integer>
    </dict>
    <key>StandardOutPath</key>
    <string>{}</string>
    <key>StandardErrorPath</key>
    <string>{}</string>
</dict>
</plist>
"#,
        config.binary_path.display(),
        config.hour,
        config.minute,
        config.log_path.display(),
        config.log_path.display()
    )
}

/// Generates a Linux `systemd` user service unit file.
pub fn generate_systemd_service(config: &ScheduleConfig) -> String {
    format!(
        r#"[Unit]
Description=Automated CE-AI Fleet Update
After=network-online.target

[Service]
Type=oneshot
ExecStart="{}" update -q
StandardOutput=append:{}
StandardError=append:{}
"#,
        config.binary_path.display(),
        config.log_path.display(),
        config.log_path.display()
    )
}

/// Generates a Linux `systemd` user timer unit file.
pub fn generate_systemd_timer(config: &ScheduleConfig) -> String {
    let calendar = match config.frequency {
        ScheduleFrequency::Daily => format!("*-*-* {:02}:{:02}:00", config.hour, config.minute),
        ScheduleFrequency::Weekly => {
            format!("Mon *-*-* {:02}:{:02}:00", config.hour, config.minute)
        }
    };

    format!(
        r#"[Unit]
Description=Run CE-AI Fleet Update periodically

[Timer]
OnCalendar={}
Persistent=true

[Install]
WantedBy=timers.target
"#,
        calendar
    )
}

/// Generates a Windows `schtasks.exe` creation command.
pub fn generate_windows_schtasks_create(config: &ScheduleConfig) -> String {
    let sc = match config.frequency {
        ScheduleFrequency::Daily => "DAILY",
        ScheduleFrequency::Weekly => "WEEKLY",
    };
    format!(
        "schtasks /create /f /tn \"ce-ai-fleet-update\" /tr \"\\\"{}\\\" update -q\" /sc {} /st {:02}:{:02}",
        config.binary_path.display(),
        sc,
        config.hour,
        config.minute
    )
}

/// Returns the default LaunchAgent plist path on macOS.
pub fn macos_launch_agent_path(home_dir: &Path) -> PathBuf {
    home_dir
        .join("Library")
        .join("LaunchAgents")
        .join("com.ce-ai.fleet-update.plist")
}

/// Returns the systemd user service and timer paths on Linux.
pub fn linux_systemd_paths(home_dir: &Path) -> (PathBuf, PathBuf) {
    let base = home_dir.join(".config").join("systemd").join("user");
    (
        base.join("ce-ai-fleet-update.service"),
        base.join("ce-ai-fleet-update.timer"),
    )
}

/// Detects if a crontab line corresponds to ce-ai fleet update.
pub fn is_ce_ai_cron_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }
    trimmed.contains("ce-ai-fleet-update")
        || (trimmed.contains("ce-ai") && trimmed.contains("update"))
}

/// Filters out ce-ai update lines from a list of crontab lines.
/// Returns (filtered_lines, did_remove).
pub fn filter_crontab_lines(lines: &[String]) -> (Vec<String>, bool) {
    let mut filtered = Vec::new();
    let mut removed = false;
    for line in lines {
        if is_ce_ai_cron_line(line) {
            removed = true;
        } else {
            filtered.push(line.clone());
        }
    }
    (filtered, removed)
}

/// Reads existing crontab lines for the current user.
/// Returns `Ok(lines)` if successful or an empty vec if no crontab exists.
pub fn read_crontab() -> Result<Vec<String>, CeError> {
    let output = std::process::Command::new("crontab")
        .arg("-l")
        .output()
        .map_err(|e| {
            CeError::Io(std::io::Error::new(
                e.kind(),
                format!("failed to execute 'crontab -l': {e}"),
            ))
        })?;

    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout);
        Ok(text.lines().map(|s| s.to_string()).collect())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("no crontab for") {
            Ok(Vec::new())
        } else {
            Err(CeError::Runtime(format!(
                "crontab -l failed with exit code {:?}: {}",
                output.status.code(),
                stderr.trim()
            )))
        }
    }
}

/// Writes crontab lines via a temp file, or runs `crontab -r` if empty.
pub fn write_crontab(lines: &[String]) -> Result<(), CeError> {
    if lines.is_empty() {
        let _ = std::process::Command::new("crontab").arg("-r").status();
        return Ok(());
    }

    let mut temp = tempfile::NamedTempFile::new().map_err(CeError::Io)?;
    use std::io::Write;
    for line in lines {
        writeln!(temp, "{line}").map_err(CeError::Io)?;
    }
    temp.flush().map_err(CeError::Io)?;

    let status = std::process::Command::new("crontab")
        .arg(temp.path())
        .status()
        .map_err(CeError::Io)?;

    if !status.success() {
        return Err(CeError::Runtime(format!(
            "'crontab {}' exited with non-zero code {:?}",
            temp.path().display(),
            status.code()
        )));
    }
    Ok(())
}

/// Installs or updates a crontab entry for ce-ai update.
pub fn install_crontab_entry(config: &ScheduleConfig, dry_run: bool) -> Result<(), CeError> {
    let new_entry = format!("{} # ce-ai-fleet-update", generate_cron_line(config));
    if dry_run {
        println!("[dry-run] would add to crontab:\n  {new_entry}");
        return Ok(());
    }

    match read_crontab() {
        Ok(lines) => {
            let (mut filtered, _) = filter_crontab_lines(&lines);
            filtered.push(new_entry.clone());

            match write_crontab(&filtered) {
                Ok(()) => {
                    println!("Installed crontab schedule entry successfully:");
                    println!("  Entry:    {new_entry}");
                    println!(
                        "  Schedule: {} at {:02}:{:02}",
                        config.frequency.as_str(),
                        config.hour,
                        config.minute
                    );
                    println!("  Logs:     {}", config.log_path.display());
                    Ok(())
                }
                Err(err) => {
                    println!("Notice: automated crontab install failed ({err}). Add manually via 'crontab -e':");
                    println!("  {new_entry}");
                    Ok(())
                }
            }
        }
        Err(err) => {
            println!(
                "Notice: automated crontab read failed ({err}). Add manually via 'crontab -e':"
            );
            println!("  {new_entry}");
            Ok(())
        }
    }
}

/// Removes ce-ai update entries from crontab.
pub fn remove_crontab_entry(dry_run: bool) -> Result<(), CeError> {
    if dry_run {
        println!("[dry-run] would remove 'ce-ai update' entries from crontab");
        return Ok(());
    }

    match read_crontab() {
        Ok(lines) => {
            let (filtered, did_remove) = filter_crontab_lines(&lines);
            if did_remove {
                match write_crontab(&filtered) {
                    Ok(()) => {
                        println!("Removed 'ce-ai update' entry from crontab successfully.");
                        Ok(())
                    }
                    Err(err) => {
                        println!("Notice: failed to write crontab ({err}). Edit manually via 'crontab -e'.");
                        Ok(())
                    }
                }
            } else {
                println!("No 'ce-ai update' entry found in crontab.");
                Ok(())
            }
        }
        Err(err) => {
            println!("Notice: could not read crontab ({err}). Check manually via 'crontab -e'.");
            Ok(())
        }
    }
}

/// CLI arguments for `ce-ai fleet schedule`.
#[derive(clap::Args, Debug, Clone, PartialEq, Eq)]
pub struct ScheduleArgs {
    /// Print schedule configuration without installing or modifying system files.
    #[arg(long)]
    pub print: bool,
    /// Remove/uninstall the scheduled background update task.
    #[arg(long, alias = "uninstall")]
    pub remove: bool,
    /// Target traditional crontab instead of native OS daemon (launchd/systemd).
    #[arg(long)]
    pub cron: bool,
    /// Frequency for the scheduled update (daily, weekly; default: daily).
    #[arg(long, default_value = "daily")]
    pub frequency: String,
    /// Scheduled run time in 24h format HH:MM (default: 09:00).
    #[arg(long, default_value = "09:00")]
    pub time: String,
    /// Preview planned actions without writing.
    #[arg(long)]
    pub dry_run: bool,
}

/// Dispatches the `ce-ai fleet schedule` command.
pub fn handle_schedule(ctx: &Context, args: &ScheduleArgs) -> Result<(), CeError> {
    let effective_dry_run = args.dry_run || ctx.dry_run;

    let binary_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("ce-ai"));
    let log_path = ctx.config_dir.join("update.log");
    let config = ScheduleConfig::new(binary_path, log_path, &args.frequency, &args.time)?;

    // 1. Just print configuration
    if args.print {
        println!(
            "Fleet Update Schedule Configuration ({}, {:02}:{:02}):\n",
            config.frequency.as_str(),
            config.hour,
            config.minute
        );
        if args.cron {
            println!(
                "# Crontab entry (add via 'crontab -e'):\n{}",
                generate_cron_line(&config)
            );
        } else if cfg!(target_os = "macos") {
            println!("# macOS LaunchAgent Plist (~/Library/LaunchAgents/com.ce-ai.fleet-update.plist):\n{}", generate_macos_launchd_plist(&config));
            println!("# Crontab equivalent:\n{}", generate_cron_line(&config));
        } else if cfg!(target_os = "windows") {
            println!(
                "# Windows Task Scheduler command:\n{}",
                generate_windows_schtasks_create(&config)
            );
        } else {
            println!("# Linux systemd service unit (~/.config/systemd/user/ce-ai-fleet-update.service):\n{}", generate_systemd_service(&config));
            println!(
                "# Linux systemd timer unit (~/.config/systemd/user/ce-ai-fleet-update.timer):\n{}",
                generate_systemd_timer(&config)
            );
            println!("# Crontab equivalent:\n{}", generate_cron_line(&config));
        }
        return Ok(());
    }

    // 2. Remove scheduled task
    if args.remove {
        if args.cron {
            remove_crontab_entry(effective_dry_run)?;
            return Ok(());
        }

        if cfg!(target_os = "macos") {
            let plist_path = macos_launch_agent_path(&ctx.home_dir());
            if plist_path.exists() {
                if effective_dry_run {
                    println!(
                        "[dry-run] would unload and delete LaunchAgent at {}",
                        plist_path.display()
                    );
                } else {
                    let _ = std::process::Command::new("launchctl")
                        .args(["unload", &plist_path.to_string_lossy()])
                        .status();
                    let _ = std::fs::remove_file(&plist_path);
                    println!("Removed macOS LaunchAgent: {}", plist_path.display());
                }
            } else {
                println!("No LaunchAgent found at {}", plist_path.display());
            }
        } else if cfg!(target_os = "windows") {
            if effective_dry_run {
                println!("[dry-run] would run 'schtasks /delete /f /tn \"ce-ai-fleet-update\"'");
            } else {
                let status = std::process::Command::new("schtasks")
                    .args(["/delete", "/f", "/tn", "ce-ai-fleet-update"])
                    .status();
                match status {
                    Ok(s) if s.success() => {
                        println!("Removed Windows scheduled task 'ce-ai-fleet-update'.")
                    }
                    _ => println!(
                        "Notice: Task Scheduler command completed or task was not present."
                    ),
                }
            }
        } else {
            let (svc_path, timer_path) = linux_systemd_paths(&ctx.home_dir());
            if timer_path.exists() || svc_path.exists() {
                if effective_dry_run {
                    println!("[dry-run] would disable and remove systemd timer and service units");
                } else {
                    let _ = std::process::Command::new("systemctl")
                        .args(["--user", "disable", "--now", "ce-ai-fleet-update.timer"])
                        .status();
                    let _ = std::fs::remove_file(&timer_path);
                    let _ = std::fs::remove_file(&svc_path);
                    let _ = std::process::Command::new("systemctl")
                        .args(["--user", "daemon-reload"])
                        .status();
                    println!("Removed systemd user units for ce-ai-fleet-update.");
                }
            } else {
                println!("No systemd user units found.");
            }
        }

        // Also clean up any crontab entries if present
        let _ = remove_crontab_entry(effective_dry_run);
        return Ok(());
    }

    // 3. Install / configure schedule
    if args.cron {
        return install_crontab_entry(&config, effective_dry_run);
    }

    if cfg!(target_os = "macos") {
        let plist_path = macos_launch_agent_path(&ctx.home_dir());
        let plist_content = generate_macos_launchd_plist(&config);

        if effective_dry_run {
            println!(
                "[dry-run] would write LaunchAgent to {}",
                plist_path.display()
            );
            println!(
                "[dry-run] would run 'launchctl load -w {}'",
                plist_path.display()
            );
        } else {
            if let Some(parent) = plist_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            crate::state::write_atomic(&plist_path, plist_content.as_bytes())?;
            let _ = std::process::Command::new("launchctl")
                .args(["unload", &plist_path.to_string_lossy()])
                .status();
            let load_status = std::process::Command::new("launchctl")
                .args(["load", "-w", &plist_path.to_string_lossy()])
                .status();

            println!("Configured macOS LaunchAgent successfully:");
            println!("  Plist:    {}", plist_path.display());
            println!(
                "  Schedule: {} at {:02}:{:02}",
                config.frequency.as_str(),
                config.hour,
                config.minute
            );
            println!("  Command:  {} update -q", config.binary_path.display());
            println!("  Logs:     {}", config.log_path.display());

            if let Ok(st) = load_status {
                if !st.success() {
                    println!("  Notice: 'launchctl load' returned exit code {:?}; verify permissions or load manually with:", st.code());
                    println!("    launchctl load -w {}", plist_path.display());
                }
            }
        }
    } else if cfg!(target_os = "windows") {
        let cmd_str = generate_windows_schtasks_create(&config);
        if effective_dry_run {
            println!("[dry-run] would execute: {cmd_str}");
        } else {
            println!("Registering Windows Task Scheduler job...");
            let sc = if config.frequency == ScheduleFrequency::Daily {
                "DAILY"
            } else {
                "WEEKLY"
            };
            let time_val = format!("{:02}:{:02}", config.hour, config.minute);
            let tr_val = format!("\"{}\" update -q", config.binary_path.display());

            let status = std::process::Command::new("schtasks")
                .args([
                    "/create",
                    "/f",
                    "/tn",
                    "ce-ai-fleet-update",
                    "/tr",
                    &tr_val,
                    "/sc",
                    sc,
                    "/st",
                    &time_val,
                ])
                .status();

            match status {
                Ok(s) if s.success() => {
                    println!("Configured Windows Task Scheduler task 'ce-ai-fleet-update' successfully ({} at {}).", config.frequency.as_str(), time_val);
                }
                _ => {
                    println!("Failed to run schtasks automatically. Execute manually in an elevated prompt:");
                    println!("  {cmd_str}");
                }
            }
        }
    } else {
        // Linux / systemd
        let (svc_path, timer_path) = linux_systemd_paths(&ctx.home_dir());
        let svc_content = generate_systemd_service(&config);
        let timer_content = generate_systemd_timer(&config);

        if effective_dry_run {
            println!(
                "[dry-run] would write systemd service to {}",
                svc_path.display()
            );
            println!(
                "[dry-run] would write systemd timer to {}",
                timer_path.display()
            );
            println!("[dry-run] would run 'systemctl --user daemon-reload && systemctl --user enable --now ce-ai-fleet-update.timer'");
        } else {
            if let Some(parent) = svc_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            crate::state::write_atomic(&svc_path, svc_content.as_bytes())?;
            crate::state::write_atomic(&timer_path, timer_content.as_bytes())?;

            let _ = std::process::Command::new("systemctl")
                .args(["--user", "daemon-reload"])
                .status();
            let enable_status = std::process::Command::new("systemctl")
                .args(["--user", "enable", "--now", "ce-ai-fleet-update.timer"])
                .status();

            println!("Configured Linux systemd user timer successfully:");
            println!("  Timer:    {}", timer_path.display());
            println!("  Service:  {}", svc_path.display());
            println!(
                "  Schedule: {} at {:02}:{:02}",
                config.frequency.as_str(),
                config.hour,
                config.minute
            );
            println!("  Logs:     {}", config.log_path.display());

            if let Ok(st) = enable_status {
                if !st.success() {
                    println!("  Notice: 'systemctl' returned exit code {:?}; if systemd user session is inactive, use crontab instead:", st.code());
                    println!("    ce-ai fleet schedule --cron");
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_frequency_parse() {
        assert_eq!(
            ScheduleFrequency::parse("daily").unwrap(),
            ScheduleFrequency::Daily
        );
        assert_eq!(
            ScheduleFrequency::parse("DAILY").unwrap(),
            ScheduleFrequency::Daily
        );
        assert_eq!(
            ScheduleFrequency::parse("weekly").unwrap(),
            ScheduleFrequency::Weekly
        );
        assert_eq!(
            ScheduleFrequency::parse("WEEKLY").unwrap(),
            ScheduleFrequency::Weekly
        );
        assert!(ScheduleFrequency::parse("monthly").is_err());
    }

    #[test]
    fn test_schedule_time_parse() {
        assert_eq!(ScheduleConfig::parse_time("09:00").unwrap(), (9, 0));
        assert_eq!(ScheduleConfig::parse_time("23:59").unwrap(), (23, 59));
        assert_eq!(ScheduleConfig::parse_time("0:0").unwrap(), (0, 0));
        assert!(ScheduleConfig::parse_time("24:00").is_err());
        assert!(ScheduleConfig::parse_time("12:60").is_err());
        assert!(ScheduleConfig::parse_time("invalid").is_err());
        assert!(ScheduleConfig::parse_time("12").is_err());
    }

    #[test]
    fn test_generate_cron_line() {
        let config = ScheduleConfig {
            binary_path: PathBuf::from("/usr/local/bin/ce-ai"),
            log_path: PathBuf::from("/home/user/.ce-ai/update.log"),
            frequency: ScheduleFrequency::Daily,
            hour: 9,
            minute: 30,
        };
        let cron = generate_cron_line(&config);
        assert_eq!(
            cron,
            "30 9 * * * \"/usr/local/bin/ce-ai\" update -q >> \"/home/user/.ce-ai/update.log\" 2>&1"
        );

        let weekly_config = ScheduleConfig {
            frequency: ScheduleFrequency::Weekly,
            ..config
        };
        let weekly_cron = generate_cron_line(&weekly_config);
        assert_eq!(
            weekly_cron,
            "30 9 * * 1 \"/usr/local/bin/ce-ai\" update -q >> \"/home/user/.ce-ai/update.log\" 2>&1"
        );
    }

    #[test]
    fn test_generate_macos_launchd_plist() {
        let config = ScheduleConfig {
            binary_path: PathBuf::from("/opt/homebrew/bin/ce-ai"),
            log_path: PathBuf::from("/Users/test/.ce-ai/update.log"),
            frequency: ScheduleFrequency::Daily,
            hour: 8,
            minute: 15,
        };
        let plist = generate_macos_launchd_plist(&config);
        assert!(plist.contains("<string>com.ce-ai.fleet-update</string>"));
        assert!(plist.contains("<string>/opt/homebrew/bin/ce-ai</string>"));
        assert!(plist.contains("<string>update</string>"));
        assert!(plist.contains("<string>-q</string>"));
        assert!(plist.contains("<integer>8</integer>"));
        assert!(plist.contains("<integer>15</integer>"));
        assert!(plist.contains("<string>/Users/test/.ce-ai/update.log</string>"));
    }

    #[test]
    fn test_generate_systemd_units() {
        let config = ScheduleConfig {
            binary_path: PathBuf::from("/usr/bin/ce-ai"),
            log_path: PathBuf::from("/home/test/.ce-ai/update.log"),
            frequency: ScheduleFrequency::Daily,
            hour: 9,
            minute: 0,
        };
        let svc = generate_systemd_service(&config);
        assert!(svc.contains("ExecStart=\"/usr/bin/ce-ai\" update -q"));
        let timer = generate_systemd_timer(&config);
        assert!(timer.contains("OnCalendar=*-*-* 09:00:00"));
    }

    #[test]
    fn test_generate_windows_schtasks() {
        let config = ScheduleConfig {
            binary_path: PathBuf::from("C:\\Tools\\ce-ai.exe"),
            log_path: PathBuf::from("C:\\Users\\test\\.ce-ai\\update.log"),
            frequency: ScheduleFrequency::Weekly,
            hour: 10,
            minute: 30,
        };
        let cmd = generate_windows_schtasks_create(&config);
        assert!(cmd.contains("schtasks /create /f /tn \"ce-ai-fleet-update\""));
        assert!(cmd.contains("/sc WEEKLY"));
        assert!(cmd.contains("/st 10:30"));
    }

    #[test]
    fn test_is_ce_ai_cron_line() {
        assert!(is_ce_ai_cron_line("0 9 * * * /usr/bin/ce-ai update -q"));
        assert!(is_ce_ai_cron_line(
            "30 8 * * 1 \"/path/ce-ai\" update -q >> log 2>&1 # ce-ai-fleet-update"
        ));
        assert!(is_ce_ai_cron_line(
            "0 0 * * * /bin/custom # ce-ai-fleet-update"
        ));
        assert!(!is_ce_ai_cron_line("0 2 * * * /usr/bin/backup.sh"));
        assert!(!is_ce_ai_cron_line("# daily backup comment"));
        assert!(!is_ce_ai_cron_line(""));
    }

    #[test]
    fn test_filter_crontab_lines() {
        let lines = vec![
            "# crontab header".to_string(),
            "0 2 * * * /usr/bin/backup.sh".to_string(),
            "0 9 * * * /usr/local/bin/ce-ai update -q >> log 2>&1 # ce-ai-fleet-update".to_string(),
            "# footer".to_string(),
        ];
        let (filtered, did_remove) = filter_crontab_lines(&lines);
        assert!(did_remove);
        assert_eq!(filtered.len(), 3);
        assert_eq!(filtered[0], "# crontab header");
        assert_eq!(filtered[1], "0 2 * * * /usr/bin/backup.sh");
        assert_eq!(filtered[2], "# footer");

        let no_match = vec!["0 2 * * * /usr/bin/backup.sh".to_string()];
        let (filtered2, did_remove2) = filter_crontab_lines(&no_match);
        assert!(!did_remove2);
        assert_eq!(filtered2.len(), 1);
    }
}
