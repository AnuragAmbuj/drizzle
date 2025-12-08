use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use reqwest::Client;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "admin-cli")]
#[command(about = "CLI for Drizzle Admin API", long_about = None)]
struct Cli {
    #[arg(long, env = "ADMIN_URL", default_value = "http://localhost:3000")]
    url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage Tenants
    Tenant(TenantArgs),
    /// Manage Services
    Service(ServiceArgs),
    /// Manage Routes
    Route(RouteArgs),
    /// Manage API Keys
    ApiKey(ApiKeyArgs),
}

#[derive(Args)]
struct TenantArgs {
    #[command(subcommand)]
    command: TenantCommands,
}

#[derive(Subcommand)]
enum TenantCommands {
    /// Create a new tenant
    Create {
        #[arg(long)]
        slug: String,
        #[arg(long)]
        name: String,
    },
}

#[derive(Args)]
struct ServiceArgs {
    #[command(subcommand)]
    command: ServiceCommands,
}

#[derive(Subcommand)]
enum ServiceCommands {
    /// Create a new service
    Create {
        #[arg(long)]
        tenant_id: Uuid,
        #[arg(long)]
        name: String,
        #[arg(long)]
        host: String,
    },
}

#[derive(Args)]
struct RouteArgs {
    #[command(subcommand)]
    command: RouteCommands,
}

#[derive(Subcommand)]
enum RouteCommands {
    /// Create a new route
    Create {
        #[arg(long)]
        service_id: Uuid,
        #[arg(long)]
        name: String,
        #[arg(long)]
        path: String,
    },
}

#[derive(Args)]
struct ApiKeyArgs {
    #[command(subcommand)]
    command: ApiKeyCommands,
}

#[derive(Subcommand)]
enum ApiKeyCommands {
    /// Create a new API key
    Create {
        #[arg(long)]
        tenant_id: Uuid,
        #[arg(long)]
        key: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = Client::new();

    match &cli.command {
        Commands::Tenant(args) => match &args.command {
            TenantCommands::Create { slug, name } => {
                let url = format!("{}/tenants", cli.url);
                let res = client
                    .post(&url)
                    .json(&serde_json::json!({
                        "slug": slug,
                        "display_name": name
                    }))
                    .send()
                    .await?;

                print_response(res).await?;
            }
        },
        Commands::Service(args) => match &args.command {
            ServiceCommands::Create {
                tenant_id,
                name,
                host,
            } => {
                let url = format!("{}/services", cli.url);
                let res = client
                    .post(&url)
                    .json(&serde_json::json!({
                        "tenant_id": tenant_id,
                        "name": name,
                        "host": host
                    }))
                    .send()
                    .await?;
                print_response(res).await?;
            }
        },
        Commands::Route(args) => match &args.command {
            RouteCommands::Create {
                service_id,
                name,
                path,
            } => {
                let url = format!("{}/routes", cli.url);
                let res = client
                    .post(&url)
                    .json(&serde_json::json!({
                        "service_id": service_id,
                        "name": name,
                        "path": path
                    }))
                    .send()
                    .await?;
                print_response(res).await?;
            }
        }, // Added missing closing brace and comma here
        Commands::ApiKey(args) => match &args.command {
            ApiKeyCommands::Create { tenant_id, key } => {
                let url = format!("{}/api-keys", cli.url);
                let res = client
                    .post(&url)
                    .json(&serde_json::json!({
                        "tenant_id": tenant_id,
                        "key": key
                    }))
                    .send()
                    .await?;
                print_response(res).await?;
            }
        },
    }

    Ok(())
}

async fn print_response(res: reqwest::Response) -> Result<()> {
    let status = res.status();
    let text = res.text().await?;
    if status.is_success() {
        // Pretty print json if possible
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            println!("{}", serde_json::to_string_pretty(&json)?);
        } else {
            println!("{}", text);
        }
    } else {
        eprintln!("Error ({}): {}", status, text);
        std::process::exit(1);
    }
    Ok(())
}
