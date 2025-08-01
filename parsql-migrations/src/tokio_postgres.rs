//! Tokio PostgreSQL adapter for migrations.

use crate::{
    error::{MigrationError, Result},
    traits::{AsyncMigrationConnection, FromSql, FromSqlValue, SqlRow},
};
use async_trait::async_trait;
use tokio_postgres::{Client, Row, Transaction};
use std::any::Any;

/// Tokio PostgreSQL migration connection adapter
pub struct TokioPostgresMigrationConnection<'a> {
    client: &'a Client,
}

impl<'a> TokioPostgresMigrationConnection<'a> {
    /// Create a new Tokio PostgreSQL migration connection
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<'a> AsyncMigrationConnection for TokioPostgresMigrationConnection<'a> {
    async fn execute(&mut self, sql: &str) -> Result<()> {
        self.client.execute(sql, &[]).await?;
        Ok(())
    }
    
    async fn execute_with_result(&mut self, sql: &str) -> Result<u64> {
        let rows = self.client.execute(sql, &[]).await?;
        Ok(rows)
    }
    
    async fn query_one<T>(&mut self, sql: &str) -> Result<T>
    where
        T: FromSql + Send,
    {
        let row = self.client.query_one(sql, &[]).await?;
        T::from_sql_row(&TokioPostgresRowAdapter(&row))
    }
    
    async fn query<T>(&mut self, sql: &str) -> Result<Vec<T>>
    where
        T: FromSql + Send,
    {
        let rows = self.client.query(sql, &[]).await?;
        rows.iter()
            .map(|row| T::from_sql_row(&TokioPostgresRowAdapter(row)))
            .collect()
    }
    
    async fn transaction<F, R>(&mut self, f: F) -> Result<R>
    where
        F: for<'b> FnOnce(&'b mut dyn AsyncMigrationConnection) -> 
            std::pin::Pin<Box<dyn std::future::Future<Output = Result<R>> + Send + 'b>> + Send,
        R: Send,
    {
        let transaction = self.client.transaction().await?;
        let mut tx_conn = TokioPostgresTransactionConnection { transaction };
        
        match f(&mut tx_conn).await {
            Ok(result) => {
                tx_conn.transaction.commit().await?;
                Ok(result)
            }
            Err(e) => {
                tx_conn.transaction.rollback().await?;
                Err(e)
            }
        }
    }
    
    fn database_type(&self) -> &str {
        "postgresql"
    }
}

/// Tokio PostgreSQL transaction connection adapter
struct TokioPostgresTransactionConnection<'a> {
    transaction: Transaction<'a>,
}

#[async_trait]
impl<'a> AsyncMigrationConnection for TokioPostgresTransactionConnection<'a> {
    async fn execute(&mut self, sql: &str) -> Result<()> {
        self.transaction.execute(sql, &[]).await?;
        Ok(())
    }
    
    async fn execute_with_result(&mut self, sql: &str) -> Result<u64> {
        let rows = self.transaction.execute(sql, &[]).await?;
        Ok(rows)
    }
    
    async fn query_one<T>(&mut self, sql: &str) -> Result<T>
    where
        T: FromSql + Send,
    {
        let row = self.transaction.query_one(sql, &[]).await?;
        T::from_sql_row(&TokioPostgresRowAdapter(&row))
    }
    
    async fn query<T>(&mut self, sql: &str) -> Result<Vec<T>>
    where
        T: FromSql + Send,
    {
        let rows = self.transaction.query(sql, &[]).await?;
        rows.iter()
            .map(|row| T::from_sql_row(&TokioPostgresRowAdapter(row)))
            .collect()
    }
    
    async fn transaction<F, R>(&mut self, f: F) -> Result<R>
    where
        F: for<'b> FnOnce(&'b mut dyn AsyncMigrationConnection) -> 
            std::pin::Pin<Box<dyn std::future::Future<Output = Result<R>> + Send + 'b>> + Send,
        R: Send,
    {
        // Use savepoints for nested transactions
        self.transaction.execute("SAVEPOINT migration_savepoint", &[]).await?;
        
        match f(self).await {
            Ok(result) => {
                self.transaction.execute("RELEASE SAVEPOINT migration_savepoint", &[]).await?;
                Ok(result)
            }
            Err(e) => {
                self.transaction.execute("ROLLBACK TO SAVEPOINT migration_savepoint", &[]).await?;
                Err(e)
            }
        }
    }
    
    fn database_type(&self) -> &str {
        "postgresql"
    }
}

