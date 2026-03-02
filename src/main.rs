use clap::Parser;
use driftguard::{Config, db::Database, cli::Cli, api};
use rusqlite::Error as SqliteError;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        ))
        .init();
    
    let cli = Cli::parse();
    let config = Config::from_env();
    
    // Ensure data directory exists
    let data_dir = std::path::Path::new(&config.database_url)
        .parent()
        .unwrap_or(std::path::Path::new("./data"));
    std::fs::create_dir_all(data_dir)?;
    
    match &cli.command {
        driftguard::cli::Commands::Serve { port } => {
            println!("🚀 Starting API server on port {}...", port);
            let db = Database::new(&config.database_url).map_err(|e: SqliteError| e.to_string())?;
            api::run_api(db, *port).await;
        }
        _ => {
            run_cli(&config, &cli).await?;
        }
    }
    
    Ok(())
}

async fn run_cli(config: &Config, cli: &Cli) -> Result<(), String> {
    let db = Database::new(&config.database_url).map_err(|e: SqliteError| e.to_string())?;
    
    match &cli.command {
        driftguard::cli::Commands::Add { url, interval, name } => {
            let mut monitor = driftguard::db::Monitor::new(url.clone(), *interval);
            monitor.name = name.clone();
            db.add_monitor(&monitor).map_err(|e: SqliteError| e.to_string())?;
            println!("✅ Added monitor: {} (ID: {})", url, monitor.id);
            Ok(())
        }
        
        driftguard::cli::Commands::List => {
            let monitors = db.get_monitors().map_err(|e: SqliteError| e.to_string())?;
            if monitors.is_empty() {
                println!("No monitors configured. Use 'driftguard add <url>' to add one.");
            } else {
                println!("📋 Monitors:\n");
                for m in &monitors {
                    println!("  [{}] {}", m.id.chars().take(8).collect::<String>(), m.url);
                    if let Some(name) = &m.name {
                        println!("       Name: {}", name);
                    }
                    println!("       Interval: {}s | Enabled: {}\n", m.interval_seconds, m.enabled);
                }
            }
            Ok(())
        }
        
        driftguard::cli::Commands::Status => {
            let monitors = db.get_monitors().map_err(|e: SqliteError| e.to_string())?;
            println!("📊 Monitor Status:\n");
            for m in &monitors {
                let status = m.last_status.as_deref().unwrap_or("never");
                let last = m.last_check.as_deref().unwrap_or("never");
                println!("  {} → {}", m.url, status);
                println!("     Last check: {} | Next: in {}s\n", last, m.interval_seconds);
            }
            Ok(())
        }
        
        driftguard::cli::Commands::Changes { monitor_id } => {
            let changes = db.get_changes(monitor_id.as_deref()).map_err(|e: SqliteError| e.to_string())?;
            if changes.is_empty() {
                println!("No changes detected yet.");
            } else {
                println!("📝 Recent Changes:\n");
                for c in changes.iter().take(20) {
                    println!("  [{}] {}", c.detected_at, c.change_type);
                    if let Some(details) = &c.details {
                        println!("       {}", details);
                    }
                }
            }
            Ok(())
        }
        
        driftguard::cli::Commands::Remove { id } => {
            db.delete_monitor(id).map_err(|e: SqliteError| e.to_string())?;
            println!("✅ Removed monitor: {}", id);
            Ok(())
        }
        
        driftguard::cli::Commands::Check { url } => {
            println!("🔍 Checking {}...", url);
            let client = reqwest::Client::new();
            match client.get(url).send().await {
                Ok(resp) => {
                    println!("   Status: {}", resp.status());
                    println!("   Size: {} bytes", resp.content_length().unwrap_or(0));
                }
                Err(e) => {
                    println!("   ❌ Error: {}", e);
                }
            }
            Ok(())
        }
        
        driftguard::cli::Commands::Serve { .. } => {
            // Already handled above
            Ok(())
        }
    }
}
