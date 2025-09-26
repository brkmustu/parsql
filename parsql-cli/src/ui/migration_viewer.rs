//! Migration file viewing and editing utilities

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use anyhow::{Context, Result};
use super::output_stream::OutputStreamWidget;

pub struct MigrationViewer {
    migrations_dir: PathBuf,
}

impl MigrationViewer {
    pub fn new(migrations_dir: PathBuf) -> Self {
        Self { migrations_dir }
    }
    
    /// View the contents of a migration file
    pub fn view_migration(
        &self,
        version: i64,
        file_type: MigrationFileType,
        output: &mut OutputStreamWidget,
    ) -> Result<String> {
        let file_path = self.find_migration_file(version, file_type)?;
        
        output.add_info(format!("Reading migration file: {}", file_path.display()));
        
        let content = fs::read_to_string(&file_path)
            .context(format!("Failed to read migration file: {}", file_path.display()))?;
        
        Ok(content)
    }
    
    /// Open a migration file in the user's editor
    pub fn edit_migration(
        &self,
        version: i64,
        file_type: MigrationFileType,
        output: &mut OutputStreamWidget,
    ) -> Result<()> {
        let file_path = self.find_migration_file(version, file_type)?;
        
        // Get editor from environment or use default
        let editor = std::env::var("EDITOR")
            .or_else(|_| std::env::var("VISUAL"))
            .unwrap_or_else(|_| {
                if cfg!(windows) {
                    "notepad".to_string()
                } else {
                    "vi".to_string()
                }
            });
        
        output.add_info(format!("Opening {} in {}", file_path.display(), editor));
        
        // Launch editor
        let status = Command::new(&editor)
            .arg(&file_path)
            .status()
            .context(format!("Failed to launch editor: {}", editor))?;
        
        if status.success() {
            output.add_success(format!("Editor closed successfully"));
            
            // Verify the file still exists and is valid
            if file_path.exists() {
                let content = fs::read_to_string(&file_path)?;
                if content.trim().is_empty() {
                    output.add_warning("Migration file is empty after editing".to_string());
                } else {
                    output.add_info(format!("Migration file saved: {} bytes", content.len()));
                }
            } else {
                output.add_error("Migration file was deleted!".to_string());
            }
        } else {
            output.add_error(format!("Editor exited with error code: {:?}", status.code()));
        }
        
        Ok(())
    }
    
    /// Create and open a new migration file in editor
    pub fn create_and_edit_migration(
        &self,
        _version: i64,
        name: &str,
        output: &mut OutputStreamWidget,
    ) -> Result<()> {
        // First create the migration files
        let migrations_dir = self.migrations_dir.clone();
        let creator = super::migration_creator::MigrationCreator::new(migrations_dir);
        let files = creator.create_migration(name, "sql")?;
        
        output.add_success(format!("Created migration files for version {}", files.version));
        
        // Then open the up file in editor
        let version_num = files.version.parse::<i64>()
            .context("Invalid version number")?;
        
        self.edit_migration(version_num, MigrationFileType::Up, output)?;
        
        Ok(())
    }
    
    /// Find a migration file by version and type
    fn find_migration_file(&self, version: i64, file_type: MigrationFileType) -> Result<PathBuf> {
        let suffix = match file_type {
            MigrationFileType::Up => ".up.sql",
            MigrationFileType::Down => ".down.sql",
        };
        
        // List all files in migrations directory
        let entries = fs::read_dir(&self.migrations_dir)
            .context("Failed to read migrations directory")?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                // Check if filename starts with version and ends with suffix
                if filename.starts_with(&version.to_string()) && filename.ends_with(suffix) {
                    return Ok(path);
                }
            }
        }
        
        anyhow::bail!(
            "Migration file not found for version {} ({})",
            version,
            match file_type {
                MigrationFileType::Up => "up",
                MigrationFileType::Down => "down",
            }
        )
    }
    
    /// List all migration files with their sizes
    pub fn list_migration_files(&self) -> Result<Vec<MigrationFileInfo>> {
        let mut files = Vec::new();
        
        if !self.migrations_dir.exists() {
            return Ok(files);
        }
        
        let entries = fs::read_dir(&self.migrations_dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                if filename.ends_with(".up.sql") || filename.ends_with(".down.sql") {
                    let metadata = fs::metadata(&path)?;
                    let size = metadata.len();
                    
                    // Parse version from filename
                    if let Some(underscore_pos) = filename.find('_') {
                        if let Ok(version) = filename[..underscore_pos].parse::<i64>() {
                            let is_up = filename.ends_with(".up.sql");
                            files.push(MigrationFileInfo {
                                version,
                                filename: filename.to_string(),
                                path: path.clone(),
                                size,
                                is_up,
                            });
                        }
                    }
                }
            }
        }
        
        files.sort_by_key(|f| (f.version, !f.is_up));
        Ok(files)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MigrationFileType {
    Up,
    Down,
}

#[derive(Debug, Clone)]
pub struct MigrationFileInfo {
    pub version: i64,
    pub filename: String,
    pub path: PathBuf,
    pub size: u64,
    pub is_up: bool,
}