/// Row adapter for Tokio PostgreSQL
struct TokioPostgresRowAdapter<'a>(&'a Row);

impl<'a> SqlRow for TokioPostgresRowAdapter<'a> {
    fn get<T>(&self, idx: usize) -> Result<T>
    where
        T: FromSqlValue,
    {
        // This is a simplified implementation
        // In a real implementation, we'd need to handle all PostgreSQL types
        if let Ok(value) = self.0.try_get::<_, i64>(idx) {
            if let Ok(result) = T::from_sql_value(&value as &dyn Any) {
                return Ok(result);
            }
        }
        
        if let Ok(value) = self.0.try_get::<_, String>(idx) {
            if let Ok(result) = T::from_sql_value(&value as &dyn Any) {
                return Ok(result);
            }
        }
        
        if let Ok(value) = self.0.try_get::<_, bool>(idx) {
            if let Ok(result) = T::from_sql_value(&value as &dyn Any) {
                return Ok(result);
            }
        }
        
        if let Ok(value) = self.0.try_get::<_, Option<String>>(idx) {
            if let Ok(result) = T::from_sql_value(&value as &dyn Any) {
                return Ok(result);
            }
        }
        
        if let Ok(value) = self.0.try_get::<_, chrono::DateTime<chrono::Utc>>(idx) {
            if let Ok(result) = T::from_sql_value(&value as &dyn Any) {
                return Ok(result);
            }
        }
        
        if let Ok(value) = self.0.try_get::<_, Option<i64>>(idx) {
            if let Ok(result) = T::from_sql_value(&value as &dyn Any) {
                return Ok(result);
            }
        }
        
        Err(MigrationError::Custom(format!("Failed to get value at index {}", idx)))
    }
    
    fn get_by_name<T>(&self, name: &str) -> Result<T>
    where
        T: FromSqlValue,
    {
        // Find column index by name
        for (idx, column) in self.0.columns().iter().enumerate() {
            if column.name() == name {
                return self.get(idx);
            }
        }
        
        Err(MigrationError::Custom(format!("Column '{}' not found", name)))
    }
}

/// Async migration runner for Tokio PostgreSQL
pub struct AsyncMigrationRunner {
    migrations: Vec<Box<dyn crate::traits::AsyncMigration>>,
    config: crate::config::MigrationConfig,
}

impl AsyncMigrationRunner {
    /// Create a new async migration runner
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
            config: crate::config::MigrationConfig::default(),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(config: crate::config::MigrationConfig) -> Self {
        Self {
            migrations: Vec::new(),
            config,
        }
    }
    
    /// Add an async migration
    pub fn add_migration(&mut self, migration: Box<dyn crate::traits::AsyncMigration>) -> &mut Self {
        self.migrations.push(migration);
        self
    }
    
