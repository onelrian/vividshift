#!/bin/bash

# VividShift Database Backup Script
# This script creates backups of the PostgreSQL database with rotation

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BACKUP_DIR="${PROJECT_ROOT}/data/backups"
LOG_FILE="${PROJECT_ROOT}/logs/backup.log"

# Default values
DB_URL="${VIVIDSHIFT_DATABASE_URL:-postgresql://postgres:password@localhost:5432/vividshift_dev}"
BACKUP_RETENTION_DAYS=7
COMPRESS=true
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

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
VividShift Database Backup Script

Usage: $0 [OPTIONS]

Options:
    -h, --help              Show this help message
    -d, --database URL      Database URL (default: from VIVIDSHIFT_DATABASE_URL)
    -o, --output DIR        Backup output directory (default: ./data/backups)
    -r, --retention DAYS    Backup retention in days (default: 7)
    -n, --no-compress       Don't compress the backup
    -s, --schema-only       Backup schema only (no data)
    -t, --data-only         Backup data only (no schema)
    --docker                Use Docker to run pg_dump
    --dry-run               Show what would be done without executing

Examples:
    $0                                          # Basic backup
    $0 -d postgresql://user:pass@host/db        # Custom database URL
    $0 -o /custom/backup/path                   # Custom output directory
    $0 --schema-only                            # Schema only backup
    $0 --docker                                 # Use Docker for pg_dump

EOF
}

# Parse command line arguments
SCHEMA_ONLY=false
DATA_ONLY=false
USE_DOCKER=false
DRY_RUN=false

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
        -o|--output)
            BACKUP_DIR="$2"
            shift 2
            ;;
        -r|--retention)
            BACKUP_RETENTION_DAYS="$2"
            shift 2
            ;;
        -n|--no-compress)
            COMPRESS=false
            shift
            ;;
        -s|--schema-only)
            SCHEMA_ONLY=true
            shift
            ;;
        -t|--data-only)
            DATA_ONLY=true
            shift
            ;;
        --docker)
            USE_DOCKER=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        *)
            error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Validate inputs
if [[ "$SCHEMA_ONLY" == true && "$DATA_ONLY" == true ]]; then
    error "Cannot specify both --schema-only and --data-only"
    exit 1
fi

# Create necessary directories
mkdir -p "$BACKUP_DIR"
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

# Determine backup filename
if [[ "$SCHEMA_ONLY" == true ]]; then
    BACKUP_TYPE="schema"
elif [[ "$DATA_ONLY" == true ]]; then
    BACKUP_TYPE="data"
else
    BACKUP_TYPE="full"
fi

BACKUP_FILENAME="${DB_NAME}_${BACKUP_TYPE}_${TIMESTAMP}.sql"
if [[ "$COMPRESS" == true ]]; then
    BACKUP_FILENAME="${BACKUP_FILENAME}.gz"
fi

BACKUP_PATH="${BACKUP_DIR}/${BACKUP_FILENAME}"

# Build pg_dump command
PG_DUMP_CMD="pg_dump"
if [[ "$USE_DOCKER" == true ]]; then
    PG_DUMP_CMD="docker run --rm -i postgres:15 pg_dump"
fi

PG_DUMP_ARGS=()
PG_DUMP_ARGS+=("--host=$DB_HOST")
PG_DUMP_ARGS+=("--port=$DB_PORT")
PG_DUMP_ARGS+=("--username=$DB_USER")
PG_DUMP_ARGS+=("--no-password")
PG_DUMP_ARGS+=("--verbose")
PG_DUMP_ARGS+=("--clean")
PG_DUMP_ARGS+=("--if-exists")

if [[ "$SCHEMA_ONLY" == true ]]; then
    PG_DUMP_ARGS+=("--schema-only")
elif [[ "$DATA_ONLY" == true ]]; then
    PG_DUMP_ARGS+=("--data-only")
fi

PG_DUMP_ARGS+=("$DB_NAME")

# Set environment variables for pg_dump
export PGPASSWORD="$DB_PASS"

log "Starting VividShift database backup..."
log "Database: $DB_NAME@$DB_HOST:$DB_PORT"
log "Backup type: $BACKUP_TYPE"
log "Output: $BACKUP_PATH"
log "Compression: $COMPRESS"

if [[ "$DRY_RUN" == true ]]; then
    log "DRY RUN - Would execute:"
    if [[ "$COMPRESS" == true ]]; then
        log "$PG_DUMP_CMD ${PG_DUMP_ARGS[*]} | gzip > $BACKUP_PATH"
    else
        log "$PG_DUMP_CMD ${PG_DUMP_ARGS[*]} > $BACKUP_PATH"
    fi
    exit 0
fi

# Perform the backup
log "Executing backup command..."
if [[ "$COMPRESS" == true ]]; then
    if $PG_DUMP_CMD "${PG_DUMP_ARGS[@]}" | gzip > "$BACKUP_PATH"; then
        success "Backup completed successfully"
    else
        error "Backup failed"
        exit 1
    fi
else
    if $PG_DUMP_CMD "${PG_DUMP_ARGS[@]}" > "$BACKUP_PATH"; then
        success "Backup completed successfully"
    else
        error "Backup failed"
        exit 1
    fi
fi

# Get backup file size
BACKUP_SIZE=$(du -h "$BACKUP_PATH" | cut -f1)
log "Backup size: $BACKUP_SIZE"

# Clean up old backups
if [[ $BACKUP_RETENTION_DAYS -gt 0 ]]; then
    log "Cleaning up backups older than $BACKUP_RETENTION_DAYS days..."
    find "$BACKUP_DIR" -name "${DB_NAME}_*.sql*" -type f -mtime +$BACKUP_RETENTION_DAYS -delete
    success "Old backups cleaned up"
fi

# Create backup metadata
METADATA_FILE="${BACKUP_PATH}.meta"
cat > "$METADATA_FILE" << EOF
{
    "database": "$DB_NAME",
    "host": "$DB_HOST",
    "port": $DB_PORT,
    "backup_type": "$BACKUP_TYPE",
    "timestamp": "$TIMESTAMP",
    "compressed": $COMPRESS,
    "size": "$BACKUP_SIZE",
    "pg_dump_version": "$(pg_dump --version | head -n1)",
    "created_by": "$(whoami)",
    "hostname": "$(hostname)"
}
EOF

success "Backup completed: $BACKUP_PATH"
log "Metadata saved: $METADATA_FILE"

# Verify backup integrity
log "Verifying backup integrity..."
if [[ "$COMPRESS" == true ]]; then
    if gzip -t "$BACKUP_PATH"; then
        success "Backup integrity verified"
    else
        error "Backup integrity check failed"
        exit 1
    fi
else
    if [[ -s "$BACKUP_PATH" ]]; then
        success "Backup integrity verified"
    else
        error "Backup file is empty"
        exit 1
    fi
fi

log "Backup process completed successfully"
