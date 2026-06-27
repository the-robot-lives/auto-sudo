use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use crate::config::{Config, FileArgSpec, FileChecks, PositionSpec, Rule, SudoMode, SudoSpec};

pub struct DecisionRequest<'a> {
    pub command: &'a str,
    pub args: &'a [String],
    pub stdin_piped: bool,
    pub stdout_piped: bool,
}

pub struct Decision {
    pub prefix: String,
    pub reason: String,
}

pub fn decide(config: &Config, request: &DecisionRequest<'_>) -> Result<Decision, String> {
    let Some(command_config) = config.commands.get(request.command) else {
        return Ok(Decision {
            prefix: String::new(),
            reason: format!("no config for command {}", request.command),
        });
    };

    let pipe_detected = request.stdin_piped || request.stdout_piped;
    if pipe_detected && !command_config.allow_pipes(&config.defaults) {
        return Ok(Decision {
            prefix: String::new(),
            reason: format!("pipes are disabled for {}", request.command),
        });
    }

    if command_config.always_sudo() {
        let sudo = config.sudo_for_command(command_config);
        return Ok(Decision {
            prefix: render_prefix(&sudo)?,
            reason: format!("always_sudo enabled for {}", request.command),
        });
    }

    for rule in &command_config.rules {
        if rule_matches(rule, request)? {
            let sudo = config.sudo_for_rule(command_config, rule);
            return Ok(Decision {
                prefix: render_prefix(&sudo)?,
                reason: format!(
                    "matched rule {} for {}",
                    rule.name.as_deref().unwrap_or("<unnamed>"),
                    request.command
                ),
            });
        }
    }

    Ok(Decision {
        prefix: String::new(),
        reason: format!("no matching rule for {}", request.command),
    })
}

fn rule_matches(rule: &Rule, request: &DecisionRequest<'_>) -> Result<bool, String> {
    if rule.when.always.unwrap_or(false) {
        return Ok(true);
    }

    let files = extract_files(&rule.args.files, request.args);
    if let Some(checks) = &rule.when.any_file {
        return Ok(files
            .iter()
            .any(|file| checks_match(file, checks).unwrap_or(false)));
    }
    if let Some(checks) = &rule.when.all_files {
        return Ok(!files.is_empty()
            && files
                .iter()
                .all(|file| checks_match(file, checks).unwrap_or(false)));
    }
    Ok(false)
}

fn extract_files(specs: &[FileArgSpec], args: &[String]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for spec in specs {
        if let Some(flag) = &spec.flag {
            out.extend(
                extract_flag_values(flag, args)
                    .into_iter()
                    .map(PathBuf::from),
            );
        }
        if let Some(position) = &spec.position {
            match position {
                PositionSpec::Index(index) => {
                    if let Some(value) = args.get(*index) {
                        if !skip_arg(value, &spec.skip_prefixes) {
                            out.push(PathBuf::from(value));
                        }
                    }
                }
                PositionSpec::Any(_) if position.is_any() => {
                    out.extend(
                        args.iter()
                            .filter(|arg| !skip_arg(arg, &spec.skip_prefixes))
                            .map(PathBuf::from),
                    );
                }
                PositionSpec::Any(_) => {}
            }
        }
    }
    out
}

fn extract_flag_values(flag: &str, args: &[String]) -> Vec<String> {
    let mut values = Vec::new();
    let eq_prefix = format!("{flag}=");
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if let Some(value) = arg.strip_prefix(&eq_prefix) {
            values.push(value.to_string());
        } else if arg == flag {
            if let Some(value) = args.get(i + 1) {
                values.push(value.clone());
                i += 1;
            }
        }
        i += 1;
    }
    values
}

fn skip_arg(arg: &str, skip_prefixes: &[String]) -> bool {
    skip_prefixes.iter().any(|prefix| arg.starts_with(prefix))
}

