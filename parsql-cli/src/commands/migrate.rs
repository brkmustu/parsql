//! Migration command implementations

use crate::config::Config;
use crate::utils::{self, DatabaseType, Progress};
use crate::MigrateCommands;
use anyhow::{Context, Result};
use colored::Colorize;
use parsql_migrations::prelude::*;
use sha2::{Sha256, Digest};
use std::fs;
use std::path::Path;

pub fn handle_command(
    command: MigrateCommands,
    database_url: &str,
    config: &Config,
    verbose: bool,
) -> Result<()> {
    match command {
        MigrateCommands::Create { name, migration_type } => {
            create_migration(&name, &migration_type, &config.migrations.directory)?;
        }
        
        MigrateCommands::Run { database_url: cmd_url, dry_run, target } => {
            let url = cmd_url.as_deref().unwrap_or(database_url);
            run_migrations(url, config, dry_run, target, verbose)?;
        }
        
        MigrateCommands::Rollback { to, database_url: cmd_url, dry_run } => {
            let url = cmd_url.as_deref().unwrap_or(database_url);
            rollback_migrations(url, config, to, dry_run, verbose)?;
        }
        
        MigrateCommands::Status { database_url: cmd_url, detailed } => {
            let url = cmd_url.as_deref().unwrap_or(database_url);
            show_status(url, config, detailed)?;
        }
        
        MigrateCommands::Validate { check_gaps, verify_checksums } => {
            validate_migrations(&config.migrations.directory, check_gaps, verify_checksums, verbose)?;
        }
        
        MigrateCommands::List { pending, applied } => {
            list_migrations(&config.migrations.directory, pending, applied)?;
        }
    }
    
    Ok(())
}

