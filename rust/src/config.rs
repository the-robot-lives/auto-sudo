use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[allow(dead_code)]
    pub version: Option<u32>,
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default)]
    pub commands: BTreeMap<String, CommandConfig>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Defaults {
    pub allow_pipes: Option<bool>,
    pub sudo: Option<SudoSpec>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CommandConfig {
    pub wrap: Option<bool>,
    pub allow_pipes: Option<bool>,
    #[serde(default)]
    pub always_sudo: bool,
    pub sudo: Option<SudoSpec>,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    pub name: Option<String>,
    pub action: Option<Action>,
    #[serde(default)]
    pub args: ArgSpec,
    pub when: When,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Action {
    pub sudo: SudoSpec,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SudoMode {
    Root,
    User,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct SudoSpec {
    #[serde(default = "default_sudo_mode")]
    pub mode: SudoMode,
    pub user: Option<String>,
    pub group: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ArgSpec {
    #[serde(default)]
    pub files: Vec<FileArgSpec>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileArgSpec {
    pub position: Option<PositionSpec>,
    pub flag: Option<String>,
    #[serde(default)]
    pub skip_prefixes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum PositionSpec {
    Any(String),
    Index(usize),
}

#[derive(Debug, Clone, Deserialize)]
pub struct When {
    pub always: Option<bool>,
    pub any_file: Option<FileChecks>,
    pub all_files: Option<FileChecks>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct FileChecks {
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub path_prefixes: Vec<String>,
    #[serde(default)]
    pub path_suffixes: Vec<String>,
    #[serde(default)]
    pub exists: bool,
    #[serde(default)]
    pub missing: bool,
    #[serde(default)]
    pub exists_not_writable: bool,
    #[serde(default)]
    pub missing_parent_not_writable: bool,
    #[serde(default)]
    pub missing_parent_not_readable: bool,
    #[serde(default)]
    pub current_user_can_read: bool,
    #[serde(default)]
    pub current_user_can_write: bool,
    #[serde(default)]
    pub current_user_can_execute: bool,
    #[serde(default)]
    pub current_user_cannot_read: bool,
    #[serde(default)]
    pub current_user_cannot_write: bool,
    #[serde(default)]
    pub current_user_cannot_execute: bool,
    #[serde(default)]
    pub owner_is_current_user: bool,
    #[serde(default)]
    pub owner_is_not_current_user: bool,
    #[serde(default)]
    pub group_in_current_user_groups: bool,
    #[serde(default)]
    pub group_not_in_current_user_groups: bool,
}

fn default_sudo_mode() -> SudoMode {
    SudoMode::Root
}

impl Default for SudoSpec {
    fn default() -> Self {
        Self {
            mode: SudoMode::Root,
            user: None,
            group: None,
        }
    }
}

impl Config {
    pub fn default_path() -> Result<PathBuf, String> {
        dirs::home_dir()
            .map(|home| home.join(".config/auto-sudo/config.yaml"))
            .ok_or_else(|| "could not resolve home directory".to_string())
    }

    pub fn load(path: Option<&Path>) -> Result<Self, String> {
        let path = match path {
            Some(path) => path.to_path_buf(),
            None => Self::default_path()?,
        };
        let body = fs::read_to_string(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
        serde_yaml::from_str(&body)
            .map_err(|err| format!("failed to parse {}: {err}", path.display()))
    }

    pub fn sudo_for_command(&self, command_config: &CommandConfig) -> SudoSpec {
        command_config
            .sudo
            .clone()
            .or_else(|| self.defaults.sudo.clone())
            .unwrap_or_default()
    }

    pub fn sudo_for_rule(&self, command_config: &CommandConfig, rule: &Rule) -> SudoSpec {
        rule.action
            .as_ref()
            .map(|action| action.sudo.clone())
            .or_else(|| command_config.sudo.clone())
            .or_else(|| self.defaults.sudo.clone())
            .unwrap_or_default()
    }
}

impl CommandConfig {
    pub fn should_wrap(&self) -> bool {
        self.wrap.unwrap_or(true)
    }

    pub fn always_sudo(&self) -> bool {
        self.always_sudo
    }

    pub fn allow_pipes(&self, defaults: &Defaults) -> bool {
        self.allow_pipes.or(defaults.allow_pipes).unwrap_or(false)
    }
}

impl PositionSpec {
    pub fn is_any(&self) -> bool {
        matches!(self, PositionSpec::Any(value) if value == "any")
    }
}