fn checks_match(path: &Path, checks: &FileChecks) -> Result<bool, String> {
    if !path_filters_match(path, checks) {
        return Ok(false);
    }

    let metadata = fs::metadata(path).ok();
    let exists = metadata.is_some();

    if checks.exists && !exists {
        return Ok(false);
    }
    if checks.missing && exists {
        return Ok(false);
    }

    if checks.exists_not_writable && (!exists || can_access(path, Access::Write)) {
        return Ok(false);
    }
    if checks.missing_parent_not_writable {
        if exists {
            return Ok(false);
        }
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        if can_access(parent, Access::Write) {
            return Ok(false);
        }
    }

    if checks.current_user_can_read && !can_access(path, Access::Read) {
        return Ok(false);
    }
    if checks.current_user_can_write && !can_access(path, Access::Write) {
        return Ok(false);
    }
    if checks.current_user_can_execute && !can_access(path, Access::Execute) {
        return Ok(false);
    }
    if checks.current_user_cannot_read && can_access(path, Access::Read) {
        return Ok(false);
    }
    if checks.current_user_cannot_write && can_access(path, Access::Write) {
        return Ok(false);
    }
    if checks.current_user_cannot_execute && can_access(path, Access::Execute) {
        return Ok(false);
    }

    if checks.owner_is_current_user || checks.owner_is_not_current_user {
        let Some(metadata) = &metadata else {
            return Ok(false);
        };
        let owned = metadata.uid() == current_uid();
        if checks.owner_is_current_user && !owned {
            return Ok(false);
        }
        if checks.owner_is_not_current_user && owned {
            return Ok(false);
        }
    }

    if checks.group_in_current_user_groups || checks.group_not_in_current_user_groups {
        let Some(metadata) = &metadata else {
            return Ok(false);
        };
        let in_group = current_groups().contains(&metadata.gid());
        if checks.group_in_current_user_groups && !in_group {
            return Ok(false);
        }
        if checks.group_not_in_current_user_groups && in_group {
            return Ok(false);
        }
    }

    Ok(true)
}

fn path_filters_match(path: &Path, checks: &FileChecks) -> bool {
    let value = path.to_string_lossy();
    if !checks.paths.is_empty()
        && !checks
            .paths
            .iter()
            .any(|pattern| wildcard_match(pattern, &value))
    {
        return false;
    }
    if !checks.path_prefixes.is_empty()
        && !checks
            .path_prefixes
            .iter()
            .any(|prefix| value.starts_with(prefix))
    {
        return false;
    }
    if !checks.path_suffixes.is_empty()
        && !checks
            .path_suffixes
            .iter()
            .any(|suffix| value.ends_with(suffix))
    {
        return false;
    }
    true
}

fn wildcard_match(pattern: &str, value: &str) -> bool {
    let pattern = pattern.as_bytes();
    let value = value.as_bytes();
    let (mut p, mut v) = (0, 0);
    let mut star = None;
    let mut star_value = 0;

    while v < value.len() {
        if p < pattern.len() && (pattern[p] == b'?' || pattern[p] == value[v]) {
            p += 1;
            v += 1;
        } else if p < pattern.len() && pattern[p] == b'*' {
            star = Some(p);
            p += 1;
            star_value = v;
        } else if let Some(star_pos) = star {
            p = star_pos + 1;
            star_value += 1;
            v = star_value;
        } else {
            return false;
        }
    }

    while p < pattern.len() && pattern[p] == b'*' {
        p += 1;
    }
    p == pattern.len()
}

fn render_prefix(sudo: &SudoSpec) -> Result<String, String> {
    let mut parts = vec!["sudo".to_string()];
    match sudo.mode {
        SudoMode::Root => {}
        SudoMode::User => {
            let user = sudo
                .user
                .as_deref()
                .ok_or_else(|| "sudo mode user requires action.sudo.user".to_string())?;
            parts.push("-u".to_string());
            parts.push(shell_word(user));
        }
    }
    if let Some(group) = &sudo.group {
        parts.push("-g".to_string());
        parts.push(shell_word(group));
    }
    Ok(format!("{} ", parts.join(" ")))
}

fn shell_word(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '/' | ':'))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

#[derive(Copy, Clone)]
enum Access {
    Read,
    Write,
    Execute,
}

fn can_access(path: &Path, access: Access) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    let mode = metadata.mode();
    let uid = current_uid();
    if uid == 0 {
        return !matches!(access, Access::Execute) || mode & 0o111 != 0;
    }

    let bit = match access {
        Access::Read => 0o4,
        Access::Write => 0o2,
        Access::Execute => 0o1,
    };

    if metadata.uid() == uid {
        mode & (bit << 6) != 0
    } else if current_groups().contains(&metadata.gid()) {
        mode & (bit << 3) != 0
    } else {
        mode & bit != 0
    }
}

fn current_uid() -> u32 {
    unsafe { libc::getuid() as u32 }
}

