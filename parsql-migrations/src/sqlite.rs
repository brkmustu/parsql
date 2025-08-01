//! SQLite adapter for migrations.

use crate::{
    error::{MigrationError, Result},
    traits::{FromSql, FromSqlValue, MigrationConnection, SqlRow},
};
use rusqlite::{Connection, Row, Transaction};
use std::any::Any;

/// SQLite migration connection adapter
pub struct SqliteMigrationConnection<'a> {
    connection: &'a mut Connection,
}

impl<'a> SqliteMigrationConnection<'a> {
    /// Create a new SQLite migration connection
    pub fn new(connection: &'a mut Connection) -> Self {
        Self { connection }
    }
}

impl<'a> MigrationConnection for SqliteMigrationConnection<'a> {
    fn execute(&mut self, sql: &str) -> Result<()> {
        self.connection.execute(sql, [])?;
        Ok(())
    }
    
    fn execute_with_result(&mut self, sql: &str) -> Result<u64> {
        let changes = self.connection.execute(sql, [])?;
        Ok(changes as u64)
    }
    
    fn query_one<T>(&mut self, sql: &str) -> Result<T>
    where
        T: FromSql,
    {
        let mut stmt = self.connection.prepare(sql)?;
        let mut rows = stmt.query([])?;
        
        if let Some(row) = rows.next()? {
            T::from_sql_row(&SqliteRowAdapter(row))
        } else {
            Err(MigrationError::Custom("No rows returned".into()))
        }
    }
    
    fn query<T>(&mut self, sql: &str) -> Result<Vec<T>>
    where
        T: FromSql,
    {
        let mut stmt = self.connection.prepare(sql)?;
        let rows = stmt.query_map([], |row| {
            Ok(SqliteRowWrapper { row: row.try_into().unwrap() })
        })?;
        
        let mut results = Vec::new();
        for row in rows {
            let wrapper = row.map_err(|e| MigrationError::DatabaseError(e.to_string()))?;
            results.push(T::from_sql_row(&wrapper)?);
        }
        
        Ok(results)
    }
    
    fn transaction<F, R>(&mut self, f: F) -> Result<R>
    where
        F: FnOnce(&mut dyn MigrationConnection) -> Result<R>,
    {
        let tx = self.connection.transaction()?;
        let mut tx_conn = SqliteTransactionConnection::new(tx);
        
        match f(&mut tx_conn) {
            Ok(result) => {
                tx_conn.transaction.commit()?;
                Ok(result)
            }
            Err(e) => {
                tx_conn.transaction.rollback()?;
                Err(e)
            }
        }
    }
    
    fn database_type(&self) -> &str {
        "sqlite"
    }
}

/// SQLite transaction connection adapter
struct SqliteTransactionConnection<'a> {
    transaction: Transaction<'a>,
}

impl<'a> SqliteTransactionConnection<'a> {
    fn new(transaction: Transaction<'a>) -> Self {
        Self { transaction }
    }
}

impl<'a> MigrationConnection for SqliteTransactionConnection<'a> {
    fn execute(&mut self, sql: &str) -> Result<()> {
        self.transaction.execute(sql, [])?;
        Ok(())
    }
    
    fn execute_with_result(&mut self, sql: &str) -> Result<u64> {
        let changes = self.transaction.execute(sql, [])?;
        Ok(changes as u64)
    }
    
    fn query_one<T>(&mut self, sql: &str) -> Result<T>
    where
        T: FromSql,
    {
        let mut stmt = self.transaction.prepare(sql)?;
        let mut rows = stmt.query([])?;
        
        if let Some(row) = rows.next()? {
            T::from_sql_row(&SqliteRowAdapter(row))
        } else {
            Err(MigrationError::Custom("No rows returned".into()))
        }
    }
    
    fn query<T>(&mut self, sql: &str) -> Result<Vec<T>>
    where
        T: FromSql,
    {
        let mut stmt = self.transaction.prepare(sql)?;
        let rows = stmt.query_map([], |row| {
            Ok(SqliteRowWrapper { row: row.try_into().unwrap() })
        })?;
        
        let mut results = Vec::new();
        for row in rows {
            let wrapper = row.map_err(|e| MigrationError::DatabaseError(e.to_string()))?;
            results.push(T::from_sql_row(&wrapper)?);
        }
        
        Ok(results)
    }
    
