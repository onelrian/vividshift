#!/bin/bash

# VividShift Database Restore Script
# This script restores PostgreSQL database from backups

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BACKUP_DIR="${PROJECT_ROOT}/data/backups"
LOG_FILE="${PROJECT_ROOT}/logs/restore.log"

# Default values
DB_URL="${VIVIDSHIFT_DATABASE_URL:-postgresql://postgres:password@localhost:5432/vividshift_dev}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} $1" | tee -a "$LOG_FILE"
}

error() {
    echo -e "${RED}[$(date '+%Y-%m-%d %H:%M:%S')] ERROR:${NC} $1" | tee -a "$LOG_FILE"
}

success() {
    echo -e "${GREEN}[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS:${NC} $1" | tee -a "$LOG_FILE"
}

warning() {
    echo -e "${YELLOW}[$(date '+%Y-%m-%d %H:%M:%S')] WARNING:${NC} $1" | tee -a "$LOG_FILE"
}

# Help function
show_help() {
    cat << EOF
VividShift Database Restore Script

Usage: $0 [OPTIONS] BACKUP_FILE

Options:
    -h, --help              Show this help message
    -d, --database URL      Database URL (default: from VIVIDSHIFT_DATABASE_URL)
    -l, --list              List available backups
    --latest                Restore from the latest backup
    --docker                Use Docker to run psql
    --drop-database         Drop and recreate database before restore
    --no-owner              Skip ownership restoration
    --no-privileges         Skip privilege restoration
    --dry-run               Show what would be done without executing
    -y, --yes               Skip confirmation prompts

Examples:
    $0 backup_file.sql                         # Restore from specific file
    $0 --latest                                # Restore from latest backup
    $0 -l                                      # List available backups
    $0 --drop-database backup_file.sql         # Drop DB and restore
    $0 --docker backup_file.sql                # Use Docker for psql

EOF
}

# Parse command line arguments
USE_DOCKER=false
DRY_RUN=false
LIST_BACKUPS=false
USE_LATEST=false
DROP_DATABASE=false
NO_OWNER=false
NO_PRIVILEGES=false
SKIP_CONFIRMATION=false
BACKUP_FILE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -d|--database)
            DB_URL="$2"
            shift 2
            ;;
        -l|--list)
            LIST_BACKUPS=true
            shift
            ;;
        --latest)
            USE_LATEST=true
            shift
            ;;
        --docker)
            USE_DOCKER=true
            shift
            ;;
        --drop-database)
            DROP_DATABASE=true
            shift
            ;;
        --no-owner)
            NO_OWNER=true
            shift
            ;;
        --no-privileges)
            NO_PRIVILEGES=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -y|--yes)
            SKIP_CONFIRMATION=true
            shift
            ;;
        -*)
            error "Unknown option: $1"
            show_help
            exit 1
            ;;
        *)
            BACKUP_FILE="$1"
            shift
            ;;
    esac
done

# Create necessary directories
mkdir -p "$(dirname "$LOG_FILE")"