    /// Run all pending migrations
    pub async fn run(&mut self, conn: &mut dyn AsyncMigrationConnection) -> Result<crate::types::MigrationReport> {
        let mut report = crate::types::MigrationReport::new();
        
        // Ensure migrations table exists
        if self.config.auto_create_table {
            self.ensure_migration_table(conn).await?;
        }
        
        // Sort migrations by version
        self.migrations.sort_by_key(|m| m.version());
        
        // Get applied migrations
        let applied = self.get_applied_migrations(conn).await?;
        
        // Execute migrations
        for migration in &self.migrations {
            let version = migration.version();
            
            if applied.contains_key(&version) {
                report.add_skipped(version);
                continue;
            }
            
            // Execute the migration
            let start = std::time::Instant::now();
            
            println!("Executing migration {}: {}", version, migration.name());
            
            let result = if self.config.transaction_per_migration {
                // Run in transaction
                conn.transaction(|tx| {
                    Box::pin(async move {
                        migration.up(tx).await?;
                        self.record_migration(tx, migration.as_ref(), start.elapsed().as_millis() as i64).await?;
                        Ok(())
                    })
                }).await
            } else {
                // Run without transaction
                migration.up(conn).await?;
                self.record_migration(conn, migration.as_ref(), start.elapsed().as_millis() as i64).await?;
                Ok(())
            };
            
            let execution_time = start.elapsed().as_millis() as i64;
            
            match result {
                Ok(()) => {
                    report.add_success(crate::types::MigrationResult::success(
                        version,
                        migration.name().to_string(),
                        execution_time,
                    ));
                    println!("  ✓ Migration {} completed in {}ms", version, execution_time);
                }
                Err(e) => {
                    report.add_failure(crate::types::MigrationResult::failure(
                        version,
                        migration.name().to_string(),
                        e.to_string(),
                        execution_time,
                    ));
                    println!("  ✗ Migration {} failed: {}", version, e);
                    
                    if self.config.stop_on_error {
                        report.complete();
                        return Ok(report);
                    }
                }
            }
        }
        
        report.complete();
        Ok(report)
    }
    
    async fn ensure_migration_table(&self, conn: &mut dyn AsyncMigrationConnection) -> Result<()> {
        let sql = match conn.database_type() {
            "postgresql" | "postgres" => self.config.postgres_create_table_sql(),
            db => return Err(MigrationError::Custom(format!("Unsupported database type: {}", db))),
        };
        
        conn.execute(&sql).await?;
        Ok(())
    }
    
    async fn get_applied_migrations(&self, conn: &mut dyn AsyncMigrationConnection) -> Result<crate::types::MigrationMap> {
        let sql = format!(
            "SELECT {}, {}, {}, {}, {} FROM {} ORDER BY {}",
            self.config.table.version_column,
            self.config.table.name_column,
            self.config.table.applied_at_column,
            self.config.table.checksum_column,
            self.config.table.execution_time_column,
            self.config.table.table_name,
            self.config.table.version_column
        );
        
        struct MigrationRow {
            version: i64,
            name: String,
            applied_at: chrono::DateTime<chrono::Utc>,
            checksum: Option<String>,
            execution_time_ms: Option<i64>,
        }
        
        impl FromSql for MigrationRow {
            fn from_sql_row(row: &dyn SqlRow) -> Result<Self> {
                Ok(Self {
                    version: row.get(0)?,
                    name: row.get(1)?,
                    applied_at: row.get(2)?,
                    checksum: row.get(3)?,
                    execution_time_ms: row.get(4)?,
                })
            }
        }
        
        let rows: Vec<MigrationRow> = conn.query(&sql).await?;
        
        let mut map = crate::types::MigrationMap::new();
        for row in rows {
            let mut details = crate::types::MigrationDetails::new(row.version, row.name);
            details.state = crate::types::MigrationState::Applied;
            details.applied_at = Some(row.applied_at);
            details.checksum = row.checksum;
            details.execution_time_ms = row.execution_time_ms;
            
            map.insert(row.version, details);
        }
        
        Ok(map)
    }
    
    async fn record_migration(
        &self,
        conn: &mut dyn AsyncMigrationConnection,
        migration: &dyn crate::traits::AsyncMigration,
        execution_time_ms: i64,
    ) -> Result<()> {
        let sql = format!(
            "INSERT INTO {} ({}, {}, {}, {}) VALUES ({}, '{}', '{}', {})",
            self.config.table.table_name,
            self.config.table.version_column,
            self.config.table.name_column,
            self.config.table.checksum_column,
            self.config.table.execution_time_column,
            migration.version(),
            migration.name().replace('\'', "''"),
            migration.checksum(),
            execution_time_ms
        );
        
        conn.execute(&sql).await?;
        Ok(())
    }
}

impl Default for AsyncMigrationRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_connection_type() {
        // This is a compile-time test to ensure the types are correct
        fn _test_connection_type(client: &Client) {
            let _conn = TokioPostgresMigrationConnection::new(client);
        }
    }
}