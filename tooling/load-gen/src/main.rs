use clap::{Parser, Subcommand};

mod attack;
mod provision;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Admin API URL
    #[arg(long, default_value = "http://localhost:3000")]
    admin_url: String,

    /// Gateway URL
    #[arg(long, default_value = "http://localhost:6188")]
    gateway_url: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Provision random tenants, services, and keys
    Provision {
        /// Number of tenants to create
        #[arg(long, default_value_t = 10)]
        tenants: usize,

        /// Services per tenant
        #[arg(long, default_value_t = 5)]
        services: usize,

        /// API Keys per tenant
        #[arg(long, default_value_t = 2)]
        keys: usize,
    },
    /// Run load test attack
    Attack {
        /// Duration in seconds
        #[arg(long, default_value_t = 30)]
        duration: u64,

        /// Requests per second (target)
        #[arg(long, default_value_t = 100)]
        rate: u64,

        /// Number of concurrent workers
        #[arg(long, default_value_t = 10)]
        workers: usize,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let client = reqwest::Client::new();

    match &cli.command {
        Commands::Provision {
            tenants,
            services,
            keys,
        } => {
            provision::run(&client, &cli.admin_url, *tenants, *services, *keys).await;
        }
        Commands::Attack {
            duration,
            rate,
            workers,
        } => {
            attack::run(
                &client,
                &cli.gateway_url,
                &cli.admin_url,
                *duration,
                *rate,
                *workers,
            )
            .await;
        }
    }
}
