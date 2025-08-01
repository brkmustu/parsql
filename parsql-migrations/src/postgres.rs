//! PostgreSQL adapter for migrations.

use crate::{
    error::{MigrationError, Result},
    traits::{FromSql, FromSqlValue, MigrationConnection, SqlRow},
};
use postgres::{Client, Row, Transaction};
use std::any::Any;

/// PostgreSQL migration connection adapter
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
        self.client.execute(sql, &[])?;
        Ok(())
    }
    
    fn execute_with_result(&mut self, sql: &str) -> Result<u64> {
        let rows = self.client.execute(sql, &[])?;
        Ok(rows)
    }
    
    fn query_one<T>(&mut self, sql: &str) -> Result<T>
    where
        T: FromSql,
    {
        let row = self.client.query_one(sql, &[])?;
        T::from_sql_row(&PostgresRowAdapter(&row))
    }
    
    fn query<T>(&mut self, sql: &str) -> Result<Vec<T>>
    where
        T: FromSql,
    {
        let rows = self.client.query(sql, &[])?;
        rows.iter()
            .map(|row| T::from_sql_row(&PostgresRowAdapter(row)))
            .collect()
    }
    
    fn transaction<F, R>(&mut self, f: F) -> Result<R>
    where
        F: FnOnce(&mut dyn MigrationConnection) -> Result<R>,
    {
        let mut transaction = self.client.transaction()?;
        let mut tx_conn = PostgresTransactionConnection::new(&mut transaction);
        
        match f(&mut tx_conn) {
            Ok(result) => {
                transaction.commit()?;
                Ok(result)
            }
            Err(e) => {
                transaction.rollback()?;
                Err(e)
            }
        }
    }
    
    fn database_type(&self) -> &str {
        "postgresql"
    }
}

/// PostgreSQL transaction connection adapter
struct PostgresTransactionConnection<'a> {
    transaction: &'a mut Transaction<'a>,
}

impl<'a> PostgresTransactionConnection<'a> {
    fn new(transaction: &'a mut Transaction<'a>) -> Self {
        Self { transaction }
    }
}

impl<'a> MigrationConnection for PostgresTransactionConnection<'a> {
    fn execute(&mut self, sql: &str) -> Result<()> {
        self.transaction.execute(sql, &[])?;
        Ok(())
    }
    
    fn execute_with_result(&mut self, sql: &str) -> Result<u64> {
        let rows = self.transaction.execute(sql, &[])?;
        Ok(rows)
    }
    
    fn query_one<T>(&mut self, sql: &str) -> Result<T>
    where
        T: FromSql,
    {
        let row = self.transaction.query_one(sql, &[])?;
        T::from_sql_row(&PostgresRowAdapter(&row))
    }
    
    fn query<T>(&mut self, sql: &str) -> Result<Vec<T>>
    where
        T: FromSql,
    {
        let rows = self.transaction.query(sql, &[])?;
        rows.iter()
            .map(|row| T::from_sql_row(&PostgresRowAdapter(row)))
            .collect()
    }
    
    fn transaction<F, R>(&mut self, f: F) -> Result<R>
    where
        F: FnOnce(&mut dyn MigrationConnection) -> Result<R>,
    {
        // PostgreSQL doesn't support nested transactions in the same way
        // We'll use savepoints instead
        self.transaction.execute("SAVEPOINT migration_savepoint", &[])?;
        
        match f(self) {
            Ok(result) => {
                self.transaction.execute("RELEASE SAVEPOINT migration_savepoint", &[])?;
                Ok(result)
            }
            Err(e) => {
                self.transaction.execute("ROLLBACK TO SAVEPOINT migration_savepoint", &[])?;
                Err(e)
            }
        }
    }
    
    fn database_type(&self) -> &str {
        "postgresql"
    }
}

/// Row adapter for PostgreSQL
struct PostgresRowAdapter<'a>(&'a Row);

impl<'a> SqlRow for PostgresRowAdapter<'a> {
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

// Additional FromSqlValue implementations for PostgreSQL types
impl FromSqlValue for chrono::DateTime<chrono::Utc> {
    fn from_sql_value(value: &dyn Any) -> Result<Self> {
        value.downcast_ref::<chrono::DateTime<chrono::Utc>>()
            .copied()
            .ok_or_else(|| MigrationError::Custom("Failed to convert to DateTime<Utc>".into()))
    }
}

impl FromSqlValue for Option<i64> {
    fn from_sql_value(value: &dyn Any) -> Result<Self> {
        if let Some(v) = value.downcast_ref::<i64>() {
            Ok(Some(*v))
        } else if let Some(v) = value.downcast_ref::<Option<i64>>() {
            Ok(*v)
        } else if value.downcast_ref::<()>().is_some() {
            Ok(None)
        } else {
            Err(MigrationError::Custom("Failed to convert to Option<i64>".into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_connection_type() {
        // This is a compile-time test to ensure the types are correct
        fn _test_connection_type(client: &mut Client) {
            let _conn = PostgresMigrationConnection::new(client);
        }
    }
}