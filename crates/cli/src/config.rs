use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub server: String,
    pub token: String,
    #[serde(default = "default_tls_verify")]
    pub tls_verify: bool,
    pub tls_ca: Option<String>,
    #[serde(default)]
    pub tunnels: BTreeMap<String, TunnelConfig>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TunnelConfig {
    pub local_port: u16,
    pub subdomain: Option<String>,
}

fn default_tls_verify() -> bool {
    true
}

pub fn default_config_path() -> Result<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        if let Some(home_dir) = dirs::home_dir() {
            let xdg_style_path = home_dir.join(".config/subtunnel/config.toml");
            if xdg_style_path.exists() {
                return Ok(xdg_style_path);
            }
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    if let Some(xdg_config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        if !xdg_config_home.is_empty() {
            return Ok(PathBuf::from(xdg_config_home).join("subtunnel/config.toml"));
        }
    }

    let config_dir =
        dirs::config_dir().context("could not determine the platform configuration directory")?;
    Ok(config_dir.join("subtunnel/config.toml"))
}

pub fn resolve_config_path(override_path: Option<PathBuf>) -> Result<PathBuf> {
    let path = match override_path {
        Some(path) => path,
        None => default_config_path()?,
    };

    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(std::env::current_dir()
            .context("could not determine the current directory")?
            .join(path))
    }
}

pub fn load_config(path: &Path) -> Result<Config> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read config file {}", path.display()))?;
    parse_config(&contents, path)
}

pub fn parse_config(contents: &str, path: &Path) -> Result<Config> {
    toml::from_str(contents)
        .with_context(|| format!("failed to parse config file {}", path.display()))
}

pub fn select_tunnels(
    config: &Config,
    all: bool,
    requested_names: &[String],
) -> Result<Vec<(String, TunnelConfig)>> {
    if all || requested_names.is_empty() {
        return Ok(config
            .tunnels
            .iter()
            .map(|(name, tunnel)| (name.clone(), tunnel.clone()))
            .collect());
    }

    let mut selected = Vec::with_capacity(requested_names.len());
    let mut seen = BTreeSet::new();
    for name in requested_names {
        if !seen.insert(name) {
            continue;
        }

        let Some(tunnel) = config.tunnels.get(name) else {
            let available = config
                .tunnels
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(", ");
            if available.is_empty() {
                bail!("unknown tunnel '{name}'; the config file defines no tunnels");
            }
            bail!("unknown tunnel '{name}'; available tunnels: {available}");
        };
        selected.push((name.clone(), tunnel.clone()));
    }

    Ok(selected)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG_PATH: &str = "/tmp/subtunnel-config.toml";

    #[test]
    fn parses_valid_config_with_tls_defaults() {
        let config = parse_config(
            r#"
server = "tunnel.example.com:7835"
token = "secret"

[tunnels.myapp]
local_port = 3000
"#,
            Path::new(CONFIG_PATH),
        )
        .unwrap();

        assert_eq!(config.server, "tunnel.example.com:7835");
        assert_eq!(config.token, "secret");
        assert!(config.tls_verify);
        assert!(config.tls_ca.is_none());
        assert_eq!(config.tunnels["myapp"].local_port, 3000);
    }

    #[test]
    fn rejects_unknown_fields() {
        let error = parse_config(
            r#"
server = "tunnel.example.com:7835"
token = "secret"
unexpected = true

[tunnels.myapp]
local_port = 3000
"#,
            Path::new(CONFIG_PATH),
        )
        .unwrap_err();

        let message = format!("{error:#}");
        assert!(message.contains(CONFIG_PATH));
        assert!(message.contains("unknown field `unexpected`"));
    }

    #[test]
    fn missing_token_error_names_field_and_path() {
        let error = parse_config(
            r#"
server = "tunnel.example.com:7835"

[tunnels.myapp]
local_port = 3000
"#,
            Path::new(CONFIG_PATH),
        )
        .unwrap_err();

        let message = format!("{error:#}");
        assert!(message.contains(CONFIG_PATH));
        assert!(message.contains("missing field `token`"));
    }

    #[test]
    fn parses_multiple_tunnels() {
        let config = parse_config(
            r#"
server = "tunnel.example.com:7835"
token = "secret"
tls_verify = false
tls_ca = "/tmp/ca.pem"

[tunnels.api]
local_port = 3000
subdomain = "api"

[tunnels.web]
local_port = 8080
"#,
            Path::new(CONFIG_PATH),
        )
        .unwrap();

        assert_eq!(config.tunnels.len(), 2);
        assert_eq!(config.tunnels["api"].subdomain.as_deref(), Some("api"));
        assert_eq!(config.tunnels["web"].local_port, 8080);
        assert!(!config.tls_verify);
        assert_eq!(config.tls_ca.as_deref(), Some("/tmp/ca.pem"));
    }

    #[test]
    fn config_without_tunnels_selects_an_empty_list() {
        let config = parse_config(
            r#"
server = "tunnel.example.com:7835"
token = "secret"
"#,
            Path::new(CONFIG_PATH),
        )
        .unwrap();

        assert!(config.tunnels.is_empty());
        assert!(select_tunnels(&config, false, &[]).unwrap().is_empty());
    }

    #[test]
    fn tunnel_selection_supports_all_names_and_unknown_errors() {
        let config = parse_config(
            r#"
server = "tunnel.example.com:7835"
token = "secret"

[tunnels.api]
local_port = 3000

[tunnels.web]
local_port = 8080
"#,
            Path::new(CONFIG_PATH),
        )
        .unwrap();

        let all = select_tunnels(&config, true, &[]).unwrap();
        assert_eq!(
            all.iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>(),
            ["api", "web"]
        );

        let default_all = select_tunnels(&config, false, &[]).unwrap();
        assert_eq!(default_all, all);

        let names = vec!["web".to_string()];
        let selected = select_tunnels(&config, false, &names).unwrap();
        assert_eq!(
            selected,
            vec![("web".to_string(), config.tunnels["web"].clone())]
        );

        let error = select_tunnels(&config, false, &["missing".to_string()]).unwrap_err();
        assert!(error.to_string().contains("unknown tunnel 'missing'"));
        assert!(error.to_string().contains("api, web"));
    }
}