# Extract database info from URL
if [[ $DB_URL =~ postgresql://([^:]+):([^@]+)@([^:]+):([0-9]+)/(.+) ]]; then
    DB_USER="${BASH_REMATCH[1]}"
    DB_PASS="${BASH_REMATCH[2]}"
    DB_HOST="${BASH_REMATCH[3]}"
    DB_PORT="${BASH_REMATCH[4]}"
    DB_NAME="${BASH_REMATCH[5]}"
else
    error "Invalid database URL format"
    exit 1
fi

# List available backups
list_backups() {
    log "Available backups in $BACKUP_DIR:"
    if [[ ! -d "$BACKUP_DIR" ]]; then
        warning "Backup directory does not exist: $BACKUP_DIR"
        return
    fi
    
    find "$BACKUP_DIR" -name "*.sql*" -type f -printf "%T@ %Tc %p\n" | sort -n | while read -r timestamp date_str filepath; do
        filename=$(basename "$filepath")
        size=$(du -h "$filepath" | cut -f1)
        
        # Check for metadata file
        meta_file="${filepath}.meta"
        if [[ -f "$meta_file" ]]; then
            backup_type=$(grep '"backup_type"' "$meta_file" | cut -d'"' -f4)
            echo "  $filename ($size, $backup_type) - $date_str"
        else
            echo "  $filename ($size) - $date_str"
        fi
    done
}

if [[ "$LIST_BACKUPS" == true ]]; then
    list_backups
    exit 0
fi

# Find latest backup
if [[ "$USE_LATEST" == true ]]; then
    BACKUP_FILE=$(find "$BACKUP_DIR" -name "${DB_NAME}_*.sql*" -type f -printf "%T@ %p\n" | sort -n | tail -1 | cut -d' ' -f2-)
    if [[ -z "$BACKUP_FILE" ]]; then
        error "No backup files found for database $DB_NAME"
        exit 1
    fi
    log "Using latest backup: $(basename "$BACKUP_FILE")"
fi

# Validate backup file
if [[ -z "$BACKUP_FILE" ]]; then
    error "No backup file specified"
    show_help
    exit 1
fi

# If backup file is not an absolute path, look in backup directory
if [[ ! "$BACKUP_FILE" =~ ^/ ]]; then
    BACKUP_FILE="$BACKUP_DIR/$BACKUP_FILE"
fi

if [[ ! -f "$BACKUP_FILE" ]]; then
    error "Backup file not found: $BACKUP_FILE"
    exit 1
fi

# Check if backup is compressed
IS_COMPRESSED=false
if [[ "$BACKUP_FILE" =~ \.gz$ ]]; then
    IS_COMPRESSED=true
fi

# Build psql command
PSQL_CMD="psql"
if [[ "$USE_DOCKER" == true ]]; then
    PSQL_CMD="docker run --rm -i postgres:15 psql"
fi

PSQL_ARGS=()
PSQL_ARGS+=("--host=$DB_HOST")
PSQL_ARGS+=("--port=$DB_PORT")
PSQL_ARGS+=("--username=$DB_USER")
PSQL_ARGS+=("--no-password")

if [[ "$NO_OWNER" == true ]]; then
    PSQL_ARGS+=("--no-owner")
fi

if [[ "$NO_PRIVILEGES" == true ]]; then
    PSQL_ARGS+=("--no-privileges")
fi

# Set environment variables
export PGPASSWORD="$DB_PASS"

log "Starting VividShift database restore..."
log "Database: $DB_NAME@$DB_HOST:$DB_PORT"
log "Backup file: $BACKUP_FILE"
log "Compressed: $IS_COMPRESSED"
log "Drop database: $DROP_DATABASE"

# Show backup metadata if available
METADATA_FILE="${BACKUP_FILE}.meta"
if [[ -f "$METADATA_FILE" ]]; then
    log "Backup metadata:"
    cat "$METADATA_FILE" | jq . 2>/dev/null || cat "$METADATA_FILE"
fi

# Confirmation prompt
if [[ "$SKIP_CONFIRMATION" != true && "$DRY_RUN" != true ]]; then
    warning "This will restore the database '$DB_NAME' from backup."
    if [[ "$DROP_DATABASE" == true ]]; then
        warning "The database will be DROPPED and recreated!"
    fi
    read -p "Are you sure you want to continue? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log "Restore cancelled by user"
        exit 0
    fi
fi

if [[ "$DRY_RUN" == true ]]; then
    log "DRY RUN - Would execute:"
    if [[ "$DROP_DATABASE" == true ]]; then
        log "DROP DATABASE IF EXISTS $DB_NAME;"
        log "CREATE DATABASE $DB_NAME;"
    fi
    if [[ "$IS_COMPRESSED" == true ]]; then
        log "gunzip -c $BACKUP_FILE | $PSQL_CMD ${PSQL_ARGS[*]} $DB_NAME"
    else
        log "$PSQL_CMD ${PSQL_ARGS[*]} $DB_NAME < $BACKUP_FILE"
    fi
    exit 0
fi

# Drop and recreate database if requested
if [[ "$DROP_DATABASE" == true ]]; then
    warning "Dropping and recreating database..."
    
    # Connect to postgres database to drop the target database
    POSTGRES_ARGS=("${PSQL_ARGS[@]}")
    POSTGRES_ARGS+=("postgres")
    
    log "Dropping database $DB_NAME..."
    echo "DROP DATABASE IF EXISTS $DB_NAME;" | $PSQL_CMD "${POSTGRES_ARGS[@]}"
    
    log "Creating database $DB_NAME..."
    echo "CREATE DATABASE $DB_NAME;" | $PSQL_CMD "${POSTGRES_ARGS[@]}"
    
    success "Database recreated"
fi

# Verify database exists and is accessible
log "Verifying database connection..."
if ! echo "SELECT 1;" | $PSQL_CMD "${PSQL_ARGS[@]}" "$DB_NAME" >/dev/null 2>&1; then
    error "Cannot connect to database $DB_NAME"
    exit 1
fi
success "Database connection verified"

# Perform the restore
log "Starting restore operation..."
RESTORE_START_TIME=$(date +%s)

if [[ "$IS_COMPRESSED" == true ]]; then
    log "Decompressing and restoring..."
    if gunzip -c "$BACKUP_FILE" | $PSQL_CMD "${PSQL_ARGS[@]}" "$DB_NAME"; then
        success "Restore completed successfully"
    else
        error "Restore failed"
        exit 1
    fi
else
    log "Restoring from uncompressed backup..."
    if $PSQL_CMD "${PSQL_ARGS[@]}" "$DB_NAME" < "$BACKUP_FILE"; then
        success "Restore completed successfully"
    else
        error "Restore failed"
        exit 1
    fi
fi

RESTORE_END_TIME=$(date +%s)
RESTORE_DURATION=$((RESTORE_END_TIME - RESTORE_START_TIME))

success "Restore completed in ${RESTORE_DURATION} seconds"

# Verify restore
log "Verifying restored database..."

# Check if key tables exist
EXPECTED_TABLES=("users" "participants" "assignment_targets" "assignments")
for table in "${EXPECTED_TABLES[@]}"; do
    if echo "SELECT 1 FROM $table LIMIT 1;" | $PSQL_CMD "${PSQL_ARGS[@]}" "$DB_NAME" >/dev/null 2>&1; then
        log "✓ Table '$table' exists and is accessible"
    else
        warning "⚠ Table '$table' may not exist or is not accessible"
    fi
done

# Get basic statistics
log "Database statistics after restore:"
echo "
SELECT 
    schemaname,
    tablename,
    n_live_tup as row_count
FROM pg_stat_user_tables 
WHERE n_live_tup > 0
ORDER BY n_live_tup DESC;
" | $PSQL_CMD "${PSQL_ARGS[@]}" "$DB_NAME" 2>/dev/null || warning "Could not retrieve statistics"

success "Database restore process completed successfully"
log "Database '$DB_NAME' has been restored from: $(basename "$BACKUP_FILE")"
