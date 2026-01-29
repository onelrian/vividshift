# VividShift - Automated Work Group Distributor

A Rust-based application that automatically distributes household chores among residents every two weeks. The system uses a PostgreSQL database to track people and assignment history, ensuring fair rotation and preventing repetitive assignments.

## Features

- **Configurable Scheduling**: Assignment interval fully configurable (1-365 days, defaults to 14)
- **Fair Rotation**: Tracks assignment history to ensure people don't get the same tasks repeatedly
- **Group-Based Constraints**: Enforces rules based on group membership (Group A vs Group B)
- **Discord Integration**: Automatically posts new assignments to Discord when generated
- **Database-Backed**: Uses Neon PostgreSQL for persistent state management
- **Stateless Execution**: Perfect for CI/CD environments

## Architecture

The application follows a stateless architecture where all state is stored in a PostgreSQL database. For a detailed explanation of the system architecture, including diagrams and data flow, see [docs/architecture.md](docs/architecture.md).

### High-Level Flow

1. **Daily Check**: GitHub Actions runs the application daily at 9 AM UTC
2. **14-Day Rule**: The app checks if 14 days have passed since the last assignment
3. **Assignment Generation**: If eligible, generates new fair work distributions
4. **Discord Notification**: Posts results to Discord (only when new assignments are made)

## Setup

### Prerequisites

- Rust (latest stable version)
- PostgreSQL database (we recommend [Neon](https://neon.tech))
- Diesel CLI: `cargo install diesel_cli --no-default-features --features postgres`

### 1. Clone the Repository

```bash
git clone https://github.com/onelrian/VividShift.git
cd VividShift
```

### 2. Configure Environment

Create a `.env` file in the project root:

```bash
DATABASE_URL=postgresql://user:password@host/dbname?sslmode=require
RUST_LOG=info
```

### 3. Configure Assignment Interval (Optional)

The default interval is 14 days. To customize:

**Option 1: Edit configuration file**

Edit `config/default.toml`:
```toml
# Assignment interval in days (1-365)
assignment_interval_days = 7   # Weekly
# assignment_interval_days = 14  # Bi-weekly (default)
# assignment_interval_days = 30  # Monthly
```

**Option 2: Use environment variable**

Add to `.env`:
```bash
APP__ASSIGNMENT_INTERVAL_DAYS=7  # Weekly assignments
```

> **Note**: Environment variables override file configuration. Valid range: 1-365 days.

### 4. Run Migrations

```bash
diesel migration run
```

This will:
- Create the `people` and `assignments` tables
- Seed initial data from legacy files (if present)

### 5. Run the Application

```bash
cargo run
```

## Database Schema

### `people` Table
Stores resident information and group assignments.

| Column | Type | Description |
|--------|------|-------------|
| id | SERIAL | Primary key |
| name | TEXT | Person's name |
| group_type | TEXT | 'A' or 'B' |
| active | BOOLEAN | Active status |

### `assignments` Table
Tracks assignment history for rotation logic.

| Column | Type | Description |
|--------|------|-------------|
| id | SERIAL | Primary key |
| person_id | INTEGER | Foreign key to people |
| task_name | TEXT | Assigned task |
| assigned_at | TIMESTAMP | Assignment date |

## GitHub Actions Setup

### Required Secrets

Configure these in your GitHub repository settings:

- `DATABASE_URL`: Your Neon PostgreSQL connection string
- `DISCORD_WEBHOOK`: Discord webhook URL for notifications
- `ASSIGNMENT_INTERVAL_DAYS` (optional): Override assignment interval (defaults to 14)

### Workflow Scheduling Options

By default, the workflow uses **manual triggering** (`workflow_dispatch`) for efficiency:

```yaml
on:
  workflow_dispatch:  # Trigger manually or via API
```

**Scheduling Strategies:**

1. **Manual Trigger** (Default)
   - Run from GitHub Actions UI when needed
   - Most efficient - no unnecessary workflow runs

2. **External Scheduler** (Recommended for automation)
   - Use GitHub API with cron service (cron-job.org, etc.)
   - Trigger only when needed based on interval
   - Example: Weekly API call triggers workflow

3. **Daily Cron** (Less efficient but simpler)
   - Uncomment cron schedule in `.github/workflows/worker.yml`
   - Runs daily but only generates assignments when interval passes
   - Can waste Actions minutes on unnecessary checks

The Rust application enforces the configured interval regardless of trigger method.

## Customization

### Modifying Work Assignments

Edit the `work_assignments` HashMap in `src/main.rs`:

```rust
let work_assignments: HashMap<String, usize> = [
    ("Parlor".to_string(), 5),
    ("Frontyard".to_string(), 3),
    ("Backyard".to_string(), 1),
    ("Tank".to_string(), 2),
    ("Toilet B".to_string(), 4),
    ("Toilet A".to_string(), 2),
    ("Bin".to_string(), 1),
]
.into_iter()
.collect();
```

### Adding/Removing People
 
1.  Edit `config/people.toml` (or `people.example.toml` in new envs).
2.  Add/remove `[[person]]` blocks.
3.  Set `active = false` to temporarily remove someone from rotation.
 
```toml
[[person]]
name = "John"
group = "A"
active = true
```

### Changing Constraints

Modify the eligibility logic in `src/group.rs`:

```rust
// Example: Group B restriction for Toilet A
let is_from_b_in_toilet_a = *area == "Toilet A" && names_b_set.contains(person);
```

## Testing

Run the test suite:

```bash
cargo test
```

Tests cover:
- Assignment distribution logic
- Constraint enforcement
- Edge cases (insufficient people, etc.)

## Development

### Project Structure

```
vividshift/
├── src/
│   ├── main.rs          # Entry point, schedule checking
│   ├── db.rs            # Database operations
│   ├── group.rs         # Assignment algorithm
│   ├── models.rs        # Diesel ORM models
│   ├── schema.rs        # Database schema
│   └── output.rs        # Formatting utilities
├── migrations/          # Diesel migrations
├── docs/
│   └── architecture.md  # Detailed architecture docs
└── .github/workflows/
    └── worker.yml       # GitHub Actions workflow
```

### Running Locally

```bash
# Build
cargo build

# Run with database check
cargo run

# Run tests
cargo test

# Format code
cargo fmt
```

## Troubleshooting

### "Could not find a valid assignment after 50 attempts"

This error occurs when the constraints are too restrictive. Possible solutions:
- Check that you have enough people for all tasks
- Review the assignment history (people might be blocked from all available tasks)
- Consider adjusting the `HISTORY_LENGTH` in `src/db.rs`

### "Failed to get DB connection"

Ensure:
- Your `DATABASE_URL` is correct in `.env`
- The database is accessible from your network
- Migrations have been run successfully

### Discord notifications not sent

Check:
- The `SHOULD_NOTIFY` environment variable is set to `true` (only happens when assignments are generated)
- Your `DISCORD_WEBHOOK` secret is configured correctly
- The workflow has the necessary permissions

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