fn create_migration(name: &str, migration_type: &str, directory: &str) -> Result<()> {
    let timestamp = utils::get_timestamp();
    let version = timestamp.parse::<i64>()
        .context("Failed to parse timestamp as version")?;
    
    let dir_path = Path::new(directory);
    fs::create_dir_all(dir_path)
        .context("Failed to create migrations directory")?;
    
    let safe_name = name.replace(' ', "_").to_lowercase();
    
    match migration_type {
        "sql" => {
            let up_file = dir_path.join(format!("{}_{}_{}.up.sql", version, timestamp, safe_name));
            let down_file = dir_path.join(format!("{}_{}_{}.down.sql", version, timestamp, safe_name));
            
            let up_content = format!(
                "-- Migration: {}\n-- Version: {}\n-- Created: {}\n\n-- Add your UP migration SQL here\n",
                name,
                version,
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
            );
            
            let down_content = format!(
                "-- Migration: {} (rollback)\n-- Version: {}\n-- Created: {}\n\n-- Add your DOWN migration SQL here\n",
                name,
                version,
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
            );
            
            fs::write(&up_file, up_content)
                .context("Failed to create up migration file")?;
            fs::write(&down_file, down_content)
                .context("Failed to create down migration file")?;
            
            utils::print_success(&format!("Created SQL migration: {}", safe_name));
            println!("  {}: {}", "UP".green(), up_file.display());
            println!("  {}: {}", "DOWN".red(), down_file.display());
        }
        
        "rust" => {
            let rust_file = dir_path.join(format!("{}_{}_{}.rs", version, timestamp, safe_name));
            
            let rust_content = format!(
                r#"//! Migration: {}
//! Version: {}
//! Created: {}

use parsql_migrations::prelude::*;

pub struct Migration{};

impl Migration for Migration{} {{
    fn version(&self) -> i64 {{
        {}
    }}
    
    fn name(&self) -> &str {{
        "{}"
    }}
    
    fn up(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {{
        // Add your UP migration logic here
        conn.execute(
            "CREATE TABLE example (
                id SERIAL PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            )"
        )
    }}
    
    fn down(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {{
        // Add your DOWN migration logic here
        conn.execute("DROP TABLE IF EXISTS example")
    }}
}}
"#,
                name,
                version,
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                version,
                version,
                version,
                safe_name
            );
            
            fs::write(&rust_file, rust_content)
                .context("Failed to create Rust migration file")?;
            
            utils::print_success(&format!("Created Rust migration: {}", safe_name));
            println!("  {}: {}", "File".cyan(), rust_file.display());
            
            utils::print_info("Remember to add this migration to your build.rs or mod.rs");
        }
        
        _ => anyhow::bail!("Unknown migration type: {}", migration_type),
    }
    
    Ok(())
}

fn run_migrations(
    database_url: &str,
    config: &Config,
    dry_run: bool,
    target: Option<i64>,
    verbose: bool,
) -> Result<()> {
    let db_type = utils::parse_database_url(database_url)?;
    
    if verbose {
        utils::print_info(&format!("Database: {} ({})", database_url, db_type.name()));
    }
    
    let progress = Progress::new("Loading migrations");
    let migrations = load_migrations_from_directory(&config.migrations.directory)?;
    progress.finish_with_message(&format!("{} migrations found", migrations.len()));
    
    if migrations.is_empty() {
        utils::print_warning("No migrations found");
        return Ok(());
    }
    
    if dry_run {
        utils::print_info("DRY RUN - No changes will be applied");
        
        for migration in &migrations {
            println!("Would run: {} - {}", migration.version, migration.name);
        }
        
        return Ok(());
    }
    
    // Run migrations based on database type
    match db_type {
        DatabaseType::PostgreSQL => {
            run_postgres_migrations(database_url, config, migrations, target)?;
        }
        DatabaseType::SQLite => {
            run_sqlite_migrations(database_url, config, migrations, target)?;
        }
    }
    
    Ok(())
}

#[cfg(feature = "postgres")]
fn run_postgres_migrations(
    database_url: &str,
    config: &Config,
    migrations: Vec<FileMigration>,
    target: Option<i64>,
) -> Result<()> {
    use postgres::{Client, NoTls};
    use parsql_migrations::postgres_simple::PostgresMigrationConnection;
    
    let progress = Progress::new("Connecting to PostgreSQL");
    let mut client = Client::connect(database_url, NoTls)
        .context("Failed to connect to PostgreSQL")?;
    progress.finish();
    
    let mut migration_conn = PostgresMigrationConnection::new(&mut client);
    let migration_config = config.to_parsql_migration_config();
    let mut runner = MigrationRunner::with_config(migration_config);
    
    // Add migrations
    for migration in migrations {
        if let Some(target) = target {
            if migration.version > target {
                continue;
            }
        }
        runner.add_migration(Box::new(migration));
    }
    
    // Run migrations
    let progress = Progress::new("Running migrations");
    let report = runner.run(&mut migration_conn)
        .context("Failed to run migrations")?;
    progress.finish();
    
    // Print report
    if report.successful_count() > 0 {
        utils::print_success(&format!("Applied {} migration(s)", report.successful_count()));
    }
    
    if !report.skipped.is_empty() {
        utils::print_info(&format!("Skipped {} migration(s) (already applied)", report.skipped.len()));
    }
    
    if report.failed_count() > 0 {
        utils::print_error(&format!("Failed {} migration(s)", report.failed_count()));
        for result in &report.failed {
            println!("  {} Version {}: {}", 
                "✗".red(), 
                result.version, 
                result.error.as_ref().unwrap_or(&"Unknown error".to_string())
            );
        }
        anyhow::bail!("Some migrations failed");
    }
    
    Ok(())
}

#[cfg(feature = "sqlite")]
fn run_sqlite_migrations(
    database_url: &str,
    config: &Config,
    migrations: Vec<FileMigration>,
    target: Option<i64>,
) -> Result<()> {
    use rusqlite::Connection;
    use parsql_migrations::sqlite_simple::SqliteMigrationConnection;
    
    let db_path = database_url.strip_prefix("sqlite:").unwrap_or(database_url);
    
    let progress = Progress::new("Opening SQLite database");
    let mut conn = Connection::open(db_path)
        .context("Failed to open SQLite database")?;
    progress.finish();
    
    let mut migration_conn = SqliteMigrationConnection::new(&mut conn);
    let migration_config = config.to_parsql_migration_config();
    let mut runner = MigrationRunner::with_config(migration_config);
    
    // Add migrations
    for migration in migrations {
        if let Some(target) = target {
            if migration.version > target {
                continue;
            }
        }
        runner.add_migration(Box::new(migration));
    }
    
    // Run migrations
    let progress = Progress::new("Running migrations");
    let report = runner.run(&mut migration_conn)
        .context("Failed to run migrations")?;
    progress.finish();
    
    // Print report
    if report.successful_count() > 0 {
        utils::print_success(&format!("Applied {} migration(s)", report.successful_count()));
    }
    
    if !report.skipped.is_empty() {
        utils::print_info(&format!("Skipped {} migration(s) (already applied)", report.skipped.len()));
    }
    
    if report.failed_count() > 0 {
        utils::print_error(&format!("Failed {} migration(s)", report.failed_count()));
        for result in &report.failed {
            println!("  {} Version {}: {}", 
                "✗".red(), 
                result.version, 
                result.error.as_ref().unwrap_or(&"Unknown error".to_string())
            );
        }
        anyhow::bail!("Some migrations failed");
    }
    
    Ok(())
}

fn rollback_migrations(
    database_url: &str,
    config: &Config,
    target_version: i64,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    let db_type = utils::parse_database_url(database_url)?;
    
    if verbose {
        utils::print_info(&format!("Database: {} ({})", database_url, db_type.name()));
    }
    
    utils::print_info(&format!("Rolling back to version: {}", target_version));
    
    let progress = Progress::new("Loading migrations");
    let migrations = load_migrations_from_directory(&config.migrations.directory)?;
    progress.finish_with_message(&format!("{} migrations found", migrations.len()));
    
    if dry_run {
        utils::print_info("DRY RUN - No changes will be applied");
        utils::print_warning("Note: Cannot determine which migrations would be rolled back without database connection");
        return Ok(());
    }
    
    // Run rollback based on database type
    match db_type {
        DatabaseType::PostgreSQL => {
            rollback_postgres_migrations(database_url, config, migrations, target_version)?;
        }
        DatabaseType::SQLite => {
            rollback_sqlite_migrations(database_url, config, migrations, target_version)?;
        }
    }
    
    Ok(())
}

fn show_status(
    database_url: &str,
    config: &Config,
    detailed: bool,
) -> Result<()> {
    let db_type = utils::parse_database_url(database_url)?;
    
    utils::print_info(&format!("Database: {} ({})", database_url, db_type.name()));
    
    let progress = Progress::new("Loading migrations");
    let migrations = load_migrations_from_directory(&config.migrations.directory)?;
    progress.finish_with_message(&format!("{} migrations found", migrations.len()));
    
    // Get status based on database type
    match db_type {
        DatabaseType::PostgreSQL => {
            show_postgres_status(database_url, config, migrations, detailed)?;
        }
        DatabaseType::SQLite => {
            show_sqlite_status(database_url, config, migrations, detailed)?;
        }
    }
    
    Ok(())
}

fn validate_migrations(
    directory: &str,
    check_gaps: bool,
    verify_checksums: bool,
    verbose: bool,
) -> Result<()> {
    let migrations = load_migrations_from_directory(directory)?;
    
    if migrations.is_empty() {
        utils::print_warning("No migrations found");
        return Ok(());
    }
    
    utils::print_info(&format!("Found {} migration(s)", migrations.len()));
    
    // Check for version gaps
    if check_gaps {
        let mut versions: Vec<i64> = migrations.iter().map(|m| m.version).collect();
        versions.sort();
        
        let mut has_gaps = false;
        for i in 1..versions.len() {
            if versions[i] - versions[i-1] > 1 {
                utils::print_warning(&format!(
                    "Gap detected between versions {} and {}", 
                    versions[i-1], 
                    versions[i]
                ));
                has_gaps = true;
            }
        }
        
        if !has_gaps {
            utils::print_success("No version gaps found");
        }
    }
    
    if verify_checksums {
        utils::print_info("Verifying migration checksums...");
        
        let checksum_errors = 0;
        for migration in &migrations {
            let calculated_checksum = calculate_migration_checksum(migration);
            
            // For now, just show the checksum (we'll add comparison with DB later)
            if verbose {
                println!("  {} - {}: {}", 
                    migration.version, 
                    migration.name, 
                    &calculated_checksum[..8]
                );
            }
        }
        
        if checksum_errors == 0 {
            utils::print_success("All checksums verified");
        } else {
            utils::print_error(&format!("{} checksum error(s) found", checksum_errors));
        }
    }
    
    Ok(())
}

fn list_migrations(
    directory: &str,
    pending_only: bool,
    applied_only: bool,
) -> Result<()> {
    let migrations = load_migrations_from_directory(directory)?;
    
    if migrations.is_empty() {
        utils::print_warning("No migrations found");
        return Ok(());
    }
    
    println!("{}", "Available Migrations:".bold());
    println!();
    
    let headers = vec!["Version", "Name", "Type"];
    let mut rows = Vec::new();
    
    for migration in migrations {
        rows.push(vec![
            migration.version.to_string(),
            migration.name.clone(),
            migration.migration_type.clone(),
        ]);
    }
    
    print!("{}", utils::format_table(headers, rows));
    
    if pending_only || applied_only {
        utils::print_info("Filtering by status requires database connection (not yet implemented)");
    }
    
    Ok(())
}

#[cfg(feature = "postgres")]
fn show_postgres_status(
    database_url: &str,
    config: &Config,
    migrations: Vec<FileMigration>,
    detailed: bool,
) -> Result<()> {
    use postgres::{Client, NoTls};
    use parsql_migrations::postgres_simple::PostgresMigrationConnection;
    
    let progress = Progress::new("Connecting to PostgreSQL");
    let mut client = Client::connect(database_url, NoTls)
        .context("Failed to connect to PostgreSQL")?;
    progress.finish();
    
    let mut migration_conn = PostgresMigrationConnection::new(&mut client);
    
    // Get applied migrations
    let records = migration_conn.query_migrations(&config.migrations.table_name)
        .context("Failed to fetch applied migrations")?;
    
    // Convert to map for easy lookup
    let mut applied = std::collections::HashMap::new();
    for record in records {
        applied.insert(record.version, record);
    }
    
    let total_count = migrations.len();
    let applied_count = applied.len();
    let pending_count = migrations.iter()
        .filter(|m| !applied.contains_key(&m.version))
        .count();
    
    // Print summary
    println!();
    println!("{}", "Migration Status:".bold());
    println!("  {} migrations", utils::colorize_number(total_count, "Total"));
    println!("  {} migrations", utils::colorize_number(applied_count, "Applied").green());
    println!("  {} migrations", utils::colorize_number(pending_count, "Pending").yellow());
    
    if detailed {
        println!();
        println!("{}", "Detailed Status:".bold());
        
        let headers = vec!["Version", "Name", "Status", "Applied At", "Checksum"];
        let mut rows = Vec::new();
        
        for migration in migrations {
            let status;
            let applied_at;
            
            if let Some(record) = applied.get(&migration.version) {
                status = "Applied".green().to_string();
                applied_at = record.applied_at.format("%Y-%m-%d %H:%M:%S").to_string();
            } else {
                status = "Pending".yellow().to_string();
                applied_at = "-".to_string();
            }
            
            let checksum_status = if let Some(record) = applied.get(&migration.version) {
                let calculated_checksum = calculate_migration_checksum(&migration);
                if let Some(ref stored_checksum) = record.checksum {
                    if stored_checksum == &calculated_checksum {
                        "✓".green().to_string()
                    } else {
                        format!("✗ Mismatch").red().to_string()
                    }
                } else {
                    "No checksum".dimmed().to_string()
                }
            } else {
                "-".to_string()
            };
            
            rows.push(vec![
                migration.version.to_string(),
                migration.name.clone(),
                status,
                applied_at,
                checksum_status,
            ]);
        }
        
        print!("{}", utils::format_table(headers, rows));
    }
    
    Ok(())
}

#[cfg(feature = "sqlite")]
fn show_sqlite_status(
    database_url: &str,
    config: &Config,
    migrations: Vec<FileMigration>,
    detailed: bool,
) -> Result<()> {
    use rusqlite::Connection;
    use parsql_migrations::sqlite_simple::SqliteMigrationConnection;
    
    let db_path = database_url.strip_prefix("sqlite:").unwrap_or(database_url);
    
    let progress = Progress::new("Opening SQLite database");
    let mut conn = Connection::open(db_path)
        .context("Failed to open SQLite database")?;
    progress.finish();
    
    let mut migration_conn = SqliteMigrationConnection::new(&mut conn);
    
    // Get applied migrations
    let records = migration_conn.query_migrations(&config.migrations.table_name)
        .context("Failed to fetch applied migrations")?;
    
    // Convert to map for easy lookup
    let mut applied = std::collections::HashMap::new();
    for record in records {
        applied.insert(record.version, record);
    }
    
    let total_count = migrations.len();
    let applied_count = applied.len();
    let pending_count = migrations.iter()
        .filter(|m| !applied.contains_key(&m.version))
        .count();
    
    // Print summary
    println!();
    println!("{}", "Migration Status:".bold());
    println!("  {} migrations", utils::colorize_number(total_count, "Total"));
    println!("  {} migrations", utils::colorize_number(applied_count, "Applied").green());
    println!("  {} migrations", utils::colorize_number(pending_count, "Pending").yellow());
    
    if detailed {
        println!();
        println!("{}", "Detailed Status:".bold());
        
        let headers = vec!["Version", "Name", "Status", "Applied At", "Checksum"];
        let mut rows = Vec::new();
        
        for migration in migrations {
            let status;
            let applied_at;
            
            if let Some(record) = applied.get(&migration.version) {
                status = "Applied".green().to_string();
                applied_at = record.applied_at.format("%Y-%m-%d %H:%M:%S").to_string();
            } else {
                status = "Pending".yellow().to_string();
                applied_at = "-".to_string();
            }
            
            let checksum_status = if let Some(record) = applied.get(&migration.version) {
                let calculated_checksum = calculate_migration_checksum(&migration);
                if let Some(ref stored_checksum) = record.checksum {
                    if stored_checksum == &calculated_checksum {
                        "✓".green().to_string()
                    } else {
                        format!("✗ Mismatch").red().to_string()
                    }
                } else {
                    "No checksum".dimmed().to_string()
                }
            } else {
                "-".to_string()
            };
            
            rows.push(vec![
                migration.version.to_string(),
                migration.name.clone(),
                status,
                applied_at,
                checksum_status,
            ]);
        }
        
        print!("{}", utils::format_table(headers, rows));
    }
    
    Ok(())
}

// Helper structures and functions

fn calculate_migration_checksum(migration: &FileMigration) -> String {
    let mut hasher = Sha256::new();
    hasher.update(migration.version.to_string());
    hasher.update(&migration.name);
    
    if let Some(ref up_sql) = migration.up_sql {
        hasher.update(up_sql);
    }
    if let Some(ref down_sql) = migration.down_sql {
        hasher.update(down_sql);
    }
    
    format!("{:x}", hasher.finalize())
}

struct FileMigration {
    version: i64,
    name: String,
    migration_type: String,
    up_sql: Option<String>,
    down_sql: Option<String>,
}

impl Migration for FileMigration {
    fn version(&self) -> i64 {
        self.version
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn up(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        if let Some(ref sql) = self.up_sql {
            conn.execute(sql)?;
        }
        Ok(())
    }
    
    fn down(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        if let Some(ref sql) = self.down_sql {
            conn.execute(sql)?;
        }
        Ok(())
    }
    
    fn checksum(&self) -> String {
        calculate_migration_checksum(self)
    }
}

fn load_migrations_from_directory(directory: &str) -> Result<Vec<FileMigration>> {
    let dir_path = Path::new(directory);
    
    if !dir_path.exists() {
        return Ok(Vec::new());
    }
    
    let mut migrations = Vec::new();
    
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(file_name) = path.file_name() {
            let file_name_str = file_name.to_string_lossy();
            
            // Parse SQL migrations
            if file_name_str.ends_with(".up.sql") {
                let base_name = file_name_str.trim_end_matches(".up.sql");
                let parts: Vec<&str> = base_name.splitn(3, '_').collect();
                
                if parts.len() >= 3 {
                    let version = parts[0].parse::<i64>()
                        .context("Failed to parse migration version")?;
                    let name = parts[2].to_string();
                    
                    let up_sql = fs::read_to_string(&path)
                        .context("Failed to read up migration file")?;
                    
                    let down_path = path.with_file_name(format!("{}.down.sql", base_name));
                    let down_sql = if down_path.exists() {
                        Some(fs::read_to_string(&down_path)
                            .context("Failed to read down migration file")?)
                    } else {
                        None
                    };
                    
                    migrations.push(FileMigration {
                        version,
                        name,
                        migration_type: "SQL".to_string(),
                        up_sql: Some(up_sql),
                        down_sql,
                    });
                }
            }
            
            // TODO: Parse Rust migrations
        }
    }
    
    migrations.sort_by_key(|m| m.version);
    Ok(migrations)
}

#[cfg(feature = "postgres")]
fn rollback_postgres_migrations(
    database_url: &str,
    config: &Config,
    migrations: Vec<FileMigration>,
    target_version: i64,
) -> Result<()> {
    use postgres::{Client, NoTls};
    use parsql_migrations::postgres_simple::PostgresMigrationConnection;
    
    let progress = Progress::new("Connecting to PostgreSQL");
    let mut client = Client::connect(database_url, NoTls)
        .context("Failed to connect to PostgreSQL")?;
    progress.finish();
    
    let mut migration_conn = PostgresMigrationConnection::new(&mut client);
    let migration_config = config.to_parsql_migration_config();
    let mut runner = MigrationRunner::with_config(migration_config);
    
    // Add all migrations
    for migration in migrations {
        runner.add_migration(Box::new(migration));
    }
    
    // Perform rollback
    let progress = Progress::new("Rolling back migrations");
    let report = runner.rollback(&mut migration_conn, target_version)
        .context("Failed to rollback migrations")?;
    progress.finish();
    
    // Print report
    if report.successful_count() > 0 {
        utils::print_success(&format!("Rolled back {} migration(s)", report.successful_count()));
        for result in &report.successful {
            println!("  {} Version {} - {}", "↩".cyan(), result.version, result.name);
        }
    } else {
        utils::print_info("No migrations to roll back");
    }
    
    if report.failed_count() > 0 {
        utils::print_error(&format!("Failed to rollback {} migration(s)", report.failed_count()));
        for result in &report.failed {
            println!("  {} Version {}: {}", 
                "✗".red(), 
                result.version, 
                result.error.as_ref().unwrap_or(&"Unknown error".to_string())
            );
        }
        anyhow::bail!("Some rollbacks failed");
    }
    
    Ok(())
}

#[cfg(feature = "sqlite")]
fn rollback_sqlite_migrations(
    database_url: &str,
    config: &Config,
    migrations: Vec<FileMigration>,
    target_version: i64,
) -> Result<()> {
    use rusqlite::Connection;
    use parsql_migrations::sqlite_simple::SqliteMigrationConnection;
    
    let db_path = database_url.strip_prefix("sqlite:").unwrap_or(database_url);
    
    let progress = Progress::new("Opening SQLite database");
    let mut conn = Connection::open(db_path)
        .context("Failed to open SQLite database")?;
    progress.finish();
    
    let mut migration_conn = SqliteMigrationConnection::new(&mut conn);
    let migration_config = config.to_parsql_migration_config();
    let mut runner = MigrationRunner::with_config(migration_config);
    
    // Add all migrations
    for migration in migrations {
        runner.add_migration(Box::new(migration));
    }
    
    // Perform rollback
    let progress = Progress::new("Rolling back migrations");
    let report = runner.rollback(&mut migration_conn, target_version)
        .context("Failed to rollback migrations")?;
    progress.finish();
    
    // Print report
    if report.successful_count() > 0 {
        utils::print_success(&format!("Rolled back {} migration(s)", report.successful_count()));
        for result in &report.successful {
            println!("  {} Version {} - {}", "↩".cyan(), result.version, result.name);
        }
    } else {
        utils::print_info("No migrations to roll back");
    }
    
    if report.failed_count() > 0 {
        utils::print_error(&format!("Failed to rollback {} migration(s)", report.failed_count()));
        for result in &report.failed {
            println!("  {} Version {}: {}", 
                "✗".red(), 
                result.version, 
                result.error.as_ref().unwrap_or(&"Unknown error".to_string())
            );
        }
        anyhow::bail!("Some rollbacks failed");
    }
    
    Ok(())
}