use std::path::{Path, PathBuf};

use anyhow::{ensure, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use tracing::info;
use tracing_subscriber::EnvFilter;

use subtunnel::client::Client;
use subtunnel::config::{load_config, resolve_config_path, select_tunnels, TunnelConfig};
use subtunnel::runner::run_tunnels;
use subtunnel::server::{Server, ServerConfig};
use subtunnel::service::{handle_service, ServiceAction, ServiceFormat};

#[derive(Parser)]
#[command(
    name = "subtunnel",
    version,
    about = "Expose local services to the internet"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the SubTunnel server (public-facing VPS).
    Server {
        /// Port to listen on for agent connections (control plane).
        #[arg(long, default_value_t = 7835)]
        port: u16,

        /// Port for HTTP listener (receives proxied traffic from nginx).
        #[arg(long, default_value_t = 8080)]
        http_port: u16,

        /// Host to bind to / advertise.
        #[arg(long, default_value = "0.0.0.0")]
        host: String,

        /// Domain for tunnel subdomains (e.g. tunnel.example.com).
        #[arg(long)]
        domain: String,

        /// Additional domains to accept (can be repeated).
        #[arg(long = "extra-domain")]
        extra_domains: Vec<String>,

        /// Authentication token that agents must provide.
        #[arg(long, env = "SUBTUNNEL_TOKEN")]
        token: Option<String>,

        /// Path to TLS certificate PEM file (e.g. Let's Encrypt fullchain.pem).
        #[arg(long)]
        tls_cert: Option<String>,

        /// Path to TLS private key PEM file.
        #[arg(long)]
        tls_key: Option<String>,
    },

    /// Connect to a SubTunnel server and expose a local port.
    Local {
        /// Local port to expose (e.g. 8080).
        local_port: u16,

        /// Server address to connect to (host:port).
        #[arg(long)]
        to: String,

        /// Authentication token.
        #[arg(long, env = "SUBTUNNEL_TOKEN")]
        token: String,

        /// Request a specific subdomain (e.g. "myapp" for myapp.tunnel.example.com).
        #[arg(long)]
        subdomain: Option<String>,

        /// Verify server TLS certificate (default: true). Set to false for self-signed certs.
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        tls_verify: bool,

        /// Path to a custom CA certificate PEM file for server verification.
        #[arg(long)]
        tls_ca: Option<String>,
    },

    /// Run one or more tunnels from a config file.
    Run {
        /// Start every tunnel in the config file.
        #[arg(long, conflicts_with = "tunnel_names")]
        all: bool,

        /// Tunnel names to start. Starts all tunnels when omitted.
        #[arg(value_name = "TUNNEL")]
        tunnel_names: Vec<String>,

        /// Path to the SubTunnel config file.
        #[arg(long, value_name = "PATH")]
        config: Option<PathBuf>,
    },

    /// Manage the native SubTunnel agent service.
    Service(ServiceArgs),
}

#[derive(Args)]
struct ServiceArgs {
    /// Path to the SubTunnel config file.
    #[arg(long, global = true, value_name = "PATH")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: ServiceCommand,
}

#[derive(Subcommand)]
enum ServiceCommand {
    /// Install, enable, and start the native service.
    Install,
    /// Stop, disable, and remove the native service.
    Uninstall,
    /// Start the installed native service.
    Start,
    /// Stop the installed native service.
    Stop,
    /// Show the native service status.
    Status,
    /// Print a service definition without installing it.
    Generate {
        /// Service definition format.
        #[arg(value_enum)]
        format: ServiceFormatArg,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum ServiceFormatArg {
    Systemd,
    Launchd,
}

impl From<ServiceFormatArg> for ServiceFormat {
    fn from(value: ServiceFormatArg) -> Self {
        match value {
            ServiceFormatArg::Systemd => Self::Systemd,
            ServiceFormatArg::Launchd => Self::Launchd,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for Ctrl+C");
        info!("Ctrl+C received, shutting down...");
        let _ = shutdown_tx.send(true);
    });

    match cli.command {
        Command::Server {
            port,
            http_port,
            host,
            domain,
            extra_domains,
            token,
            tls_cert,
            tls_key,
        } => {
            eprintln!(
                "\n  \x1b[1;32msubtunnel\x1b[0m v{}\n  \x1b[1mMode:\x1b[0m       server\n  \x1b[1mControl:\x1b[0m    {}:{}\n  \x1b[1mHTTP:\x1b[0m       {}:{}\n  \x1b[1mDomain:\x1b[0m     *.{}\n  \x1b[1mAuth:\x1b[0m       {}\n",
                env!("CARGO_PKG_VERSION"),
                host, port,
                host, http_port,
                domain,
                if token.is_some() { "token required" } else { "disabled" },
            );

            let config = ServerConfig {
                control_port: port,
                http_port,
                auth_token: token,
                host,
                domain,
                extra_domains,
                tls_cert,
                tls_key,
            };
            let server = Server::new(config);
            server.run().await?;
        }
        Command::Local {
            local_port,
            to,
            token,
            subdomain,
            tls_verify,
            tls_ca,
        } => {
            eprintln!(
                "\n  \x1b[1;32msubtunnel\x1b[0m v{}\n  \x1b[1mMode:\x1b[0m       client\n  \x1b[1mLocal:\x1b[0m      localhost:{}\n  \x1b[1mServer:\x1b[0m     {}\n  \x1b[1mConnecting...\x1b[0m\n",
                env!("CARGO_PKG_VERSION"),
                local_port,
                to,
            );

            use subtunnel::client::ConnectTlsOptions;
            let tls_opts = ConnectTlsOptions {
                verify: tls_verify,
                ca_path: tls_ca,
            };
            let client = Client::new(to, token, local_port, subdomain, tls_opts);
            client.run(shutdown_rx).await?;
        }
        Command::Run {
            all,
            tunnel_names,
            config,
        } => {
            let config_path = resolve_config_path(config)?;
            let config = load_config(&config_path)?;
            let tunnels = select_tunnels(&config, all, &tunnel_names)?;
            ensure!(
                !tunnels.is_empty(),
                "config file {} defines no tunnels",
                config_path.display()
            );
            print_run_banner(&config_path, &tunnels);
            run_tunnels(&config, tunnels, shutdown_rx).await?;
        }
        Command::Service(args) => {
            let action = match args.command {
                ServiceCommand::Install => ServiceAction::Install,
                ServiceCommand::Uninstall => ServiceAction::Uninstall,
                ServiceCommand::Start => ServiceAction::Start,
                ServiceCommand::Stop => ServiceAction::Stop,
                ServiceCommand::Status => ServiceAction::Status,
                ServiceCommand::Generate { format } => ServiceAction::Generate(format.into()),
            };
            handle_service(action, args.config)?;
        }
    }

    Ok(())
}

fn print_run_banner(config_path: &Path, tunnels: &[(String, TunnelConfig)]) {
    eprintln!(
        "\n  \x1b[1;32msubtunnel\x1b[0m v{}\n  \x1b[1mMode:\x1b[0m       agent\n  \x1b[1mConfig:\x1b[0m     {}\n  \x1b[1mTunnels:\x1b[0m",
        env!("CARGO_PKG_VERSION"),
        config_path.display(),
    );
    for (name, tunnel) in tunnels {
        let subdomain = tunnel.subdomain.as_deref().unwrap_or("automatic subdomain");
        eprintln!("    {name}: localhost:{} ({subdomain})", tunnel.local_port);
    }
    eprintln!("  \x1b[1mConnecting...\x1b[0m\n");
}
