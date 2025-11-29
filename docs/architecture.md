# VividShift Architecture

This document provides a detailed overview of the VividShift work group generator architecture, including the database schema, application flow, and scheduling logic.

## Overview

VividShift is a Rust-based application that automatically distributes household chores among residents every two weeks. It uses a PostgreSQL database (Neon) to store people, their group assignments, and historical task assignments. The application runs on a daily schedule via GitHub Actions but only generates new assignments if 14 days have passed since the last run.

## System Architecture

```mermaid
graph TB
    subgraph "GitHub Actions"
        A[Daily Cron Trigger<br/>0 9 * * *]
        B[Build & Run Rust App]
        C{SHOULD_NOTIFY<br/>== true?}
        D[Send Discord Notification]
    end
    
    subgraph "Rust Application"
        E[Check DB:<br/>14 days passed?]
        F{Days >= 14?}
        G[Exit Early<br/>Set SHOULD_NOTIFY=false]
        H[Fetch People & History]
        I[Generate Assignments]
        J[Save to DB<br/>Set SHOULD_NOTIFY=true]
    end
    
    subgraph "Neon Database"
        K[(people table)]
        L[(assignments table)]
    end
    
    A --> B
    B --> E
    E --> F
    F -->|No| G
    F -->|Yes| H
    H --> K
    H --> L
    H --> I
    I --> J
    J --> L
    J --> C
    G --> C
    C -->|Yes| D
    C -->|No| End[Workflow Complete]
    D --> End
```

## Database Schema

### Tables

#### `people`
Stores information about all residents.

| Column | Type | Description |
|--------|------|-------------|
| `id` | SERIAL | Primary key |
| `name` | TEXT | Person's name |
| `group_type` | TEXT | 'A' or 'B' (determines task eligibility) |
| `active` | BOOLEAN | Whether the person is currently active |

#### `assignments`
Stores the history of task assignments.

| Column | Type | Description |
|--------|------|-------------|
| `id` | SERIAL | Primary key |
| `person_id` | INTEGER | Foreign key to `people.id` |
| `task_name` | TEXT | Name of the assigned task |
| `assigned_at` | TIMESTAMP | When the assignment was made |

### Relationships

```mermaid
erDiagram
    people ||--o{ assignments : has
    people {
        int id PK
        text name
        text group_type
        boolean active
    }
    assignments {
        int id PK
        int person_id FK
        text task_name
        timestamp assigned_at
    }
```

## Application Flow

### 1. Startup & Schedule Check

When the application starts, it first checks if 14 days have passed since the last assignment run.

```mermaid
sequenceDiagram
    participant App
    participant DB
    participant Env as GITHUB_ENV
    
    App->>DB: SELECT MAX(assigned_at) FROM assignments
    DB-->>App: last_run_date
    
    alt Days since last run < 14
        App->>Env: Write SHOULD_NOTIFY=false
        App->>App: Exit early
    else Days >= 14 or no history
        App->>App: Continue to assignment generation
    end
```

### 2. Assignment Generation

If the schedule check passes, the application proceeds to generate new assignments.

```mermaid
flowchart TD
    A[Fetch Active People from DB] --> B[Fetch Assignment History]
    B --> C[Build Candidate Lists<br/>Based on Constraints]
    C --> D{Valid Assignment<br/>Found?}
    D -->|Yes| E[Save to Database]
    D -->|No| F[Retry with New Shuffle<br/>Max 50 attempts]
    F --> C
    E --> G[Set SHOULD_NOTIFY=true]
    G --> H[Print Results]
```

### 3. Constraints & Rules

The assignment algorithm enforces several rules:

- **History Check**: People cannot be assigned to a task they performed in the last 2 runs (stored in history)
- **Group Restrictions**:
  - Group B members cannot do "Toilet A"
  - Group A members cannot do "Toilet B"
- **Capacity**: Each task has a fixed number of slots
- **Fairness**: The algorithm uses a constraint satisfaction approach to find valid distributions

### 4. GitHub Actions Integration

The application integrates with GitHub Actions through environment variables.

```mermaid
sequenceDiagram
    participant GHA as GitHub Actions
    participant App as Rust App
    participant Env as $GITHUB_ENV
    participant Discord
    
    GHA->>App: Run daily (cron: 0 9 * * *)
    App->>App: Check 14-day rule
    
    alt Should run
        App->>Env: SHOULD_NOTIFY=true
        App->>App: Generate assignments
    else Too soon
        App->>Env: SHOULD_NOTIFY=false
    end
    
    App-->>GHA: Exit
    GHA->>GHA: Read $GITHUB_ENV
    
    alt SHOULD_NOTIFY == true
        GHA->>Discord: Send notification with assignments
    else SHOULD_NOTIFY == false
        GHA->>GHA: Skip Discord notification
    end
```

## Key Components

### `src/main.rs`
- Entry point
- Handles schedule checking
- Writes to `GITHUB_ENV` for conditional workflow steps

### `src/db.rs`
- Database connection management (connection pooling with r2d2)
- Functions to fetch people and history
- `should_run()`: Checks if 14 days have passed
- `save_assignments()`: Persists new assignments

### `src/group.rs`
- Core assignment algorithm
- Constraint satisfaction logic
- Retry mechanism (up to 50 attempts)

### `src/models.rs` & `src/schema.rs`
- Diesel ORM models and schema definitions
- Type-safe database interactions

## Deployment

### Environment Variables

The application requires the following environment variable:

```bash
DATABASE_URL=postgresql://user:password@host/dbname?sslmode=require
```

### GitHub Secrets

Configure these secrets in your GitHub repository:

- `DATABASE_URL`: Neon PostgreSQL connection string
- `DISCORD_WEBHOOK`: Discord webhook URL for notifications

### Workflow Schedule

The GitHub Actions workflow runs daily at 9 AM UTC:

```yaml
schedule:
  - cron: '0 9 * * *'
```

However, the Rust application itself enforces the 14-day interval, so notifications are only sent every two weeks.

## Testing

The project includes unit tests for the core distribution logic:

```bash
cargo test
```

Tests verify:
- Basic assignment distribution with sufficient people
- Failure handling when there are insufficient people
- Constraint enforcement

## Migration Management

Database schema is managed through Diesel migrations:

```bash
# Run pending migrations
diesel migration run

# Revert last migration
diesel migration revert
```

Current migrations:
1. `2024-01-01-000000_create_initial_tables`: Creates `people` and `assignments` tables
2. `2025-11-28-141600_seed_initial_data`: Seeds initial data from legacy files
