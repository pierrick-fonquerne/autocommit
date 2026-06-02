//! Configuration loading for autocommit.
//!
//! Assembles the effective [`Config`] from built-in defaults, a global
//! `autocommit.toml` and an optional repository-local one, merged with a
//! layered deep-merge strategy. A missing file falls back to defaults; a
//! present-but-invalid file is a hard error.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use figment::Figment;
use figment::providers::{Format, Serialized, Toml};
use serde::{Deserialize, Serialize};

/// Effective configuration, assembled from defaults and up to two TOML files.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Local model endpoint settings.
    pub llm: LlmConfig,
    /// Generated-message settings.
    pub message: MessageConfig,
    /// Scope-derivation settings.
    pub scope: ScopeConfig,
    /// Diff-reduction settings.
    pub diff: DiffConfig,
    /// Runtime behaviour settings.
    pub behavior: BehaviorConfig,
}

/// Local model (Ollama-compatible) connection settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LlmConfig {
    /// Base URL of the model endpoint.
    pub endpoint: String,
    /// Model name to request (falls back to a secondary model if unavailable).
    pub model: String,
    /// Request timeout, in milliseconds.
    pub timeout_ms: u64,
}

/// Settings controlling the generated commit message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MessageConfig {
    /// Natural language the description is written in.
    pub language: Language,
    /// Maximum length of the commit header.
    pub max_header_len: usize,
    /// Allowed Conventional Commits types.
    pub types: Vec<String>,
}

/// Natural language used for the generated description.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// French.
    Fr,
    /// English.
    En,
}

/// Settings controlling scope derivation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScopeConfig {
    /// Whether to derive the scope automatically from the dominant path.
    pub auto: bool,
    /// Explicit glob-to-scope mappings (TOML table `[scope.mappings]`).
    pub mappings: BTreeMap<String, String>,
}

/// Settings controlling diff reduction before model inference.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DiffConfig {
    /// Line threshold above which the diff is condensed.
    pub max_lines: usize,
    /// Glob patterns excluded from the diff.
    pub exclude: Vec<String>,
}

/// Settings controlling runtime behaviour.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BehaviorConfig {
    /// Never block a commit; degrade gracefully on failure.
    pub fail_open: bool,
    /// Ask for confirmation in interactive CLI mode.
    pub confirm_in_cli: bool,
}

/// Errors that can occur while loading configuration.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// A configuration file existed but could not be read or parsed.
    #[error("invalid configuration: {0}")]
    Invalid(String),
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:11434".to_owned(),
            model: "codestral-mamba".to_owned(),
            timeout_ms: 8000,
        }
    }
}

impl Default for MessageConfig {
    fn default() -> Self {
        Self {
            language: Language::default(),
            max_header_len: 72,
            types: [
                "feat", "fix", "refactor", "perf", "docs", "test", "chore", "ci", "build",
            ]
            .iter()
            .map(|s| (*s).to_owned())
            .collect(),
        }
    }
}

impl Default for Language {
    fn default() -> Self {
        Self::En
    }
}

impl Default for ScopeConfig {
    fn default() -> Self {
        Self {
            auto: true,
            mappings: BTreeMap::new(),
        }
    }
}

impl Default for DiffConfig {
    fn default() -> Self {
        Self {
            max_lines: 400,
            exclude: ["*.lock", "*.png", "dist/**"]
                .iter()
                .map(|s| (*s).to_owned())
                .collect(),
        }
    }
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            fail_open: true,
            confirm_in_cli: true,
        }
    }
}

/// Loads the effective configuration.
///
/// Merges, in order: built-in defaults, the global config file and an optional
/// repository-local `autocommit.toml` resolved from `repo_root`. Missing files
/// are ignored; a present-but-invalid file yields [`ConfigError`].
pub fn load(repo_root: Option<&Path>) -> Result<Config, ConfigError> {
    let global = read_if_present(global_config_path().as_deref())?;
    let local = read_if_present(repo_root.map(local_config_path).as_deref())?;
    from_layers(global.as_deref(), local.as_deref())
}

/// Reads a file to a string when the path exists, mapping IO failures to
/// [`ConfigError::Invalid`]. A `None` path or a missing file yields `None`.
fn read_if_present(path: Option<&Path>) -> Result<Option<String>, ConfigError> {
    match path {
        Some(p) if p.exists() => std::fs::read_to_string(p)
            .map(Some)
            .map_err(|e| ConfigError::Invalid(format!("cannot read {}: {e}", p.display()))),
        _ => Ok(None),
    }
}

/// Merges the optional global and local TOML layers over the defaults.
///
/// This is the testable core of [`load`]: it takes the file *contents* rather
/// than paths, so the cascade can be exercised without touching the filesystem.
fn from_layers(global: Option<&str>, local: Option<&str>) -> Result<Config, ConfigError> {
    let mut figment = Figment::from(Serialized::defaults(Config::default()));
    if let Some(global) = global {
        figment = figment.merge(Toml::string(global));
    }
    if let Some(local) = local {
        figment = figment.merge(Toml::string(local));
    }
    figment
        .extract()
        .map_err(|e| ConfigError::Invalid(e.to_string()))
}

/// Returns the platform-specific path of the global config file, if resolvable.
fn global_config_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "autocommit")
        .map(|dirs| dirs.config_dir().join("autocommit.toml"))
}

/// Returns the repository-local config path for the given repo root.
fn local_config_path(repo_root: &Path) -> PathBuf {
    repo_root.join("autocommit.toml")
}

#[cfg(test)]
mod tests {
    use super::{Language, from_layers};

    #[test]
    fn defaults_are_complete() {
        let config = from_layers(None, None).expect("defaults must load");
        assert_eq!(config.llm.model, "codestral-mamba");
        assert_eq!(config.llm.timeout_ms, 8000);
        assert_eq!(config.message.max_header_len, 72);
        assert_eq!(config.message.language, Language::En);
        assert!(config.scope.auto);
        assert!(config.behavior.fail_open);
    }

    #[test]
    fn local_overrides_global() {
        let global = "[llm]\nmodel = \"mistral\"\n";
        let local = "[llm]\nmodel = \"qwen\"\n";
        let config = from_layers(Some(global), Some(local)).expect("layers must merge");
        assert_eq!(config.llm.model, "qwen");
    }

    #[test]
    fn partial_section_keeps_other_defaults() {
        let local = "[llm]\nmodel = \"qwen\"\n";
        let config = from_layers(None, Some(local)).expect("partial section must merge");
        assert_eq!(config.llm.model, "qwen");
        assert_eq!(config.llm.endpoint, "http://localhost:11434");
        assert_eq!(config.llm.timeout_ms, 8000);
    }

    #[test]
    fn language_parses_lowercase() {
        let local = "[message]\nlanguage = \"fr\"\n";
        let config = from_layers(None, Some(local)).expect("language must parse");
        assert_eq!(config.message.language, Language::Fr);
    }

    #[test]
    fn scope_mappings_are_loaded() {
        let local = "[scope.mappings]\n\"src/auth/**\" = \"auth\"\n";
        let config = from_layers(None, Some(local)).expect("mappings must load");
        assert_eq!(
            config.scope.mappings.get("src/auth/**").map(String::as_str),
            Some("auth")
        );
    }

    #[test]
    fn invalid_toml_is_an_error() {
        let bad = "this is = not = valid";
        assert!(from_layers(None, Some(bad)).is_err());
    }

    #[test]
    fn unknown_language_is_an_error() {
        let local = "[message]\nlanguage = \"de\"\n";
        assert!(from_layers(None, Some(local)).is_err());
    }
}
