# parsql-migrations

A simple, type-safe database migration system for Rust that works with PostgreSQL and SQLite.

## Features

- **Multi-database support**: PostgreSQL (sync/async) and SQLite
- **Transaction safety**: Each migration runs in its own transaction (configurable)
- **Checksum verification**: Detect modified migrations
- **Gap detection**: Prevent missing migrations in sequence
- **Simple trait-based API**: Easy to implement custom migrations
- **CLI tool**: Full-featured command-line interface

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
parsql-migrations = { version = "0.5", features = ["postgres", "sqlite"] }
```

## Quick Start

### 1. Define a Migration

```rust
use parsql_migrations::prelude::*;

pub struct CreateUsersTable;

impl Migration for CreateUsersTable {
    fn version(&self) -> i64 {
        20240101120000
    }
    
    fn name(&self) -> &str {
        "create_users_table"
    }
    
    fn up(&self, conn: &mut dyn MigrationConnection) -> Result<()> {
        conn.execute(
            "CREATE TABLE users (
                id SERIAL PRIMARY KEY,
                email VARCHAR(255) NOT NULL UNIQUE,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )"
        )
    }
    
    fn down(&self, conn: &mut dyn MigrationConnection) -> Result<()> {
        conn.execute("DROP TABLE IF EXISTS users")
    }
}
```

### 2. Run Migrations

#### PostgreSQL
```rust
use postgres::{Client, NoTls};
use parsql_migrations::postgres_simple::PostgresMigrationConnection;
use parsql_migrations::{MigrationRunner, MigrationConfig};

let mut client = Client::connect("postgresql://localhost/mydb", NoTls)?;
let mut conn = PostgresMigrationConnection::new(&mut client);

let mut runner = MigrationRunner::new();
runner.add_migration(Box::new(CreateUsersTable));

let report = runner.run(&mut conn)?;
println!("Applied {} migrations", report.successful_count());
```

#### SQLite
```rust
use rusqlite::Connection;
use parsql_migrations::sqlite_simple::SqliteMigrationConnection;
use parsql_migrations::MigrationRunner;

let mut sqlite_conn = Connection::open("app.db")?;
let mut conn = SqliteMigrationConnection::new(&mut sqlite_conn);

let mut runner = MigrationRunner::new();
runner.add_migration(Box::new(CreateUsersTable));

let report = runner.run(&mut conn)?;
```

## Migration Files

### SQL Migrations

Create migration files with timestamps:
- `migrations/20240101120000_create_users_table.up.sql`
- `migrations/20240101120000_create_users_table.down.sql`

Load from directory:
```rust
use parsql_migrations::FileMigration;

let migrations = FileMigration::from_directory("migrations")?;
for migration in migrations {
    runner.add_migration(Box::new(migration));
}
```

## Configuration

```rust
use parsql_migrations::{MigrationConfig, TableConfig};

let config = MigrationConfig {
    table: TableConfig {
        table_name: "schema_migrations".to_string(),
        version_column: "version".to_string(),
        name_column: "name".to_string(),
        applied_at_column: "applied_at".to_string(),
        checksum_column: "checksum".to_string(),
        execution_time_column: "execution_time_ms".to_string(),
    },
    verify_checksums: true,
    allow_out_of_order: false,
    transaction_per_migration: true,
};

let runner = MigrationRunner::with_config(config);
```

## Rollback

```rust
// Rollback to a specific version
let report = runner.rollback(&mut conn, 20240101000000)?;
println!("Rolled back {} migrations", report.successful_count());
```

## Status and Validation

```rust
// Get migration status
let statuses = runner.status(&mut conn)?;
for status in statuses {
    println!("{}: {} - {}", 
        status.version, 
        status.name, 
        if status.is_applied { "Applied" } else { "Pending" }
    );
}

// Validate migrations
runner.validate()?; // Checks for gaps and checksum mismatches
```

## CLI Usage

Install the CLI tool:
```bash
cargo install parsql-cli
```

Create a `parsql.toml` configuration file:
```toml
[database]
url = "postgresql://localhost/mydb"

[migrations]
directory = "migrations"
table_name = "schema_migrations"
verify_checksums = true
allow_out_of_order = false
```

Commands:
```bash
# Create a new migration
parsql migrate create "add users table" --migration-type sql

# Run pending migrations
parsql migrate run

# Check migration status
parsql migrate status --detailed

# Rollback to a specific version
parsql migrate rollback --to 20240101000000

# Validate migrations
parsql migrate validate --verify-checksums
```

## Advanced Features

### Custom Migration Connection

Implement `MigrationConnection` for custom database adapters:

```rust
use parsql_migrations::{MigrationConnection, MigrationRecord, Result};

struct MyCustomConnection;

impl MigrationConnection for MyCustomConnection {
    fn execute(&mut self, sql: &str) -> Result<()> {
        // Execute SQL
        Ok(())
    }
    
    fn database_type(&self) -> &str {
        "custom"
    }
    
    fn query_migrations(&mut self, table_name: &str) -> Result<Vec<MigrationRecord>> {
        // Query applied migrations
        Ok(vec![])
    }
}
```

### Async Support (Currently Limited)

Due to `dyn` trait limitations, async support is currently limited. Use sync adapters for now.

## Safety Features

1. **Transaction Safety**: Each migration runs in its own transaction by default
2. **Checksum Verification**: Detects if migration files have been modified after being applied
3. **Gap Detection**: Prevents running migrations with missing versions in the sequence
4. **Idempotency**: Skips already applied migrations

## Error Handling

All operations return `Result<T, MigrationError>`:

```rust
match runner.run(&mut conn) {
    Ok(report) => {
        println!("Success: {} migrations applied", report.successful_count());
    }
    Err(MigrationError::GapDetected { missing_version }) => {
        eprintln!("Missing migration version: {}", missing_version);
    }
    Err(e) => {
        eprintln!("Migration failed: {}", e);
    }
}
```

## License

This project is part of the parsql workspace and shares its license.