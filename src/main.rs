use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;
use tracing_subscriber::EnvFilter;

use subtunnel::client::Client;
use subtunnel::server::{Server, ServerConfig};

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
    /// Run the tunnelr server (public-facing VPS).
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

        /// Domain for tunnel subdomains (e.g. tunnel.ezbackend.dev).
        #[arg(long)]
        domain: String,

        /// Additional domains to accept (can be repeated).
        #[arg(long = "extra-domain")]
        extra_domains: Vec<String>,

        /// Authentication token that agents must provide.
        #[arg(long, env = "TUNNELR_TOKEN")]
        token: Option<String>,
    },

    /// Connect to a tunnelr server and expose a local port.
    Local {
        /// Local port to expose (e.g. 8080).
        local_port: u16,

        /// Server address to connect to (host:port).
        #[arg(long)]
        to: String,

        /// Authentication token.
        #[arg(long, env = "TUNNELR_TOKEN")]
        token: String,

        /// Request a specific subdomain (e.g. "myapp" for myapp.tunnel.example.com).
        #[arg(long)]
        subdomain: Option<String>,
    },
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
        Command::Server { port, http_port, host, domain, extra_domains, token } => {
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
            };
            let server = Server::new(config);
            server.run().await?;
        }
        Command::Local {
            local_port,
            to,
            token,
            subdomain,
        } => {
            eprintln!(
                "\n  \x1b[1;32msubtunnel\x1b[0m v{}\n  \x1b[1mMode:\x1b[0m       client\n  \x1b[1mLocal:\x1b[0m      localhost:{}\n  \x1b[1mServer:\x1b[0m     {}\n  \x1b[1mConnecting...\x1b[0m\n",
                env!("CARGO_PKG_VERSION"),
                local_port,
                to,
            );

            let client = Client::new(to, token, local_port, subdomain);
            client.run(shutdown_rx).await?;
        }
    }

    Ok(())
}
