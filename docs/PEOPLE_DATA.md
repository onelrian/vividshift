# People Configuration

## Overview

Structured TOML configuration for managing resident data, replacing legacy `file_a.txt` and `file_b.txt`.

### File Structure

- **Config**: `config/people.toml` (Source of Truth)
- **Module**: `src/people_config.rs`
- **Tests**: `tests/people_config_test.rs`

### Extending Metadata

To add new fields (e.g., email, preferences):
1.  Add the field to the `[[person]]` table in `people.toml`.
2.  Update `PersonConfig` struct in `src/people_config.rs`.
3.  The `active` flag handles temporary removal from rotation.

## Structure

### Groups
```toml
[groups.A]
description = "Group A residents"
constraints = ["cannot_perform_toilet_b"]
```

### People
```toml
[[person]]
name = "Onel"
group = "A"
active = true  # defaults to true
```

## Usage

```rust
use work_group_generator::people_config::PeopleConfiguration;

// Load and validate
let config = PeopleConfiguration::load()?;

// Query
let active_a = config.get_active_people_by_group("A");
let person = config.find_person("Onel");
```

## Validation

Automatically enforces:
- Unique person names
- Valid group references
- At least one active member per group
- Non-empty configuration

## Maintenance

**Add person:**
```toml
[[person]]
name = "NewPerson"
group = "A"
active = true
```

**Deactivate person:**
```toml
[[person]]
name = "SomePerson"
group = "A"
active = false  # preserve data, exclude from operations
```

**Add group:**
```toml
[groups.C]
description = "Group C residents"
constraints = []
```

## Migration

All 18 people migrated from legacy files:
- **Group A** (file_a.txt): 8 people
- **Group B** (file_b.txt): 10 people

## Testing

```bash
cargo test people_config
```
