//! Configuration handling for parsql CLI

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub migrations: MigrationConfig,
    
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
    
    #[serde(skip)]
    pub database_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationConfig {
    #[serde(default = "default_migrations_dir")]
    pub directory: String,
    
    #[serde(default = "default_table_name")]
    pub table_name: String,
    
    #[serde(default = "default_true")]
    pub transaction_per_migration: bool,
    
    #[serde(default)]
    pub allow_out_of_order: bool,
    
    #[serde(default = "default_true")]
    pub verify_checksums: bool,
    
    #[serde(default)]
    pub auto_create_table: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            directory: default_migrations_dir(),
            table_name: default_table_name(),
            transaction_per_migration: true,
            allow_out_of_order: false,
            verify_checksums: true,
            auto_create_table: Some(true),
        }
    }
}

fn default_migrations_dir() -> String {
    "migrations".to_string()
}

fn default_table_name() -> String {
    "parsql_migrations".to_string()
}

fn default_true() -> bool {
    true
}

pub fn load_config(path: &str) -> Result<Config> {
    let config_path = Path::new(path);
    
    if !config_path.exists() {
        // Return default config if file doesn't exist
        return Ok(Config::default());
    }
    
    let contents = fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config file: {}", path))?;
    
    let mut config: Config = toml::from_str(&contents)
        .with_context(|| format!("Failed to parse config file: {}", path))?;
    
    // If database URL is in config, use it
    if let Some(ref db_config) = config.database {
        config.database_url = Some(db_config.url.clone());
    }
    
    Ok(config)
}

impl Config {
    pub fn to_parsql_migration_config(&self) -> parsql_migrations::MigrationConfig {
        let mut config = parsql_migrations::MigrationConfig::default();
        
        config.table.table_name = self.migrations.table_name.clone();
        config.transaction_per_migration = self.migrations.transaction_per_migration;
        config.allow_out_of_order = self.migrations.allow_out_of_order;
        config.verify_checksums = self.migrations.verify_checksums;
        
        if let Some(auto_create) = self.migrations.auto_create_table {
            config.auto_create_table = auto_create;
        }
        
        config
    }
}