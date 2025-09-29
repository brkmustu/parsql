//! Integration tests for parsql-cli migration operations
//! Tests core functionality with real SQLite database

use parsql_cli::{config::Config, MigrateCommands, commands::migrate};
use std::fs;
use std::path::Path;
use anyhow::Result;

// Import the migration handling function
mod common;
use common::{setup_test_env, create_test_migration, assert_migration_applied};

#[tokio::test]
async fn test_migration_lifecycle() -> Result<()> {
    let temp_dir = setup_test_env()?;
    let config = Config::default_with_directory(temp_dir.path().join("migrations").to_str().unwrap());
    let db_path = temp_dir.path().join("test.db");
    let database_url = format!("sqlite:{}", db_path.display());

    // Test 1: Create a migration
    create_test_migration(&config.migrations.directory, "create_users", "sql")?;
    
    // Verify migration file was created
    let migrations_dir = Path::new(&config.migrations.directory);
    let migration_files: Vec<_> = fs::read_dir(migrations_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_name().to_string_lossy().contains("create_users")
        })
        .collect();
    
    assert_eq!(migration_files.len(), 2, "Should create both up and down migration files");

    // Test 2: Run migrations
    let result = migrate::handle_command(
        MigrateCommands::Run {
            database_url: Some(database_url.clone()),
            dry_run: false,
            target: None,
        },
        &database_url,
        &config,
        false,
    );
    
    assert!(result.is_ok(), "Migration run should succeed");

    // Test 3: Check status
    let result = migrate::handle_command(
        MigrateCommands::Status {
            database_url: Some(database_url.clone()),
            detailed: true,
        },
        &database_url,
        &config,
        false,
    );
    
    assert!(result.is_ok(), "Status check should succeed");

    // Test 4: Verify migration was applied
    assert_migration_applied(&database_url, "users")?;

    Ok(())
}

#[tokio::test]
async fn test_rollback_functionality() -> Result<()> {
    let temp_dir = setup_test_env()?;
    let config = Config::default_with_directory(temp_dir.path().join("migrations").to_str().unwrap());
    let db_path = temp_dir.path().join("test.db");
    let database_url = format!("sqlite:{}", db_path.display());

    // Create and run initial migration
    create_test_migration(&config.migrations.directory, "create_posts", "sql")?;
    std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure different timestamps
    
    migrate::handle_command(
        MigrateCommands::Run {
            database_url: Some(database_url.clone()),
            dry_run: false,
            target: None,
        },
        &database_url,
        &config,
        false,
    )?;

    // Verify table exists
    assert_migration_applied(&database_url, "posts")?;

    // Create second migration
    std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure different timestamps
    create_test_migration(&config.migrations.directory, "add_comments", "sql")?;
    
    migrate::handle_command(
        MigrateCommands::Run {
            database_url: Some(database_url.clone()),
            dry_run: false,
            target: None,
        },
        &database_url,
        &config,
        false,
    )?;

    // Get the first migration version for rollback
    let first_version = get_first_migration_version(&config.migrations.directory)?;

    // Test rollback
    let result = migrate::handle_command(
        MigrateCommands::Rollback {
            to: first_version,
            database_url: Some(database_url.clone()),
            dry_run: false,
        },
        &database_url,
        &config,
        false,
    );

    assert!(result.is_ok(), "Rollback should succeed");

    Ok(())
}

#[tokio::test]
async fn test_dry_run_mode() -> Result<()> {
    let temp_dir = setup_test_env()?;
    let config = Config::default_with_directory(temp_dir.path().join("migrations").to_str().unwrap());
    let db_path = temp_dir.path().join("test.db");
    let database_url = format!("sqlite:{}", db_path.display());

    // Create test migration
    create_test_migration(&config.migrations.directory, "create_categories", "sql")?;

    // Test dry run
    let result = migrate::handle_command(
        MigrateCommands::Run {
            database_url: Some(database_url.clone()),
            dry_run: true,
            target: None,
        },
        &database_url,
        &config,
        false,
    );

    assert!(result.is_ok(), "Dry run should succeed");

    // Verify no actual changes were made (database file shouldn't exist)
    assert!(!db_path.exists(), "Dry run should not create database file");

    Ok(())
}

#[tokio::test]
async fn test_migration_validation() -> Result<()> {
    let temp_dir = setup_test_env()?;
    let config = Config::default_with_directory(temp_dir.path().join("migrations").to_str().unwrap());

    // Create test migrations
    create_test_migration(&config.migrations.directory, "migration_one", "sql")?;
    create_test_migration(&config.migrations.directory, "migration_two", "sql")?;

    // Test validation
    let result = migrate::handle_command(
        MigrateCommands::Validate {
            check_gaps: true,
            verify_checksums: true,
        },
        "",
        &config,
        false,
    );

    assert!(result.is_ok(), "Validation should succeed");

    Ok(())
}

#[tokio::test]
async fn test_list_migrations() -> Result<()> {
    let temp_dir = setup_test_env()?;
    let config = Config::default_with_directory(temp_dir.path().join("migrations").to_str().unwrap());

    // Create test migrations
    create_test_migration(&config.migrations.directory, "list_test_one", "sql")?;
    create_test_migration(&config.migrations.directory, "list_test_two", "sql")?;

    // Test list command
    let result = migrate::handle_command(
        MigrateCommands::List {
            pending: false,
            applied: false,
        },
        "",
        &config,
        false,
    );

    assert!(result.is_ok(), "List command should succeed");

    Ok(())
}

#[tokio::test]
async fn test_target_version_migration() -> Result<()> {
    let temp_dir = setup_test_env()?;
    let config = Config::default_with_directory(temp_dir.path().join("migrations").to_str().unwrap());
    let db_path = temp_dir.path().join("test.db");
    let database_url = format!("sqlite:{}", db_path.display());

    // Create multiple migrations
    create_test_migration(&config.migrations.directory, "first_migration", "sql")?;
    std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure different timestamps
    create_test_migration(&config.migrations.directory, "second_migration", "sql")?;
    std::thread::sleep(std::time::Duration::from_millis(10));
    create_test_migration(&config.migrations.directory, "third_migration", "sql")?;

    // Get the first migration version
    let first_version = get_first_migration_version(&config.migrations.directory)?;

    // Run migrations up to the first version only
    let result = migrate::handle_command(
        MigrateCommands::Run {
            database_url: Some(database_url.clone()),
            dry_run: false,
            target: Some(first_version),
        },
        &database_url,
        &config,
        false,
    );

    assert!(result.is_ok(), "Target version migration should succeed");

    Ok(())
}

fn get_first_migration_version(directory: &str) -> Result<i64> {
    let migrations_dir = Path::new(directory);
    let mut versions = Vec::new();

    for entry in fs::read_dir(migrations_dir)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        
        if file_name.ends_with(".up.sql") {
            let parts: Vec<&str> = file_name.splitn(2, '_').collect();
            if let Ok(version) = parts[0].parse::<i64>() {
                versions.push(version);
            }
        }
    }

    versions.sort();
    versions.first().copied()
        .ok_or_else(|| anyhow::anyhow!("No migrations found"))
}