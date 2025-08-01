//! Simple PostgreSQL adapter for the migration system.

use crate::{
    error::{MigrationError, Result},
    traits_simple::{MigrationConnection, MigrationRecord},
};
use postgres::Client;

/// PostgreSQL connection wrapper for migrations
pub struct PostgresMigrationConnection<'a> {
    client: &'a mut Client,
}

impl<'a> PostgresMigrationConnection<'a> {
    /// Create a new PostgreSQL migration connection
    pub fn new(client: &'a mut Client) -> Self {
        Self { client }
    }
}

impl<'a> MigrationConnection for PostgresMigrationConnection<'a> {
    fn execute(&mut self, sql: &str) -> Result<()> {
        self.client.execute(sql, &[])
            .map_err(|e| MigrationError::database(e.to_string()))?;
        Ok(())
    }
    
    fn execute_with_result(&mut self, sql: &str) -> Result<u64> {
        let count = self.client.execute(sql, &[])
            .map_err(|e| MigrationError::database(e.to_string()))?;
        Ok(count)
    }
    
    fn database_type(&self) -> &str {
        "postgresql"
    }
    
    fn query_migrations(&mut self, table_name: &str) -> Result<Vec<MigrationRecord>> {
        let sql = format!(
            "SELECT version, name, applied_at, checksum, execution_time_ms 
             FROM {} 
             ORDER BY version",
            table_name
        );
        
        let rows = self.client.query(&sql, &[])
            .map_err(|e| MigrationError::database(e.to_string()))?;
        
        let migrations = rows.into_iter()
            .map(|row| {
                // PostgreSQL TIMESTAMPTZ can be read as SystemTime
                let applied_at: std::time::SystemTime = row.get(2);
                let applied_at = chrono::DateTime::<chrono::Utc>::from(applied_at);
                
                MigrationRecord {
                    version: row.get(0),
                    name: row.get(1),
                    applied_at,
                    checksum: row.get(3),
                    execution_time_ms: row.get(4),
                }
            })
            .collect();
        
        Ok(migrations)
    }
    
    fn begin_transaction(&mut self) -> Result<()> {
        self.execute("BEGIN")
    }
    
    fn commit_transaction(&mut self) -> Result<()> {
        self.execute("COMMIT")
    }
    
    fn rollback_transaction(&mut self) -> Result<()> {
        self.execute("ROLLBACK")
    }
}

/// Extension trait for postgres::Client
pub trait PostgresConnectionExt {
    /// Create a migration connection from this PostgreSQL client
    fn migration_connection(&mut self) -> PostgresMigrationConnection;
}

impl PostgresConnectionExt for Client {
    fn migration_connection(&mut self) -> PostgresMigrationConnection {
        PostgresMigrationConnection::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_postgres_connection_type() {
        // This is a compile-time test to ensure the types are correct
        fn _test_connection_type(client: &mut Client) {
            let _conn = PostgresMigrationConnection::new(client);
        }
    }
}