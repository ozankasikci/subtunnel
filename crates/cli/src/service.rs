use std::path::{Path, PathBuf};

#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::ffi::OsStr;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::fs;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::process::{Command, Output};

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
use anyhow::bail;
use anyhow::Result;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use anyhow::{ensure, Context};

#[cfg(any(target_os = "linux", target_os = "macos"))]
use crate::config::{load_config, resolve_config_path};

#[cfg(target_os = "linux")]
const SERVICE_NAME: &str = "subtunnel.service";
const LAUNCHD_LABEL: &str = "dev.subtunnel.agent";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceFormat {
    Systemd,
    Launchd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceAction {
    Install,
    Uninstall,
    Start,
    Stop,
    Status,
    Generate(ServiceFormat),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceScope {
    System,
    User,
}

pub fn generate_systemd_unit(executable: &Path, config_path: &Path, scope: ServiceScope) -> String {
    let wanted_by = match scope {
        ServiceScope::System => "multi-user.target",
        ServiceScope::User => "default.target",
    };
    let exec_start = [
        executable.to_string_lossy().as_ref(),
        "run",
        "--all",
        "--config",
        config_path.to_string_lossy().as_ref(),
    ]
    .into_iter()
    .map(systemd_quote)
    .collect::<Vec<_>>()
    .join(" ");

    format!(
        "[Unit]\n\
         Description=SubTunnel Agent\n\
         Wants=network-online.target\n\
         After=network-online.target\n\
         \n\
         [Service]\n\
         Type=simple\n\
         ExecStart={exec_start}\n\
         Restart=always\n\
         RestartSec=5\n\
         \n\
         [Install]\n\
         WantedBy={wanted_by}\n"
    )
}

pub fn generate_launchd_plist(
    executable: &Path,
    config_path: &Path,
    stdout_path: &Path,
    stderr_path: &Path,
) -> String {
    let arguments = [
        executable.to_string_lossy().as_ref(),
        "run",
        "--all",
        "--config",
        config_path.to_string_lossy().as_ref(),
    ]
    .into_iter()
    .map(|argument| format!("        <string>{}</string>", xml_escape(argument)))
    .collect::<Vec<_>>()
    .join("\n");

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
         \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\">\n\
         <dict>\n\
             <key>Label</key>\n\
             <string>{LAUNCHD_LABEL}</string>\n\
             <key>ProgramArguments</key>\n\
             <array>\n\
         {arguments}\n\
             </array>\n\
             <key>RunAtLoad</key>\n\
             <true/>\n\
             <key>KeepAlive</key>\n\
             <true/>\n\
             <key>StandardOutPath</key>\n\
             <string>{}</string>\n\
             <key>StandardErrorPath</key>\n\
             <string>{}</string>\n\
         </dict>\n\
         </plist>\n",
        xml_escape(&stdout_path.to_string_lossy()),
        xml_escape(&stderr_path.to_string_lossy()),
    )
}

#[cfg(target_os = "windows")]
pub fn handle_service(action: ServiceAction, config_override: Option<PathBuf>) -> Result<()> {
    let _ = (action, config_override);
    bail!("service management is not supported yet on Windows");
}

#[cfg(target_os = "linux")]
pub fn handle_service(action: ServiceAction, config_override: Option<PathBuf>) -> Result<()> {
    let scope = current_scope()?;
    match action {
        ServiceAction::Install => install_systemd(scope, config_override),
        ServiceAction::Uninstall => uninstall_systemd(scope),
        ServiceAction::Start => run_systemctl(scope, &["start", SERVICE_NAME]),
        ServiceAction::Stop => run_systemctl(scope, &["stop", SERVICE_NAME]),
        ServiceAction::Status => run_systemctl(scope, &["status", SERVICE_NAME]),
        ServiceAction::Generate(format) => generate_to_stdout(format, scope, config_override),
    }
}

#[cfg(target_os = "macos")]
pub fn handle_service(action: ServiceAction, config_override: Option<PathBuf>) -> Result<()> {
    let scope = current_scope()?;
    match action {
        ServiceAction::Install => install_launchd(scope, config_override),
        ServiceAction::Uninstall => uninstall_launchd(scope),
        ServiceAction::Start => start_launchd(scope),
        ServiceAction::Stop => stop_launchd(scope),
        ServiceAction::Status => status_launchd(scope),
        ServiceAction::Generate(format) => generate_to_stdout(format, scope, config_override),
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn handle_service(action: ServiceAction, config_override: Option<PathBuf>) -> Result<()> {
    let _ = (action, config_override);
    bail!("service management is supported only on Linux and macOS");
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn generate_to_stdout(
    format: ServiceFormat,
    scope: ServiceScope,
    config_override: Option<PathBuf>,
) -> Result<()> {
    let executable = std::env::current_exe()
        .context("failed to determine the current SubTunnel executable path")?;
    let config_path = resolve_config_path(config_override)?;
    let contents = match format {
        ServiceFormat::Systemd => generate_systemd_unit(&executable, &config_path, scope),
        ServiceFormat::Launchd => {
            let (_, stdout_path, stderr_path) = launchd_paths(scope)?;
            generate_launchd_plist(&executable, &config_path, &stdout_path, &stderr_path)
        }
    };
    print!("{contents}");
    Ok(())
}

#[cfg(target_os = "linux")]
fn install_systemd(scope: ServiceScope, config_override: Option<PathBuf>) -> Result<()> {
    let config_path = validate_config(config_override)?;
    let executable = std::env::current_exe()
        .context("failed to determine the current SubTunnel executable path")?;
    let unit_path = systemd_unit_path(scope)?;
    let contents = generate_systemd_unit(&executable, &config_path, scope);
    write_service_file(&unit_path, &contents)?;
    run_systemctl(scope, &["daemon-reload"])?;
    run_systemctl(scope, &["enable", "--now", SERVICE_NAME])?;
    if scope == ServiceScope::User {
        eprintln!(
            "Note: systemd user services start at login. Run `loginctl enable-linger $USER` to make this service start at boot."
        );
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn uninstall_systemd(scope: ServiceScope) -> Result<()> {
    let unit_path = systemd_unit_path(scope)?;
    run_systemctl(scope, &["disable", "--now", SERVICE_NAME])?;
    fs::remove_file(&unit_path)
        .with_context(|| format!("failed to remove {}", unit_path.display()))?;
    eprintln!("Removed service file: {}", unit_path.display());
    run_systemctl(scope, &["daemon-reload"])?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn systemd_unit_path(scope: ServiceScope) -> Result<PathBuf> {
    match scope {
        ServiceScope::System => Ok(PathBuf::from("/etc/systemd/system").join(SERVICE_NAME)),
        ServiceScope::User => Ok(user_home()?.join(".config/systemd/user").join(SERVICE_NAME)),
    }
}

#[cfg(target_os = "linux")]
fn run_systemctl(scope: ServiceScope, arguments: &[&str]) -> Result<()> {
    let mut command = Command::new("systemctl");
    if scope == ServiceScope::User {
        command.arg("--user");
    }
    command.args(arguments);
    run_checked(command)
}

#[cfg(target_os = "macos")]
fn install_launchd(scope: ServiceScope, config_override: Option<PathBuf>) -> Result<()> {
    let config_path = validate_config(config_override)?;
    let executable = std::env::current_exe()
        .context("failed to determine the current SubTunnel executable path")?;
    let (plist_path, stdout_path, stderr_path) = launchd_paths(scope)?;
    let log_dir = stdout_path
        .parent()
        .context("launchd log path has no parent directory")?;
    fs::create_dir_all(log_dir)
        .with_context(|| format!("failed to create log directory {}", log_dir.display()))?;
    let contents = generate_launchd_plist(&executable, &config_path, &stdout_path, &stderr_path);
    write_service_file(&plist_path, &contents)?;
    bootstrap_launchd(scope, &plist_path)
}

#[cfg(target_os = "macos")]
fn uninstall_launchd(scope: ServiceScope) -> Result<()> {
    let (plist_path, _, _) = launchd_paths(scope)?;
    if let Err(error) = bootout_launchd(scope, &plist_path) {
        eprintln!("Could not unload the service before removal: {error:#}");
    }
    fs::remove_file(&plist_path)
        .with_context(|| format!("failed to remove {}", plist_path.display()))?;
    eprintln!("Removed service file: {}", plist_path.display());
    Ok(())
}

#[cfg(target_os = "macos")]
fn start_launchd(scope: ServiceScope) -> Result<()> {
    let (plist_path, _, _) = launchd_paths(scope)?;
    bootstrap_launchd(scope, &plist_path)
}

#[cfg(target_os = "macos")]
fn stop_launchd(scope: ServiceScope) -> Result<()> {
    let (plist_path, _, _) = launchd_paths(scope)?;
    bootout_launchd(scope, &plist_path)
}

#[cfg(target_os = "macos")]
fn status_launchd(scope: ServiceScope) -> Result<()> {
    let target = format!("{}/{}", launchd_domain(scope)?, LAUNCHD_LABEL);
    let mut command = Command::new("launchctl");
    command.args(["print", &target]);
    run_checked(command)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn launchd_paths(scope: ServiceScope) -> Result<(PathBuf, PathBuf, PathBuf)> {
    match scope {
        ServiceScope::System => {
            let log_dir = PathBuf::from("/Library/Logs/subtunnel");
            Ok((
                PathBuf::from("/Library/LaunchDaemons").join(format!("{LAUNCHD_LABEL}.plist")),
                log_dir.join("subtunnel.out.log"),
                log_dir.join("subtunnel.err.log"),
            ))
        }
        ServiceScope::User => {
            let home = user_home()?;
            let log_dir = home.join("Library/Logs/subtunnel");
            Ok((
                home.join("Library/LaunchAgents")
                    .join(format!("{LAUNCHD_LABEL}.plist")),
                log_dir.join("subtunnel.out.log"),
                log_dir.join("subtunnel.err.log"),
            ))
        }
    }
}

#[cfg(target_os = "macos")]
fn bootstrap_launchd(scope: ServiceScope, plist_path: &Path) -> Result<()> {
    let domain = launchd_domain(scope)?;
    let mut primary = Command::new("launchctl");
    primary.arg("bootstrap").arg(domain).arg(plist_path);
    let mut fallback = Command::new("launchctl");
    fallback.arg("load").arg(plist_path);
    run_with_fallback(primary, fallback)
}

#[cfg(target_os = "macos")]
fn bootout_launchd(scope: ServiceScope, plist_path: &Path) -> Result<()> {
    let domain = launchd_domain(scope)?;
    let mut primary = Command::new("launchctl");
    primary.arg("bootout").arg(domain).arg(plist_path);
    let mut fallback = Command::new("launchctl");
    fallback.arg("unload").arg(plist_path);
    run_with_fallback(primary, fallback)
}

#[cfg(target_os = "macos")]
fn launchd_domain(scope: ServiceScope) -> Result<String> {
    match scope {
        ServiceScope::System => Ok("system".to_string()),
        ServiceScope::User => Ok(format!("gui/{}", current_uid()?)),
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn validate_config(config_override: Option<PathBuf>) -> Result<PathBuf> {
    let config_path = resolve_config_path(config_override)?;
    let config = load_config(&config_path)?;
    ensure!(
        !config.tunnels.is_empty(),
        "config file {} must define at least one tunnel under [tunnels.<name>]",
        config_path.display()
    );
    fs::canonicalize(&config_path)
        .with_context(|| format!("failed to resolve config path {}", config_path.display()))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn current_scope() -> Result<ServiceScope> {
    if current_uid()? == 0 {
        Ok(ServiceScope::System)
    } else {
        Ok(ServiceScope::User)
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn current_uid() -> Result<u32> {
    let output = Command::new("id")
        .arg("-u")
        .output()
        .context("failed to run `id -u`")?;
    ensure!(
        output.status.success(),
        "`id -u` failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    String::from_utf8(output.stdout)
        .context("`id -u` returned non-UTF-8 output")?
        .trim()
        .parse()
        .context("`id -u` returned an invalid user ID")
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn user_home() -> Result<PathBuf> {
    dirs::home_dir().context("could not determine the current user's home directory")
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn write_service_file(path: &Path, contents: &str) -> Result<()> {
    let parent = path
        .parent()
        .context("service file path has no parent directory")?;
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create service directory {}", parent.display()))?;
    fs::write(path, contents)
        .with_context(|| format!("failed to write service file {}", path.display()))?;
    eprintln!("Wrote service file: {}", path.display());
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn run_checked(mut command: Command) -> Result<()> {
    let display = display_command(&command);
    let output = command
        .output()
        .with_context(|| format!("failed to run `{display}`"))?;
    if !output.status.success() {
        return Err(command_failure(&display, &output));
    }
    print_command_output(&output);
    eprintln!("Ran: {display}");
    Ok(())
}

#[cfg(target_os = "macos")]
fn run_with_fallback(mut primary: Command, fallback: Command) -> Result<()> {
    let display = display_command(&primary);
    let output = primary
        .output()
        .with_context(|| format!("failed to run `{display}`"))?;
    if output.status.success() {
        print_command_output(&output);
        eprintln!("Ran: {display}");
        return Ok(());
    }

    let primary_error = command_failure(&display, &output);
    eprintln!("Primary command failed, trying compatibility fallback: {primary_error:#}");
    run_checked(fallback).context("launchctl compatibility fallback also failed")
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn command_failure(display: &str, output: &Output) -> anyhow::Error {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let details = if stderr.trim().is_empty() {
        stdout.trim()
    } else {
        stderr.trim()
    };
    anyhow::anyhow!(
        "command `{display}` failed with status {}: {details}",
        output.status
    )
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn print_command_output(output: &Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        print!("{stdout}");
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        eprint!("{stderr}");
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn display_command(command: &Command) -> String {
    std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(display_argument)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn display_argument(argument: &OsStr) -> String {
    let argument = argument.to_string_lossy();
    if argument.contains(char::is_whitespace) {
        format!("\"{}\"", argument.replace('"', "\\\""))
    } else {
        argument.into_owned()
    }
}

fn systemd_quote(argument: &str) -> String {
    format!(
        "\"{}\"",
        argument.replace('\\', "\\\\").replace('"', "\\\"")
    )
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn systemd_unit_contains_agent_command_and_restart_policy() {
        let unit = generate_systemd_unit(
            Path::new("/usr/local/bin/subtunnel"),
            Path::new("/home/test/.config/subtunnel/config.toml"),
            ServiceScope::User,
        );

        assert!(unit.contains("Wants=network-online.target"));
        assert!(unit.contains("After=network-online.target"));
        assert!(unit.contains(
            "ExecStart=\"/usr/local/bin/subtunnel\" \"run\" \"--all\" \"--config\" \
             \"/home/test/.config/subtunnel/config.toml\""
        ));
        assert!(unit.contains("Restart=always"));
        assert!(unit.contains("RestartSec=5"));
        assert!(unit.contains("WantedBy=default.target"));
    }

    #[test]
    fn launchd_plist_contains_agent_arguments_and_keep_alive() {
        let plist = generate_launchd_plist(
            Path::new("/usr/local/bin/subtunnel"),
            Path::new("/Users/test/.config/subtunnel/config.toml"),
            Path::new("/Users/test/Library/Logs/subtunnel/out.log"),
            Path::new("/Users/test/Library/Logs/subtunnel/err.log"),
        );

        assert!(plist.contains("<key>ProgramArguments</key>"));
        assert!(plist.contains("<string>/usr/local/bin/subtunnel</string>"));
        assert!(plist.contains("<string>run</string>"));
        assert!(plist.contains("<string>--all</string>"));
        assert!(plist.contains("<string>--config</string>"));
        assert!(plist.contains("<string>/Users/test/.config/subtunnel/config.toml</string>"));
        assert!(plist.contains("<key>RunAtLoad</key>"));
        assert!(plist.contains("<key>KeepAlive</key>"));
        assert_eq!(plist.matches("<true/>").count(), 2);
    }

    #[test]
    fn templates_escape_paths() {
        let unit = generate_systemd_unit(
            Path::new("/opt/Sub Tunnel/subtunnel"),
            Path::new("/tmp/config \"agent\".toml"),
            ServiceScope::System,
        );
        assert!(unit.contains(r#"ExecStart="/opt/Sub Tunnel/subtunnel""#));
        assert!(unit.contains(r#""/tmp/config \"agent\".toml""#));

        let plist = generate_launchd_plist(
            Path::new("/opt/Sub&Tunnel/subtunnel"),
            Path::new("/tmp/config.toml"),
            Path::new("/tmp/out.log"),
            Path::new("/tmp/err.log"),
        );
        assert!(plist.contains("/opt/Sub&amp;Tunnel/subtunnel"));
    }
}
