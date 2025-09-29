//! Parsql CLI - Command-line interface for parsql database toolkit

use clap::{Parser, Subcommand};
use anyhow::{Context, Result};
use colored::Colorize;
mod commands;
mod config;
mod utils;
mod ui;

use commands::migrate;
use parsql_cli::MigrateCommands;

/// Parsql database toolkit CLI
#[derive(Parser)]
#[command(
    name = "parsql",
    version,
    author,
    about = "Type-safe SQL toolkit for Rust",
    long_about = None
)]
struct Cli {
    /// Database URL (can also be set via DATABASE_URL env var)
    #[arg(long, env = "DATABASE_URL", global = true)]
    database_url: Option<String>,

    /// Configuration file path
    #[arg(long, default_value = "parsql.toml", global = true)]
    config: String,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Launch interactive TUI mode
    #[arg(short, long)]
    interactive: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Database migration commands
    #[command(alias = "m")]
    Migrate {
        #[command(subcommand)]
        action: MigrateCommands,
    },
    
    /// Initialize a new parsql project
    Init {
        /// Project name
        #[arg(default_value = ".")]
        path: String,
    },
}


fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Set up colored output
    colored::control::set_override(true);
    
    // Load configuration
    let config = config::load_config(&cli.config)?;
    
    if cli.verbose {
        println!("{}", "Parsql CLI v0.4.0".bright_blue().bold());
        println!("{}", "================".bright_blue());
    }
    
    // Check if we should run in interactive mode
    if cli.interactive || cli.command.is_none() {
        // Launch interactive TUI
        let database_url = cli.database_url
            .or(config.database_url.clone());
        
        return ui::run_tui(database_url, config, cli.verbose);
    }
    
    // Otherwise run in command mode
    match cli.command.unwrap() {
        Commands::Migrate { action } => {
            // Some commands don't need database URL
            let needs_db = matches!(
                action,
                MigrateCommands::Run { .. } | 
                MigrateCommands::Rollback { .. } | 
                MigrateCommands::Status { .. }
            );
            
            let database_url = if needs_db {
                cli.database_url
                    .or(config.database_url.clone())
                    .context("Database URL not provided. Use --database-url or set DATABASE_URL env var")?
            } else {
                String::new()
            };
                
            migrate::handle_command(action, &database_url, &config, cli.verbose)?;
        }
        
        Commands::Init { path } => {
            init_project(&path)?;
        }
    }
    
    Ok(())
}

fn init_project(path: &str) -> Result<()> {
    use std::fs;
    use std::path::Path;
    
    let project_path = Path::new(path);
    
    // Create migrations directory
    let migrations_dir = project_path.join("migrations");
    fs::create_dir_all(&migrations_dir)
        .context("Failed to create migrations directory")?;
    
    // Create parsql.toml config file
    let config_path = project_path.join("parsql.toml");
    if !config_path.exists() {
        let default_config = r#"# Parsql configuration file

[migrations]
# Directory containing migration files
directory = "migrations"

# Table name for tracking migrations
table_name = "parsql_migrations"

# Run each migration in a transaction
transaction_per_migration = true

# Allow out-of-order migrations
allow_out_of_order = false

# Verify checksums of applied migrations
verify_checksums = true

# Database connection settings (optional, can use DATABASE_URL instead)
# [database]
# url = "postgresql://user:password@localhost/dbname"
"#;
        
        fs::write(&config_path, default_config)
            .context("Failed to create parsql.toml")?;
            
        println!("{} Created parsql.toml", "✓".green().bold());
    }
    
    // Create .gitignore if it doesn't exist
    let gitignore_path = project_path.join(".gitignore");
    if !gitignore_path.exists() {
        fs::write(&gitignore_path, "target/\n*.db\n*.db-journal\n")
            .context("Failed to create .gitignore")?;
            
        println!("{} Created .gitignore", "✓".green().bold());
    }
    
    // Create README.md with basic instructions
    let readme_path = project_path.join("README.md");
    if !readme_path.exists() {
        let readme_content = r#"# Parsql Project

This project uses [parsql](https://github.com/vvmspace/parsql) for database management.

## Getting Started

1. Set your database URL:
   ```bash
   export DATABASE_URL="postgresql://user:password@localhost/dbname"
   # or
   export DATABASE_URL="sqlite:database.db"
   ```

2. Create a new migration:
   ```bash
   parsql migrate create create_users_table
   ```

3. Run migrations:
   ```bash
   parsql migrate run
   ```

4. Check migration status:
   ```bash
   parsql migrate status
   ```

## Configuration

Edit `parsql.toml` to customize migration settings.
"#;
        
        fs::write(&readme_path, readme_content)
            .context("Failed to create README.md")?;
            
        println!("{} Created README.md", "✓".green().bold());
    }
    
    println!("\n{} Parsql project initialized successfully!", "✓".green().bold());
    println!("\nNext steps:");
    println!("  1. Set your DATABASE_URL environment variable");
    println!("  2. Create your first migration: {}", "parsql migrate create <name>".cyan());
    println!("  3. Run migrations: {}", "parsql migrate run".cyan());
    println!("  4. Rollback to a specific version: {}", "parsql migrate rollback --to <version>".cyan());
    
    Ok(())
}