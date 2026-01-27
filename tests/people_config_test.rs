//! Integration tests for people configuration
//!
//! These tests verify that the people.toml configuration file loads correctly
//! and contains all expected data from the legacy file_a.txt and file_b.txt.

use work_group_generator::people_config::{ConfigError, PeopleConfiguration};

/// Test that the configuration file can be loaded successfully
#[test]
fn test_load_people_configuration() {
    let result = PeopleConfiguration::load();
    assert!(
        result.is_ok(),
        "Should successfully load people configuration: {:?}",
        result.err()
    );
}

/// Test that configuration contains exactly 18 people (8 Group A + 10 Group B)
#[test]
fn test_total_people_count() {
    let config = PeopleConfiguration::load().expect("Failed to load configuration");
    assert_eq!(
        config.total_people(),
        18,
        "Expected 18 total people (8 from Group A + 10 from Group B)"
    );
}

/// Test Group A has exactly 8 people
#[test]
fn test_group_a_count() {
    let config = PeopleConfiguration::load().expect("Failed to load configuration");
    let group_a_people = config.get_people_by_group("A");
    assert_eq!(
        group_a_people.len(),
        8,
        "Group A should have exactly 8 people"
    );
}

/// Test Group B has exactly 10 people
#[test]
fn test_group_b_count() {
    let config = PeopleConfiguration::load().expect("Failed to load configuration");
    let group_b_people = config.get_people_by_group("B");
    assert_eq!(
        group_b_people.len(),
        10,
        "Group B should have exactly 10 people"
    );
}

/// Test that all people from legacy file_a.txt (Group A) are present
#[test]
fn test_group_a_names_from_legacy() {
    let config = PeopleConfiguration::load().expect("Failed to load configuration");

    // Names from file_a.txt
    let expected_names = vec![
        "Onel", "Prosper", "Derick", "Nathan", "Junine", "Maxwell", "Severian", "Frank",
    ];

    let group_a_people = config.get_people_by_group("A");
    let actual_names: Vec<&str> = group_a_people.iter().map(|p| p.name.as_str()).collect();

    for expected_name in &expected_names {
        assert!(
            actual_names.contains(expected_name),
            "Group A missing expected person: {}",
            expected_name
        );
    }

    assert_eq!(
        group_a_people.len(),
        expected_names.len(),
        "Group A should have exactly {} people",
        expected_names.len()
    );
}

/// Test that all people from legacy file_b.txt (Group B) are present
#[test]
fn test_group_b_names_from_legacy() {
    let config = PeopleConfiguration::load().expect("Failed to load configuration");

    // Names from file_b.txt
    let expected_names = vec![
        "Emmanuel",
        "Romeo",
        "Ghislain",
        "Vitalis",
        "Richmond",
        "Christian",
        "Michael",
        "Mark",
        "Usher",
        "Desmond",
    ];

    let group_b_people = config.get_people_by_group("B");
    let actual_names: Vec<&str> = group_b_people.iter().map(|p| p.name.as_str()).collect();

    for expected_name in &expected_names {
        assert!(
            actual_names.contains(expected_name),
            "Group B missing expected person: {}",
            expected_name
        );
    }

    assert_eq!(
        group_b_people.len(),
        expected_names.len(),
        "Group B should have exactly {} people",
        expected_names.len()
    );
}

/// Test that all people are active by default
#[test]
fn test_all_people_active() {
    let config = PeopleConfiguration::load().expect("Failed to load configuration");
    let active_count = config.active_people_count();
    assert_eq!(
        active_count, 18,
        "All 18 people should be active by default"
    );
}

/// Test that both groups have definitions
#[test]
fn test_groups_defined() {
    let config = PeopleConfiguration::load().expect("Failed to load configuration");

    assert!(config.get_group("A").is_some(), "Group A should be defined");
    assert!(config.get_group("B").is_some(), "Group B should be defined");
}

/// Test Group A constraints
#[test]
fn test_group_a_constraints() {
    let config = PeopleConfiguration::load().expect("Failed to load configuration");
    let group_a = config.get_group("A").expect("Group A should exist");

    assert!(
        group_a
            .constraints
            .contains(&"cannot_perform_toilet_b".to_string()),
        "Group A should have 'cannot_perform_toilet_b' constraint"
    );
}

/// Test Group B constraints
#[test]
fn test_group_b_constraints() {
    let config = PeopleConfiguration::load().expect("Failed to load configuration");
    let group_b = config.get_group("B").expect("Group B should exist");

    assert!(
        group_b
            .constraints
            .contains(&"cannot_perform_toilet_a".to_string()),
        "Group B should have 'cannot_perform_toilet_a' constraint"
    );
}

/// Test filtering active people by group
#[test]
fn test_get_active_people_by_group() {
    let config = PeopleConfiguration::load().expect("Failed to load configuration");

    let active_a = config.get_active_people_by_group("A");
    assert_eq!(active_a.len(), 8, "Should have 8 active people in Group A");

    let active_b = config.get_active_people_by_group("B");
    assert_eq!(
        active_b.len(),
        10,
        "Should have 10 active people in Group B"
    );
}

/// Test person lookup by name
#[test]
fn test_find_person() {
    let config = PeopleConfiguration::load().expect("Failed to load configuration");

    let onel = config.find_person("Onel");
    assert!(onel.is_some(), "Should find person 'Onel'");
    assert_eq!(onel.unwrap().group, "A", "Onel should be in Group A");

    let romeo = config.find_person("Romeo");
    assert!(romeo.is_some(), "Should find person 'Romeo'");
    assert_eq!(romeo.unwrap().group, "B", "Romeo should be in Group B");

    let nonexistent = config.find_person("NonExistent");
    assert!(nonexistent.is_none(), "Should not find non-existent person");
}

/// Test has_person check
#[test]
fn test_has_person() {
    let config = PeopleConfiguration::load().expect("Failed to load configuration");

    assert!(config.has_person("Onel"), "Should have person 'Onel'");
    assert!(
        config.has_person("Emmanuel"),
        "Should have person 'Emmanuel'"
    );
    assert!(!config.has_person("FakeName"), "Should not have 'FakeName'");
}

/// Test error handling for missing file
#[test]
fn test_missing_config_file() {
    let result = PeopleConfiguration::load_from_path("nonexistent/path.toml");
    assert!(matches!(result, Err(ConfigError::NotFound(_))));
}

/// Test no duplicate names
#[test]
fn test_no_duplicate_names() {
    let config = PeopleConfiguration::load().expect("Failed to load configuration");

    let mut seen_names = std::collections::HashSet::new();
    for person in &config.people {
        assert!(
            seen_names.insert(&person.name),
            "Duplicate name found: {}",
            person.name
        );
    }
}

/// Test that all names from assignment_history.json are present
#[test]
fn test_assignment_history_names_present() {
    let config = PeopleConfiguration::load().expect("Failed to load configuration");

    // All names from assignment_history.json
    let history_names = vec![
        "Mark",
        "Romeo",
        "Michael",
        "Ghislain",
        "Nathan",
        "Frank",
        "Severian",
        "Junine",
        "Prosper",
        "Desmond",
        "Vitalis",
        "Richmond",
        "Emmanuel",
        "Christian",
        "Maxwell",
        "Usher",
        "Derick",
        "Onel",
    ];

    for name in history_names {
        assert!(
            config.has_person(name),
            "Person from assignment history missing: {}",
            name
        );
    }
}
