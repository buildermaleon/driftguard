use clap::{Parser, Subcommand};
use crate::db::{Database, Monitor};
use crate::Config;
use rusqlite::Error as SqliteError;

#[derive(Parser)]
#[command(name = "driftguard")]
#[command(about = "Real-time web monitoring and change detection", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Config file path
    #[arg(short, long, default_value = "./data/driftguard.db")]
    pub config: String,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new URL to monitor
    Add {
        /// URL to monitor
        url: String,
        
        /// Check interval in seconds
        #[arg(short, long, default_value = "3600")]
        interval: u64,
        
        /// Optional name for this monitor
        #[arg(short, long)]
        name: Option<String>,
    },
    
    /// List all monitored URLs
    List,
    
    /// Show status of all monitors
    Status,
    
    /// Show detected changes
    Changes {
        /// Filter by monitor ID
        #[arg(short, long)]
        monitor_id: Option<String>,
    },
    
    /// Remove a monitor
    Remove {
        /// Monitor ID to remove
        id: String,
    },
    
    /// Check a specific URL now
    Check {
        /// URL to check
        url: String,
    },
    
    /// Start the API server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
}

pub async fn run_cli(config: &Config, cli: &Cli) -> Result<(), String> {
    let db = Database::new(&config.database_url).map_err(|e: SqliteError| e.to_string())?;
    
    match &cli.command {
        Commands::Add { url, interval, name } => {
            let mut monitor = Monitor::new(url.clone(), *interval);
            monitor.name = name.clone();
            db.add_monitor(&monitor).map_err(|e: SqliteError| e.to_string())?;
            println!("✅ Added monitor: {} (ID: {})", url, monitor.id);
            Ok(())
        }
        
        Commands::List => {
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
        
        Commands::Status => {
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
        
        Commands::Changes { monitor_id } => {
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
        
        Commands::Remove { id } => {
            db.delete_monitor(id).map_err(|e: SqliteError| e.to_string())?;
            println!("✅ Removed monitor: {}", id);
            Ok(())
        }
        
        Commands::Check { url } => {
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
        
        Commands::Serve { port } => {
            println!("🚀 Starting API server on port {}...", port);
            let db = Database::new(&config.database_url).map_err(|e: SqliteError| e.to_string())?;
            crate::api::run_api(db, *port).await;
            Ok(())
        }
    }
}
