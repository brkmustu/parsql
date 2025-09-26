//! Parsql CLI library
//! 
//! This crate provides the command-line interface for the parsql database toolkit.

use clap::Subcommand;

pub mod config;
pub mod utils;
pub mod commands;
pub mod ui;

#[derive(Subcommand)]
pub enum MigrateCommands {
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
        
        /// Dry run - show what would be rolled back without executing
        #[arg(long)]
        dry_run: bool,
    },
    
    /// Show migration status
    #[command(alias = "s")]
    Status {
        /// Target database URL (overrides global --database-url)
        #[arg(long)]
        database_url: Option<String>,
        
        /// Show detailed information including checksums
        #[arg(long)]
        detailed: bool,
    },
    
    /// Validate migration files
    #[command(alias = "v")]
    Validate {
        /// Check for version gaps
        #[arg(long)]
        check_gaps: bool,
        
        /// Verify migration checksums
        #[arg(long)]
        verify_checksums: bool,
    },
    
    /// List migration files
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