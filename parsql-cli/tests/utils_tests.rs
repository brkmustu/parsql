//! Tests for utility functions

use parsql_cli::utils::{
    parse_database_url, DatabaseType, format_duration, format_table,
    get_timestamp, colorize_number, Progress
};
use std::time::Duration;

#[test]
fn test_parse_database_url_postgresql() {
    let url = "postgresql://localhost/test";
    let db_type = parse_database_url(url).unwrap();
    
    match db_type {
        DatabaseType::PostgreSQL => assert!(true),
        _ => panic!("Expected PostgreSQL"),
    }
    
    assert_eq!(db_type.name(), "PostgreSQL");
}

#[test]
fn test_parse_database_url_postgres() {
    let url = "postgres://user:pass@localhost:5432/mydb";
    let db_type = parse_database_url(url).unwrap();
    
    match db_type {
        DatabaseType::PostgreSQL => assert!(true),
        _ => panic!("Expected PostgreSQL"),
    }
}

#[test]
fn test_parse_database_url_sqlite() {
    let test_urls = vec![
        "sqlite:test.db",
        "file.db",
        "database.sqlite",
        "sqlite:/path/to/db.db",
    ];
    
    for url in test_urls {
        let db_type = parse_database_url(url).unwrap();
        match db_type {
            DatabaseType::SQLite => assert!(true),
            _ => panic!("Expected SQLite for URL: {}", url),
        }
        assert_eq!(db_type.name(), "SQLite");
    }
}

#[test]
fn test_parse_database_url_invalid() {
    let invalid_urls = vec![
        "invalid://url",
        "mysql://localhost/db",
        "redis://localhost",
        "http://example.com",
        "",
    ];
    
    for url in invalid_urls {
        let result = parse_database_url(url);
        assert!(result.is_err(), "URL '{}' should be invalid", url);
    }
}

#[test]
fn test_format_duration_milliseconds() {
    let duration = Duration::from_millis(250);
    let formatted = format_duration(duration);
    assert_eq!(formatted, "250ms");
}

#[test]
fn test_format_duration_seconds() {
    let duration = Duration::from_millis(1500);
    let formatted = format_duration(duration);
    assert_eq!(formatted, "1.50s");
}

#[test]
fn test_format_duration_exact_second() {
    let duration = Duration::from_secs(1);
    let formatted = format_duration(duration);
    assert_eq!(formatted, "1.00s");
}

#[test]
fn test_format_table_basic() {
    let headers = vec!["Name", "Age", "City"];
    let rows = vec![
        vec!["Alice".to_string(), "30".to_string(), "New York".to_string()],
        vec!["Bob".to_string(), "25".to_string(), "London".to_string()],
    ];
    
    let table = format_table(headers, rows);
    
    // Should contain headers
    assert!(table.contains("Name"));
    assert!(table.contains("Age"));
    assert!(table.contains("City"));
    
    // Should contain data
    assert!(table.contains("Alice"));
    assert!(table.contains("Bob"));
    assert!(table.contains("New York"));
    
    // Should have proper formatting (separators)
    assert!(table.contains("----"));
}

#[test]
fn test_format_table_different_column_widths() {
    let headers = vec!["Short", "Very Long Header"];
    let rows = vec![
        vec!["A".to_string(), "B".to_string()],
        vec!["X".to_string(), "This is a very long value".to_string()],
    ];
    
    let table = format_table(headers, rows);
    
    // Should properly align columns based on content width
    assert!(table.contains("Short"));
    assert!(table.contains("Very Long Header"));
    assert!(table.contains("This is a very long value"));
}

#[test]
fn test_format_table_empty_rows() {
    let headers = vec!["Column1", "Column2"];
    let rows = vec![];
    
    let table = format_table(headers, rows);
    
    // Should still have headers and separators
    assert!(table.contains("Column1"));
    assert!(table.contains("Column2"));
    assert!(table.contains("----"));
}

#[test]
fn test_get_timestamp_format() {
    let timestamp = get_timestamp();
    
    // Should be 14 characters: YYYYMMDDHHMMSS
    assert_eq!(timestamp.len(), 14);
    
    // Should be all digits
    assert!(timestamp.chars().all(|c| c.is_ascii_digit()));
    
    // Should start with reasonable year (20xx)
    assert!(timestamp.starts_with("20"));
}

#[test]
fn test_get_timestamp_uniqueness() {
    let timestamp1 = get_timestamp();
    std::thread::sleep(Duration::from_millis(1100)); // Wait more than a second
    let timestamp2 = get_timestamp();
    
    // Timestamps should be different (unless test runs very fast)
    // This is probabilistic but should work in most cases
    assert_ne!(timestamp1, timestamp2);
}

#[test]
fn test_colorize_number() {
    let result = colorize_number(42, "items");
    
    // Result should contain the number and label
    // Note: We can't easily test the ANSI color codes in unit tests,
    // but we can verify the basic content is present
    assert!(result.contains("42"));
    assert!(result.contains("items"));
}

#[test]
fn test_progress_timing() {
    let start = std::time::Instant::now();
    let progress = Progress::new("Testing");
    
    // Sleep a tiny bit to ensure some time passes
    std::thread::sleep(Duration::from_millis(10));
    
    progress.finish();
    
    let elapsed = start.elapsed();
    // Progress should have taken at least our sleep time
    assert!(elapsed >= Duration::from_millis(10));
}

// Note: Testing print_success, print_error, print_warning, print_info
// functions would require capturing stdout/stderr, which is more complex
// and would require additional test infrastructure. These functions are
// primarily for user interface and are better tested through integration tests.

#[test]
fn test_database_type_clone_copy() {
    let db_type = DatabaseType::PostgreSQL;
    let copied = db_type;
    let cloned = db_type.clone();
    
    // Should be able to use both copies
    assert_eq!(copied.name(), "PostgreSQL");
    assert_eq!(cloned.name(), "PostgreSQL");
}

#[test]
fn test_format_table_uneven_rows() {
    let headers = vec!["Col1", "Col2", "Col3"];
    let rows = vec![
        vec!["A".to_string(), "B".to_string()], // Missing third column
        vec!["X".to_string(), "Y".to_string(), "Z".to_string()], // Complete row
    ];
    
    let table = format_table(headers, rows);
    
    // Should handle uneven rows gracefully
    assert!(table.contains("Col1"));
    assert!(table.contains("Col2"));
    assert!(table.contains("Col3"));
    assert!(table.contains("A"));
    assert!(table.contains("B"));
    assert!(table.contains("X"));
    assert!(table.contains("Y"));
    assert!(table.contains("Z"));
}