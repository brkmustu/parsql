//! Parsql CLI - Command-line interface for parsql database toolkit

use clap::{Parser, Subcommand};
use anyhow::{Context, Result};
use colored::Colorize;

mod commands;
mod config;
mod utils;

use commands::migrate;

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

    #[command(subcommand)]
    command: Commands,
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

#[derive(Subcommand)]
enum MigrateCommands {
    /// Create a new migration
    #[command(alias = "c")]
    Create {
        /// Migration name (e.g., "create_users_table")
        name: String,
        
        /// Migration type
        #[arg(short = 't', long, default_value = "sql", value_parser = ["sql", "rust"])]
        migration_type: String,
    },
    
    /// Run pending migrations
    #[command(alias = "r")]
    Run {
        /// Target database URL (overrides global --database-url)
        #[arg(long)]
        database_url: Option<String>,
        
        /// Dry run - show what would be executed without applying
        #[arg(long)]
        dry_run: bool,
        
        /// Target version (run up to this version)
        #[arg(long)]
        target: Option<i64>,
    },
    
    /// Rollback migrations
    #[command(alias = "b")]
    Rollback {
        /// Target version to rollback to
        #[arg(long, short = 't')]
        to: i64,
        
        /// Target database URL (overrides global --database-url)
        #[arg(long)]
        database_url: Option<String>,
        
        /// Dry run - show what would be rolled back
        #[arg(long)]
        dry_run: bool,
    },
    
    /// Show migration status
    #[command(alias = "s")]
    Status {
        /// Target database URL (overrides global --database-url)
        #[arg(long)]
        database_url: Option<String>,
        
        /// Show detailed information
        #[arg(long)]
        detailed: bool,
    },
    
    /// Validate migrations
    #[command(alias = "v")]
    Validate {
        /// Check for version gaps
        #[arg(long, default_value_t = true)]
        check_gaps: bool,
        
        /// Verify checksums of applied migrations
        #[arg(long)]
        verify_checksums: bool,
    },
    
    /// List available migrations
    #[command(alias = "l")]
    List {
        /// Show only pending migrations
        #[arg(long)]
        pending: bool,
        
        /// Show only applied migrations
        #[arg(long)]
        applied: bool,
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
    
    match cli.command {
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
    
    Ok(())
}
