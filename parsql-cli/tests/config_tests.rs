//! Tests for configuration loading and validation

use parsql_cli::config::{Config, DatabaseConfig, MigrationConfig, load_config};
use tempfile::TempDir;
use std::fs;

#[test]
fn test_default_config_creation() {
    let config = Config::default();
    
    assert!(config.database.is_none());
    assert!(config.database_url.is_none());
    assert_eq!(config.migrations.directory, "migrations");
    assert_eq!(config.migrations.table_name, "parsql_migrations");
    assert!(config.migrations.verify_checksums);
    assert!(!config.migrations.allow_out_of_order);
    assert!(config.migrations.transaction_per_migration);
}

#[test]
fn test_config_from_toml() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("parsql.toml");
    
    let toml_content = r#"
[database]
url = "postgresql://localhost/test"

[migrations]
directory = "custom_migrations"
table_name = "my_migrations"
verify_checksums = false
allow_out_of_order = true
transaction_per_migration = false
"#;
    
    fs::write(&config_path, toml_content).unwrap();
    
    let config = load_config(config_path.to_str().unwrap()).unwrap();
    
    assert_eq!(config.database_url.unwrap(), "postgresql://localhost/test");
    assert_eq!(config.migrations.directory, "custom_migrations");
    assert_eq!(config.migrations.table_name, "my_migrations");
    assert!(!config.migrations.verify_checksums);
    assert!(config.migrations.allow_out_of_order);
    assert!(!config.migrations.transaction_per_migration);
}

#[test]
fn test_config_file_not_found_returns_default() {
    let config = load_config("non_existent_config.toml").unwrap();
    
    // Should return default config when file doesn't exist
    assert!(config.database.is_none());
    assert_eq!(config.migrations.directory, "migrations");
}

#[test]
fn test_invalid_toml_content() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.toml");
    
    let invalid_toml = r#"
[database
url = "postgresql://localhost/test"
"#;
    
    fs::write(&config_path, invalid_toml).unwrap();
    
    let result = load_config(config_path.to_str().unwrap());
    assert!(result.is_err());
}

#[test]
fn test_migration_config_defaults() {
    let config = MigrationConfig::default();
    
    assert_eq!(config.directory, "migrations");
    assert_eq!(config.table_name, "parsql_migrations");
    assert!(config.transaction_per_migration);
    assert!(!config.allow_out_of_order);
    assert!(config.verify_checksums);
    assert_eq!(config.auto_create_table, Some(true));
}

#[test]
fn test_config_to_parsql_migration_config() {
    let mut config = Config::default();
    config.migrations.table_name = "custom_table".to_string();
    config.migrations.transaction_per_migration = false;
    config.migrations.allow_out_of_order = true;
    config.migrations.verify_checksums = false;
    config.migrations.auto_create_table = Some(false);
    
    let parsql_config = config.to_parsql_migration_config();
    
    assert_eq!(parsql_config.table.table_name, "custom_table");
    assert!(!parsql_config.transaction_per_migration);
    assert!(parsql_config.allow_out_of_order);
    assert!(!parsql_config.verify_checksums);
    assert!(!parsql_config.auto_create_table);
}

#[test]
fn test_empty_config_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("empty.toml");
    
    fs::write(&config_path, "").unwrap();
    
    let config = load_config(config_path.to_str().unwrap()).unwrap();
    
    // Should use defaults for empty file
    assert_eq!(config.migrations.directory, "migrations");
    assert_eq!(config.migrations.table_name, "parsql_migrations");
}

#[test]
fn test_partial_config_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("partial.toml");
    
    let toml_content = r#"
[migrations]
directory = "custom_dir"
# other fields should use defaults
"#;
    
    fs::write(&config_path, toml_content).unwrap();
    
    let config = load_config(config_path.to_str().unwrap()).unwrap();
    
    assert_eq!(config.migrations.directory, "custom_dir");
    assert_eq!(config.migrations.table_name, "parsql_migrations"); // default
    assert!(config.migrations.verify_checksums); // default
}