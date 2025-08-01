//! Simple SQLite adapter for the migration system.

use crate::{
    error::{MigrationError, Result},
    traits_simple::{MigrationConnection, MigrationRecord},
};
use rusqlite::Connection;

/// SQLite connection wrapper for migrations
pub struct SqliteMigrationConnection<'a> {
    conn: &'a mut Connection,
}

impl<'a> SqliteMigrationConnection<'a> {
    /// Create a new SQLite migration connection
    pub fn new(conn: &'a mut Connection) -> Self {
        Self { conn }
    }
}

impl<'a> MigrationConnection for SqliteMigrationConnection<'a> {
    fn execute(&mut self, sql: &str) -> Result<()> {
        self.conn.execute(sql, [])
            .map_err(|e| MigrationError::database(e.to_string()))?;
        Ok(())
    }
    
    fn execute_with_result(&mut self, sql: &str) -> Result<u64> {
        let count = self.conn.execute(sql, [])
            .map_err(|e| MigrationError::database(e.to_string()))?;
        Ok(count as u64)
    }
    
    fn database_type(&self) -> &str {
        "sqlite"
    }
    
    fn query_migrations(&mut self, table_name: &str) -> Result<Vec<MigrationRecord>> {
        let sql = format!(
            "SELECT version, name, applied_at, checksum, execution_time_ms 
             FROM {} 
             ORDER BY version",
            table_name
        );
        
        let mut stmt = self.conn.prepare(&sql)
            .map_err(|e| MigrationError::database(e.to_string()))?;
        
        let migrations = stmt.query_map([], |row| {
            let applied_at_str: String = row.get(2)?;
            let applied_at = chrono::DateTime::parse_from_rfc3339(&applied_at_str)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .with_timezone(&chrono::Utc);
            
            Ok(MigrationRecord {
                version: row.get(0)?,
                name: row.get(1)?,
                applied_at,
                checksum: row.get(3)?,
                execution_time_ms: row.get(4)?,
            })
        })
        .map_err(|e| MigrationError::database(e.to_string()))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| MigrationError::database(e.to_string()))?;
        
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

/// Extension trait for rusqlite::Connection
pub trait SqliteConnectionExt {
    /// Create a migration connection from this SQLite connection
    fn migration_connection(&mut self) -> SqliteMigrationConnection;
}

impl SqliteConnectionExt for Connection {
    fn migration_connection(&mut self) -> SqliteMigrationConnection {
        SqliteMigrationConnection::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    
    #[test]
    fn test_sqlite_connection() {
        let mut conn = Connection::open_in_memory().unwrap();
        let mut migration_conn = conn.migration_connection();
        
        // Test database type
        assert_eq!(migration_conn.database_type(), "sqlite");
        
        // Test execute
        migration_conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY)").unwrap();
        
        // Test execute_with_result
        let count = migration_conn.execute_with_result("INSERT INTO test DEFAULT VALUES").unwrap();
        assert_eq!(count, 1);
    }
}