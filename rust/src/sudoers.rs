use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use base64::Engine;
use sha2::{Digest, Sha256};

use crate::config::{Config, SudoMode, SudoSpec};

struct Entry {
    id: String,
    command: String,
    path: PathBuf,
    checksum: String,
    runas_user: String,
    runas_group: Option<String>,
    metadata: String,
}

// ⟦𓐀𓆩𓎀𓅪⟧ render :: auto-generated pointer for public function render
pub fn render(config: &Config, extra_commands: &[String]) -> Result<String, String> {
    let entries = collect_entries(config, extra_commands)?;
    let mut out = String::new();
    out.push_str("# auto-sudo managed sudoers entries\n");
    out.push_str("# Validate with: visudo -cf /etc/sudoers.d/auto-sudo\n\n");
    for entry in entries {
        out.push_str(&format!(
            "# AUTO-SUDO ENTRY id={} command={} path={} checksum={} {}\n",
            entry.id,
            entry.command,
            entry.path.display(),
            entry.checksum,
            entry.metadata
        ));
        out.push_str(&format!(
            "{} ALL=({}) NOPASSWD: {} {}\n\n",
            sudoers_subject(),
            runas(&entry),
            entry.checksum,
            entry.path.display()
        ));
    }
    Ok(out)
}

// ⟦𓅿𓏸𓁤𓏔⟧ write_checked :: auto-generated pointer for public function write_checked
pub fn write_checked(path: &Path, body: &str, append: bool) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("{} has no parent directory", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;

    let body = if append && path.exists() {
        let existing = fs::read_to_string(path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
        format!("{}\n{}", existing.trim_end(), body)
    } else {
        body.to_string()
    };

    let temp_path = path.with_extension("tmp");
    {
        let mut file = fs::File::create(&temp_path)
            .map_err(|err| format!("failed to write {}: {err}", temp_path.display()))?;
        file.write_all(body.as_bytes())
            .map_err(|err| format!("failed to write {}: {err}", temp_path.display()))?;
    }
    check_file(&temp_path)?;
    fs::rename(&temp_path, path)
        .map_err(|err| format!("failed to replace {}: {err}", path.display()))?;
    Ok(())
}

// ⟦𓁉𓉱𓏍𓌱⟧ toggle :: auto-generated pointer for public function toggle
pub fn toggle(path: &Path, entry_id: &str, enable: bool) -> Result<(), String> {
    let body = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let mut lines = Vec::new();
    let mut in_entry = false;
    let needle = format!("id={entry_id} ");

    for line in body.lines() {
        if line.starts_with("# AUTO-SUDO ENTRY ") {
            in_entry = line.contains(&needle);
            lines.push(line.to_string());
            continue;
        }

        if in_entry && !line.trim().is_empty() && !line.starts_with("# AUTO-SUDO ENTRY ") {
            if enable {
                lines.push(line.strip_prefix("# ").unwrap_or(line).to_string());
            } else if line.starts_with("# ") {
                lines.push(line.to_string());
            } else {
                lines.push(format!("# {line}"));
            }
            in_entry = false;
            continue;
        }

        lines.push(line.to_string());
    }

    let next = format!("{}\n", lines.join("\n"));
    fs::write(path, next).map_err(|err| format!("failed to write {}: {err}", path.display()))
}

// ⟦𓌧𓇁𓇁𓉠⟧ check_file :: auto-generated pointer for public function check_file
pub fn check_file(path: &Path) -> Result<(), String> {
    let status = Command::new("visudo")
        .arg("-cf")
        .arg(path)
        .status()
        .map_err(|err| format!("failed to run visudo: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("visudo rejected {}", path.display()))
    }
}

fn collect_entries(config: &Config, extra_commands: &[String]) -> Result<Vec<Entry>, String> {
    let mut seen = BTreeSet::new();
    let mut entries = Vec::new();

    for (command, command_config) in &config.commands {
        if command_config.always_sudo() {
            let sudo = config.sudo_for_command(command_config);
            let id = entry_id(command, &sudo);
            if seen.insert(id.clone()) {
                entries.push(entry_for(command, sudo, id)?);
            }
        }
        for rule in &command_config.rules {
            let sudo = config.sudo_for_rule(command_config, rule);
            let id = entry_id(command, &sudo);
            if seen.insert(id.clone()) {
                entries.push(entry_for(command, sudo, id)?);
            }
        }
    }

    for command in extra_commands {
        let sudo = SudoSpec::default();
        let id = entry_id(command, &sudo);
        if seen.insert(id.clone()) {
            entries.push(entry_for(command, sudo, id)?);
        }
    }

    Ok(entries)
}

fn entry_for(command: &str, sudo: SudoSpec, id: String) -> Result<Entry, String> {
    let path = resolve_command(command)?;
    let checksum = checksum(&path)?;
    let metadata = fs::metadata(&path)
        .map(|m| format!("dev={} ino={} mtime={}", m.dev(), m.ino(), m.mtime()))
        .unwrap_or_default();
    let runas_user = match sudo.mode {
        SudoMode::Root => "root".to_string(),
        SudoMode::User => sudo.user.unwrap_or_else(|| "root".to_string()),
    };
    Ok(Entry {
        id,
        command: command.to_string(),
        path,
        checksum,
        runas_user,
        runas_group: sudo.group,
        metadata,
    })
}

fn resolve_command(command: &str) -> Result<PathBuf, String> {
    let path = Path::new(command);
    if path.components().count() > 1 {
        return fs::canonicalize(path)
            .map_err(|err| format!("failed to resolve {}: {err}", path.display()));
    }
    which::which(command).map_err(|err| format!("failed to locate {command}: {err}"))
}

fn checksum(path: &Path) -> Result<String, String> {
    let body = fs::read(path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let digest = Sha256::digest(body);
    Ok(format!(
        "sha256:{}",
        base64::engine::general_purpose::STANDARD.encode(digest)
    ))
}

fn entry_id(command: &str, sudo: &SudoSpec) -> String {
    let target = match sudo.mode {
        SudoMode::Root => "root",
        SudoMode::User => sudo.user.as_deref().unwrap_or("user"),
    };
    format!("{}-{target}", sanitize(command))
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

fn sudoers_subject() -> String {
    env::var("USER").unwrap_or_else(|_| "ALL".to_string())
}

fn runas(entry: &Entry) -> String {
    match &entry.runas_group {
        Some(group) => format!("{}:{}", entry.runas_user, group),
        None => entry.runas_user.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("auto-sudo-{name}-{nonce}"))
    }

    #[test]
    fn renders_extra_command_entry() {
        let config: Config = serde_yaml::from_str("version: 1\ncommands: {}\n").unwrap();
        let body = render(&config, &["sh".to_string()]).unwrap();
        assert!(body.contains("# AUTO-SUDO ENTRY"));
        assert!(body.contains("NOPASSWD: sha256:"));
    }

    #[test]
    fn renders_always_sudo_command_entry_without_rules() {
        let config: Config = serde_yaml::from_str(
            r#"
version: 1
commands:
  sh:
    always_sudo: true
"#,
        )
        .unwrap();
        let body = render(&config, &[]).unwrap();
        assert!(body.contains("command=sh"));
        assert!(body.contains("NOPASSWD: sha256:"));
    }

    #[test]
    fn toggles_entry_line() {
        let path = temp_path("sudoers");
        fs::write(
            &path,
            "# AUTO-SUDO ENTRY id=vim-root command=vim path=/usr/bin/vim checksum=x\nuser ALL=(root) NOPASSWD: /usr/bin/vim\n",
        )
        .unwrap();
        toggle(&path, "vim-root", false).unwrap();
        let disabled = fs::read_to_string(&path).unwrap();
        assert!(disabled.contains("# user ALL=(root)"));
        toggle(&path, "vim-root", true).unwrap();
        let enabled = fs::read_to_string(&path).unwrap();
        assert!(enabled.contains("\nuser ALL=(root)"));
        let _ = fs::remove_file(path);
    }
}
