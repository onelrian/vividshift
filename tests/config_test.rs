use work_group_generator::config::Settings;
use serial_test::serial;

/// Test that config/default.toml provides default interval of 14 days
#[test]
#[serial]
fn test_default_interval_from_config_file() {
    // Setup: Ensure no env override
    std::env::remove_var("APP__ASSIGNMENT_INTERVAL_DAYS");
    std::env::set_var("DATABASE_URL", "postgres://dummy:dummy@localhost/dummy");
    
    // When: Loading settings (reads from config/default.toml)
    let settings = Settings::new().expect("Failed to load settings");
    
    // Then: Should get 14 from config/default.toml
    assert_eq!(
        settings.assignment_interval_days(),
        14,
        "Default from config file should be 14 days"
    );
    
    // Cleanup
    std::env::remove_var("DATABASE_URL");
}

/// Test that environment variable APP__ASSIGNMENT_INTERVAL_DAYS overrides config file
#[test]
#[serial]
fn test_env_override_takes_precedence() {
    // Given: Env var set to override config file value (14)
    std::env::set_var("APP__ASSIGNMENT_INTERVAL_DAYS", "7");
    std::env::set_var("DATABASE_URL", "postgres://dummy:dummy@localhost/dummy");
    
    // When: Loading settings
    let settings = Settings::new().expect("Failed to load settings");
    
    // Then: Env var should override config file
    assert_eq!(
        settings.assignment_interval_days(),
        7,
        "Environment variable should override config file"
    );
    
    // Cleanup
    std::env::remove_var("APP__ASSIGNMENT_INTERVAL_DAYS");
    std::env::remove_var("DATABASE_URL");
}

/// Test validation: values < 1 are clamped to 14
#[test]
#[serial]
fn test_validation_clamps_zero_to_default() {
    std::env::set_var("APP__ASSIGNMENT_INTERVAL_DAYS", "0");
    std::env::set_var("DATABASE_URL", "postgres://dummy:dummy@localhost/dummy");
    
    let settings = Settings::new().expect("Failed to load settings");
    
    // Validation should clamp 0 to 14
    assert_eq!(
        settings.assignment_interval_days(),
        14,
        "Zero interval should be clamped to 14"
    );
    
    // Cleanup
    std::env::remove_var("APP__ASSIGNMENT_INTERVAL_DAYS");
    std::env::remove_var("DATABASE_URL");
}

/// Test validation: values > 365 are clamped to 365
#[test]
#[serial]
fn test_validation_clamps_excessive_to_max() {
    std::env::set_var("APP__ASSIGNMENT_INTERVAL_DAYS", "500");
    std::env::set_var("DATABASE_URL", "postgres://dummy:dummy@localhost/dummy");
    
    let settings = Settings::new().expect("Failed to load settings");
    
    // Validation should clamp 500 to 365
    assert_eq!(
        settings.assignment_interval_days(),
        365,
        "Excessive interval should be clamped to 365"
    );
    
    // Cleanup
    std::env::remove_var("APP__ASSIGNMENT_INTERVAL_DAYS");
    std::env::remove_var("DATABASE_URL");
}

/// Test validation: negative values are clamped to 14
#[test]
#[serial]
fn test_validation_clamps_negative_to_default() {
    std::env::set_var("APP__ASSIGNMENT_INTERVAL_DAYS", "-10");
    std::env::set_var("DATABASE_URL", "postgres://dummy:dummy@localhost/dummy");
    
    let settings = Settings::new().expect("Failed to load settings");
    
    // Validation should clamp negative to 14
    assert_eq!(
        settings.assignment_interval_days(),
        14,
        "Negative interval should be clamped to 14"
    );
    
    // Cleanup
    std::env::remove_var("APP__ASSIGNMENT_INTERVAL_DAYS");
    std::env::remove_var("DATABASE_URL");
}

/// Test common use case: weekly (7-day) interval
#[test]
#[serial]
fn test_weekly_interval_via_env() {
    std::env::set_var("APP__ASSIGNMENT_INTERVAL_DAYS", "7");
    std::env::set_var("DATABASE_URL", "postgres://dummy:dummy@localhost/dummy");
    
    let settings = Settings::new().expect("Failed to load settings");
    assert_eq!(settings.assignment_interval_days(), 7, "Weekly interval");
    
    std::env::remove_var("APP__ASSIGNMENT_INTERVAL_DAYS");
    std::env::remove_var("DATABASE_URL");
}

/// Test common use case: monthly (30-day) interval
#[test]
#[serial]
fn test_monthly_interval_via_env() {
    std::env::set_var("APP__ASSIGNMENT_INTERVAL_DAYS", "30");
    std::env::set_var("DATABASE_URL", "postgres://dummy:dummy@localhost/dummy");
    
    let settings = Settings::new().expect("Failed to load settings");
    assert_eq!(settings.assignment_interval_days(), 30, "Monthly interval");
    
    std::env::remove_var("APP__ASSIGNMENT_INTERVAL_DAYS");
    std::env::remove_var("DATABASE_URL");
}
