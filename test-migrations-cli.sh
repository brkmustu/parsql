#!/bin/bash

# Test script for migration commands

TEST_DIR="/home/burak/kaynak/projeler/parsql-libs/parsql/test-cli-migrations"
rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

echo "Working directory: $(pwd)"

# Create a sample migration
mkdir -p migrations
cat > migrations/20250101000000_create_users.up.sql << 'EOF'
-- Create users table
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
EOF

cat > migrations/20250101000000_create_users.down.sql << 'EOF'
-- Drop users table
DROP TABLE IF EXISTS users;
EOF

cat > migrations/20250102000000_add_posts.up.sql << 'EOF'
-- Create posts table
CREATE TABLE posts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    content TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
EOF

cat > migrations/20250102000000_add_posts.down.sql << 'EOF'
-- Drop posts table
DROP TABLE IF EXISTS posts;
EOF

echo ""
echo "Created test migrations:"
ls -la migrations/

echo ""
echo "Now run the CLI:"
echo "  cd $TEST_DIR"
echo "  ../target/release/parsql"
echo ""
echo "Commands to test:"
echo "  /connect sqlite:test.db"
echo "  /status"
echo "  /run"
echo "  /status"
echo "  /rollback 20250101000000"
echo "  /status"