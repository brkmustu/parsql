# parsql-cli

Command-line interface for the parsql database toolkit and migration system.

## Installation

```bash
cargo install parsql-cli
```

Or build from source:
```bash
git clone https://github.com/yazdostum-nettr/parsql
cd parsql/parsql-cli
cargo install --path .
```

## Quick Start

1. **Initialize a new project**:
```bash
parsql init
```

This creates:
- `parsql.toml` - Configuration file
- `migrations/` - Directory for migration files
- `.gitignore` - Ignores database files

2. **Create a migration**:
```bash
parsql migrate create "create users table" --migration-type sql
```

3. **Run migrations**:
```bash
parsql migrate run
```

## Configuration

Create a `parsql.toml` file in your project root:

```toml
[database]
url = "postgresql://user:pass@localhost/dbname"
# Or for SQLite:
# url = "sqlite:app.db"

[migrations]
directory = "migrations"
table_name = "schema_migrations"
verify_checksums = true
allow_out_of_order = false
transaction_per_migration = true
```

### Environment Variables

You can override configuration with environment variables:
- `DATABASE_URL` - Database connection string
- `PARSQL_MIGRATIONS_DIR` - Migrations directory
- `PARSQL_CONFIG` - Path to config file

## Commands

### `parsql init`

Initialize a new parsql project in the current directory.

Options:
- `--database-url <URL>` - Set initial database URL
- `--migrations-dir <DIR>` - Set migrations directory (default: "migrations")

### `parsql migrate create`

Create a new migration file.

```bash
parsql migrate create "add users table" --migration-type sql
```

Options:
- `--migration-type <TYPE>` - Migration type: `sql` or `rust` (default: sql)

Creates:
- `{timestamp}_{name}.up.sql` - Forward migration
- `{timestamp}_{name}.down.sql` - Rollback migration

### `parsql migrate run`

Run all pending migrations.

```bash
parsql migrate run
parsql migrate run --database-url sqlite:test.db
parsql migrate run --target 20240101120000  # Run up to specific version
parsql migrate run --dry-run  # Show what would be run
```

Options:
- `--database-url <URL>` - Override database URL
- `--target <VERSION>` - Run migrations up to this version
- `--dry-run` - Show migrations without running them

### `parsql migrate rollback`

Rollback migrations to a specific version.

```bash
parsql migrate rollback --to 20240101000000
parsql migrate rollback --to 0  # Rollback all
```

Options:
- `--to <VERSION>` - Target version (required)
- `--database-url <URL>` - Override database URL
- `--dry-run` - Show what would be rolled back

### `parsql migrate status`

Show migration status.

```bash
parsql migrate status
parsql migrate status --detailed  # Show checksums and timestamps
```

Options:
- `--database-url <URL>` - Override database URL
- `--detailed` - Show detailed information

Example output:
```
Migration Status:
  10 Total migrations
  7 Applied migrations
  3 Pending migrations

Detailed Status:
Version         Name                Status    Applied At           Checksum
--------------  ------------------  --------  -------------------  ----------
20240101120000  create_users_table  Applied   20.5.01-01 12:00:00  ✓
20240102130000  add_posts_table     Applied   20.5.01-02 13:00:00  ✓
20240103140000  add_comments        Pending   -                    -
```

### `parsql migrate validate`

Validate migration files.

```bash
parsql migrate validate
parsql migrate validate --check-gaps
parsql migrate validate --verify-checksums
```

Options:
- `--check-gaps` - Check for version gaps
- `--verify-checksums` - Verify migration checksums

### `parsql migrate list`

List available migrations.

```bash
parsql migrate list
parsql migrate list --pending  # Only pending
parsql migrate list --applied  # Only applied
```

Options:
- `--pending` - Show only pending migrations
- `--applied` - Show only applied migrations

## Migration File Format

### SQL Migrations

**Up migration** (`{version}_{timestamp}_{name}.up.sql`):
```sql
-- Migration: create_users_table
-- Version: 20240101120000
-- Created: 20.5.01-01 12:00:00

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

**Down migration** (`{version}_{timestamp}_{name}.down.sql`):
```sql
-- Migration: create_users_table (rollback)
-- Version: 20240101120000
-- Created: 20.5.01-01 12:00:00

DROP TABLE IF EXISTS users;
```

### Rust Migrations

For complex migrations, you can use Rust:

```rust
//! Migration: complex_data_migration
//! Version: 20240101120000
//! Created: 20.5.01-01 12:00:00

use parsql_migrations::prelude::*;

pub struct Migration20240101120000;

impl Migration for Migration20240101120000 {
    fn version(&self) -> i64 {
        20240101120000
    }
    
    fn name(&self) -> &str {
        "complex_data_migration"
    }
    
    fn up(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        // Complex migration logic
        conn.execute("ALTER TABLE users ADD COLUMN status VARCHAR(50)")?;
        // Update data...
        Ok(())
    }
    
    fn down(&self, conn: &mut dyn MigrationConnection) -> Result<(), MigrationError> {
        conn.execute("ALTER TABLE users DROP COLUMN status")?;
        Ok(())
    }
}
```

## Features

### Transaction Safety
Each migration runs in its own transaction by default. If a migration fails, it's automatically rolled back.

### Checksum Verification
Migrations are checksummed to detect modifications after they've been applied. The status command shows checksum mismatches.

### Gap Detection
The system detects gaps in migration versions to ensure migrations run in order.

### Progress Indicators
Long-running operations show progress with spinner animations and execution times.

### Colored Output
- ✓ Success messages in green
- ✗ Errors in red
- ⚠ Warnings in yellow
- ℹ Info in blue

## Examples

### PostgreSQL Project
```bash
# Initialize
parsql init --database-url postgresql://localhost/myapp

# Create migrations
parsql migrate create "create users table"
parsql migrate create "create posts table"
parsql migrate create "add user profiles"

# Run migrations
parsql migrate run

# Check status
parsql migrate status --detailed

# Rollback if needed
parsql migrate rollback --to 20240101000000
```

### SQLite Project
```bash
# Initialize with SQLite
parsql init --database-url sqlite:app.db

# Create and run migrations
parsql migrate create "initial schema"
parsql migrate run

# Validate migrations
parsql migrate validate --verify-checksums
```

## Troubleshooting

### Connection Errors
- Verify DATABASE_URL is correct
- For PostgreSQL: Check server is running and credentials are valid
- For SQLite: Ensure directory permissions are correct

### Migration Failures
- Check SQL syntax matches your database
- Verify table/column names don't conflict
- Use `--dry-run` to preview changes
- Check logs for detailed error messages

### Checksum Mismatches
- Don't modify migration files after they're applied
- If you must change a migration, rollback first
- Use `validate --verify-checksums` to find issues

## License

This project is part of the parsql workspace and shares its license.