    fn transaction<F, R>(&mut self, f: F) -> Result<R>
    where
        F: FnOnce(&mut dyn MigrationConnection) -> Result<R>,
    {
        // SQLite doesn't support nested transactions
        // We'll use savepoints instead
        self.transaction.execute("SAVEPOINT migration_savepoint", [])?;
        
        match f(self) {
            Ok(result) => {
                self.transaction.execute("RELEASE SAVEPOINT migration_savepoint", [])?;
                Ok(result)
            }
            Err(e) => {
                self.transaction.execute("ROLLBACK TO SAVEPOINT migration_savepoint", [])?;
                Err(e)
            }
        }
    }
    
    fn database_type(&self) -> &str {
        "sqlite"
    }
}

/// Row adapter for SQLite
struct SqliteRowAdapter<'a>(&'a Row<'a>);

impl<'a> SqlRow for SqliteRowAdapter<'a> {
    fn get<T>(&self, idx: usize) -> Result<T>
    where
        T: FromSqlValue,
    {
        // This is a simplified implementation
        // In a real implementation, we'd need to handle all SQLite types
        if let Ok(value) = self.0.get::<_, i64>(idx) {
            if let Ok(result) = T::from_sql_value(&value as &dyn Any) {
                return Ok(result);
            }
        }
        
        if let Ok(value) = self.0.get::<_, String>(idx) {
            if let Ok(result) = T::from_sql_value(&value as &dyn Any) {
                return Ok(result);
            }
        }
        
        if let Ok(value) = self.0.get::<_, bool>(idx) {
            if let Ok(result) = T::from_sql_value(&value as &dyn Any) {
                return Ok(result);
            }
        }
        
        if let Ok(value) = self.0.get::<_, Option<String>>(idx) {
            if let Ok(result) = T::from_sql_value(&value as &dyn Any) {
                return Ok(result);
            }
        }
        
        if let Ok(value) = self.0.get::<_, Option<i64>>(idx) {
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
        // SQLite doesn't provide direct column name to index mapping in Row
        // We'd need to store this information separately or use a different approach
        Err(MigrationError::Custom("get_by_name not implemented for SQLite".into()))
    }
}

/// Wrapper for owned row data
struct SqliteRowWrapper {
    row: Vec<rusqlite::types::Value>,
}

impl SqlRow for SqliteRowWrapper {
    fn get<T>(&self, idx: usize) -> Result<T>
    where
        T: FromSqlValue,
    {
        let value = self.row.get(idx)
            .ok_or_else(|| MigrationError::Custom(format!("Column index {} out of bounds", idx)))?;
        
        // Convert SQLite Value to appropriate type
        match value {
            rusqlite::types::Value::Integer(i) => {
                if let Ok(result) = T::from_sql_value(i as &dyn Any) {
                    return Ok(result);
                }
            }
            rusqlite::types::Value::Text(s) => {
                if let Ok(result) = T::from_sql_value(s as &dyn Any) {
                    return Ok(result);
                }
                // Try to parse as DateTime for SQLite text timestamps
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                    let utc_dt = dt.with_timezone(&chrono::Utc);
                    if let Ok(result) = T::from_sql_value(&utc_dt as &dyn Any) {
                        return Ok(result);
                    }
                }
            }
            rusqlite::types::Value::Null => {
                if let Ok(result) = T::from_sql_value(&None::<String> as &dyn Any) {
                    return Ok(result);
                }
                if let Ok(result) = T::from_sql_value(&None::<i64> as &dyn Any) {
                    return Ok(result);
                }
            }
            _ => {}
        }
        
        Err(MigrationError::Custom(format!("Failed to convert value at index {}", idx)))
    }
    
    fn get_by_name<T>(&self, _name: &str) -> Result<T>
    where
        T: FromSqlValue,
    {
        Err(MigrationError::Custom("get_by_name not implemented for SQLite".into()))
    }
}

// SQLite-specific FromSqlValue implementation for DateTime
impl FromSqlValue for chrono::DateTime<chrono::Utc> {
    fn from_sql_value(value: &dyn Any) -> Result<Self> {
        // First try direct conversion
        if let Some(dt) = value.downcast_ref::<chrono::DateTime<chrono::Utc>>() {
            return Ok(*dt);
        }
        
        // Then try string conversion (SQLite stores timestamps as text)
        if let Some(s) = value.downcast_ref::<String>() {
            // Try parsing as RFC3339
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                return Ok(dt.with_timezone(&chrono::Utc));
            }
            
            // Try parsing as SQLite default format
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
                return Ok(chrono::DateTime::from_naive_utc_and_offset(dt, chrono::Utc));
            }
        }
        
        Err(MigrationError::Custom("Failed to convert to DateTime<Utc>".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_connection_type() {
        // This is a compile-time test to ensure the types are correct
        fn _test_connection_type(conn: &mut Connection) {
            let _conn = SqliteMigrationConnection::new(conn);
        }
    }
}