fn current_groups() -> Vec<u32> {
    let count = unsafe { libc::getgroups(0, std::ptr::null_mut()) };
    if count <= 0 {
        return vec![unsafe { libc::getgid() as u32 }];
    }
    let mut groups = vec![0 as libc::gid_t; count as usize];
    let actual = unsafe { libc::getgroups(count, groups.as_mut_ptr()) };
    groups.truncate(actual.max(0) as usize);
    let primary = unsafe { libc::getgid() as u32 };
    let mut groups: Vec<u32> = groups.into_iter().map(|gid| gid as u32).collect();
    if !groups.contains(&primary) {
        groups.push(primary);
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::fs::{self, File};
    use std::os::unix::fs::PermissionsExt;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("auto-sudo-{name}-{nonce}"))
    }

    fn cfg(body: &str) -> Config {
        serde_yaml::from_str(body).unwrap()
    }

    #[test]
    fn parses_flag_equals_and_flag_value() {
        let args = vec![
            "--file=/tmp/a".to_string(),
            "--other".to_string(),
            "--file".to_string(),
            "/tmp/b".to_string(),
        ];
        assert_eq!(
            extract_flag_values("--file", &args),
            vec!["/tmp/a".to_string(), "/tmp/b".to_string()]
        );
    }

    #[test]
    fn wildcard_path_filter_matches_file_rule() {
        let config = cfg(r#"
version: 1
commands:
  vim:
    rules:
      - name: protected-conf
        args:
          files:
            - position: any
        when:
          any_file:
            paths: ["/etc/*.conf"]
"#);
        let args = vec!["/etc/example.conf".to_string()];
        let request = DecisionRequest {
            command: "vim",
            args: &args,
            stdin_piped: false,
            stdout_piped: false,
        };
        assert_eq!(decide(&config, &request).unwrap().prefix, "sudo ");
    }

    #[test]
    fn decide_returns_empty_for_unknown_command() {
        let config = cfg("version: 1\ncommands: {}\n");
        let request = DecisionRequest {
            command: "vim",
            args: &[],
            stdin_piped: false,
            stdout_piped: false,
        };
        assert_eq!(decide(&config, &request).unwrap().prefix, "");
    }

    #[test]
    fn always_rule_returns_sudo_prefix() {
        let config = cfg(r#"
version: 1
commands:
  systemctl:
    rules:
      - name: always
        when:
          always: true
"#);
        let request = DecisionRequest {
            command: "systemctl",
            args: &["restart".into(), "nginx".into()],
            stdin_piped: false,
            stdout_piped: false,
        };
        assert_eq!(decide(&config, &request).unwrap().prefix, "sudo ");
    }

    #[test]
    fn command_level_always_sudo_returns_sudo_prefix() {
        let config = cfg(r#"
version: 1
commands:
  systemctl:
    always_sudo: true
"#);
        let request = DecisionRequest {
            command: "systemctl",
            args: &["restart".into(), "nginx".into()],
            stdin_piped: false,
            stdout_piped: false,
        };
        let decision = decide(&config, &request).unwrap();
        assert_eq!(decision.prefix, "sudo ");
        assert_eq!(decision.reason, "always_sudo enabled for systemctl");
    }

    #[test]
    fn command_level_sudo_overrides_default_for_always_sudo() {
        let config = cfg(r#"
version: 1
commands:
  psql:
    always_sudo: true
    sudo:
      mode: user
      user: postgres
"#);
        let request = DecisionRequest {
            command: "psql",
            args: &[],
            stdin_piped: false,
            stdout_piped: false,
        };
        assert_eq!(
            decide(&config, &request).unwrap().prefix,
            "sudo -u postgres "
        );
    }

    #[test]
    fn command_level_always_sudo_respects_pipe_policy() {
        let config = cfg(r#"
version: 1
defaults:
  allow_pipes: false
commands:
  systemctl:
    always_sudo: true
"#);
        let request = DecisionRequest {
            command: "systemctl",
            args: &[],
            stdin_piped: true,
            stdout_piped: false,
        };
        assert_eq!(decide(&config, &request).unwrap().prefix, "");
    }

    #[test]
    fn unwritable_existing_file_matches() {
        let path = temp_path("unwritable");
        File::create(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        let config = cfg(r#"
version: 1
commands:
  vim:
    rules:
      - name: edit-unwritable
        args:
          files:
            - position: any
              skip_prefixes: ["-", "+"]
        when:
          any_file:
            exists_not_writable: true
"#);
        let args = vec![path.to_string_lossy().to_string()];
        let request = DecisionRequest {
            command: "vim",
            args: &args,
            stdin_piped: false,
            stdout_piped: false,
        };
        assert_eq!(decide(&config, &request).unwrap().prefix, "sudo ");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn pipe_policy_blocks_sudo() {
        let config = cfg(r#"
version: 1
defaults:
  allow_pipes: false
commands:
  systemctl:
    rules:
      - name: always
        when:
          always: true
"#);
        let request = DecisionRequest {
            command: "systemctl",
            args: &[],
            stdin_piped: true,
            stdout_piped: false,
        };
        assert_eq!(decide(&config, &request).unwrap().prefix, "");
    }

    #[test]
    fn sudo_as_user_prefix() {
        let config = cfg(r#"
version: 1
commands:
  vim:
    rules:
      - name: user
        action:
          sudo:
            mode: user
            user: postgres
            group: postgres
        when:
          always: true
"#);
        let request = DecisionRequest {
            command: "vim",
            args: &[],
            stdin_piped: false,
            stdout_piped: false,
        };
        assert_eq!(
            decide(&config, &request).unwrap().prefix,
            "sudo -u postgres -g postgres "
        );
    }
}
