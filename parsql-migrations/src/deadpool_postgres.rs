//! Deadpool PostgreSQL adapter for migrations.

use crate::{
    error::Result,
    traits::AsyncMigrationConnection,
    tokio_postgres::TokioPostgresMigrationConnection,
};
use async_trait::async_trait;
use deadpool_postgres::{Object, Pool};

/// Deadpool PostgreSQL migration pool adapter
pub struct DeadpoolMigrationPool {
    pool: Pool,
}

impl DeadpoolMigrationPool {
    /// Create a new migration pool
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
    
    /// Run migrations using a connection from the pool
    pub async fn run_migrations<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut dyn AsyncMigrationConnection) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<R>> + Send + '_>> + Send,
        R: Send,
    {
        let client = self.pool.get().await?;
        let mut conn = DeadpoolMigrationConnection::new(client);
        f(&mut conn).await
    }
}

/// Deadpool PostgreSQL migration connection adapter
pub struct DeadpoolMigrationConnection {
    client: Object,
}

impl DeadpoolMigrationConnection {
    /// Create a new connection from a pooled object
    pub fn new(client: Object) -> Self {
        Self { client }
    }
}

#[async_trait]
impl AsyncMigrationConnection for DeadpoolMigrationConnection {
    async fn execute(&mut self, sql: &str) -> Result<()> {
        // Delegate to tokio-postgres implementation
        let mut conn = TokioPostgresMigrationConnection::new(&*self.client);
        conn.execute(sql).await
    }
    
    async fn execute_with_result(&mut self, sql: &str) -> Result<u64> {
        let mut conn = TokioPostgresMigrationConnection::new(&*self.client);
        conn.execute_with_result(sql).await
    }
    
    async fn query_one<T>(&mut self, sql: &str) -> Result<T>
    where
        T: crate::traits::FromSql + Send,
    {
        let mut conn = TokioPostgresMigrationConnection::new(&*self.client);
        conn.query_one(sql).await
    }
    
    async fn query<T>(&mut self, sql: &str) -> Result<Vec<T>>
    where
        T: crate::traits::FromSql + Send,
    {
        let mut conn = TokioPostgresMigrationConnection::new(&*self.client);
        conn.query(sql).await
    }
    
    async fn transaction<F, R>(&mut self, f: F) -> Result<R>
    where
        F: for<'a> FnOnce(&'a mut dyn AsyncMigrationConnection) -> 
            std::pin::Pin<Box<dyn std::future::Future<Output = Result<R>> + Send + 'a>> + Send,
        R: Send,
    {
        let mut conn = TokioPostgresMigrationConnection::new(&*self.client);
        conn.transaction(f).await
    }
    
    fn database_type(&self) -> &str {
        "postgresql"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pool_creation() {
        // This is a compile-time test
        fn _test_pool_type(pool: Pool) {
            let _pool = DeadpoolMigrationPool::new(pool);
        }
    }
}