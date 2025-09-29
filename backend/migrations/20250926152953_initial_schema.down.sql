-- Rollback VividShift Initial Database Schema
-- This migration completely removes all objects created in the up migration

-- Drop all indexes first (to avoid dependency issues)
DROP INDEX IF EXISTS idx_assignments_configuration_gin;
DROP INDEX IF EXISTS idx_assignment_targets_required_skills_gin;
DROP INDEX IF EXISTS idx_participants_availability_gin;
DROP INDEX IF EXISTS idx_participants_skills_gin;
DROP INDEX IF EXISTS idx_entities_metadata_gin;
DROP INDEX IF EXISTS idx_entities_attributes_gin;

DROP INDEX IF EXISTS idx_assignment_history_performed_at;
DROP INDEX IF EXISTS idx_assignment_history_assignment_id;
DROP INDEX IF EXISTS idx_assignment_details_target_id;
DROP INDEX IF EXISTS idx_assignment_details_participant_id;
DROP INDEX IF EXISTS idx_assignment_details_assignment_id;
DROP INDEX IF EXISTS idx_assignments_created_at;
DROP INDEX IF EXISTS idx_assignments_created_by;
DROP INDEX IF EXISTS idx_assignments_status;
DROP INDEX IF EXISTS idx_assignments_strategy_used;
DROP INDEX IF EXISTS idx_assignments_assignment_date;

DROP INDEX IF EXISTS idx_assignment_targets_active;
DROP INDEX IF EXISTS idx_assignment_targets_name;
DROP INDEX IF EXISTS idx_participants_created_at;
DROP INDEX IF EXISTS idx_participants_active;
DROP INDEX IF EXISTS idx_participants_name;

DROP INDEX IF EXISTS idx_entities_created_at;
DROP INDEX IF EXISTS idx_entities_created_by;
DROP INDEX IF EXISTS idx_entities_status;
DROP INDEX IF EXISTS idx_entities_entity_type;
DROP INDEX IF EXISTS idx_entities_entity_definition_id;
DROP INDEX IF EXISTS idx_entity_definitions_name;
DROP INDEX IF EXISTS idx_entity_definitions_domain_id;
DROP INDEX IF EXISTS idx_domains_active;
DROP INDEX IF EXISTS idx_domains_name;

DROP INDEX IF EXISTS idx_user_profiles_user_id;
DROP INDEX IF EXISTS idx_user_sessions_expires_at;
DROP INDEX IF EXISTS idx_user_sessions_token_hash;
DROP INDEX IF EXISTS idx_user_sessions_user_id;
DROP INDEX IF EXISTS idx_users_active;
DROP INDEX IF EXISTS idx_users_role;
DROP INDEX IF EXISTS idx_users_email;
DROP INDEX IF EXISTS idx_users_username;

-- Drop all triggers
DROP TRIGGER IF EXISTS update_rule_configurations_updated_at ON rule_configurations;
DROP TRIGGER IF EXISTS update_assignments_updated_at ON assignments;
DROP TRIGGER IF EXISTS update_assignment_targets_updated_at ON assignment_targets;
DROP TRIGGER IF EXISTS update_participants_updated_at ON participants;
DROP TRIGGER IF EXISTS update_entities_updated_at ON entities;
DROP TRIGGER IF EXISTS update_entity_definitions_updated_at ON entity_definitions;
DROP TRIGGER IF EXISTS update_domains_updated_at ON domains;
DROP TRIGGER IF EXISTS update_user_profiles_updated_at ON user_profiles;
DROP TRIGGER IF EXISTS update_user_sessions_updated_at ON user_sessions;
DROP TRIGGER IF EXISTS update_users_updated_at ON users;

-- Drop all foreign key constraints explicitly (PostgreSQL should handle this with CASCADE, but being explicit)
ALTER TABLE IF EXISTS assignment_history DROP CONSTRAINT IF EXISTS assignment_history_assignment_id_fkey;
ALTER TABLE IF EXISTS assignment_history DROP CONSTRAINT IF EXISTS assignment_history_performed_by_fkey;
ALTER TABLE IF EXISTS assignment_details DROP CONSTRAINT IF EXISTS assignment_details_assignment_id_fkey;
ALTER TABLE IF EXISTS assignment_details DROP CONSTRAINT IF EXISTS assignment_details_participant_id_fkey;
ALTER TABLE IF EXISTS assignment_details DROP CONSTRAINT IF EXISTS assignment_details_target_id_fkey;
ALTER TABLE IF EXISTS assignments DROP CONSTRAINT IF EXISTS assignments_created_by_fkey;
ALTER TABLE IF EXISTS assignment_targets DROP CONSTRAINT IF EXISTS assignment_targets_created_by_fkey;
ALTER TABLE IF EXISTS participants DROP CONSTRAINT IF EXISTS participants_created_by_fkey;
ALTER TABLE IF EXISTS entities DROP CONSTRAINT IF EXISTS entities_entity_definition_id_fkey;
ALTER TABLE IF EXISTS entities DROP CONSTRAINT IF EXISTS entities_created_by_fkey;
ALTER TABLE IF EXISTS entities DROP CONSTRAINT IF EXISTS entities_updated_by_fkey;
ALTER TABLE IF EXISTS entity_definitions DROP CONSTRAINT IF EXISTS entity_definitions_domain_id_fkey;
ALTER TABLE IF EXISTS entity_definitions DROP CONSTRAINT IF EXISTS entity_definitions_created_by_fkey;
ALTER TABLE IF EXISTS rule_configurations DROP CONSTRAINT IF EXISTS rule_configurations_domain_id_fkey;
ALTER TABLE IF EXISTS rule_configurations DROP CONSTRAINT IF EXISTS rule_configurations_created_by_fkey;
ALTER TABLE IF EXISTS domains DROP CONSTRAINT IF EXISTS domains_created_by_fkey;
ALTER TABLE IF EXISTS user_profiles DROP CONSTRAINT IF EXISTS user_profiles_user_id_fkey;
ALTER TABLE IF EXISTS user_sessions DROP CONSTRAINT IF EXISTS user_sessions_user_id_fkey;

-- Drop all tables in correct dependency order
DROP TABLE IF EXISTS assignment_history CASCADE;
DROP TABLE IF EXISTS assignment_details CASCADE;
DROP TABLE IF EXISTS assignments CASCADE;
DROP TABLE IF EXISTS assignment_targets CASCADE;
DROP TABLE IF EXISTS participants CASCADE;
DROP TABLE IF EXISTS entities CASCADE;
DROP TABLE IF EXISTS entity_definitions CASCADE;
DROP TABLE IF EXISTS rule_configurations CASCADE;
DROP TABLE IF EXISTS domains CASCADE;
DROP TABLE IF EXISTS user_profiles CASCADE;
DROP TABLE IF EXISTS user_sessions CASCADE;
DROP TABLE IF EXISTS users CASCADE;

-- Drop all functions
DROP FUNCTION IF EXISTS update_updated_at_column() CASCADE;

-- Drop all sequences (if any were created)
-- PostgreSQL automatically creates sequences for SERIAL columns, but we use UUID defaults

-- Verify no residual objects remain
DO $$
DECLARE
    r RECORD;
BEGIN
    -- Check for any remaining tables that start with our prefixes
    FOR r IN SELECT tablename FROM pg_tables WHERE schemaname = 'public' AND 
             (tablename LIKE 'users%' OR tablename LIKE 'assignment%' OR 
              tablename LIKE 'participant%' OR tablename LIKE 'domain%' OR 
              tablename LIKE 'entit%' OR tablename LIKE 'rule%')
    LOOP
        RAISE NOTICE 'WARNING: Table % still exists after migration rollback', r.tablename;
    END LOOP;
    
    -- Check for any remaining functions
    FOR r IN SELECT proname FROM pg_proc p JOIN pg_namespace n ON p.pronamespace = n.oid 
             WHERE n.nspname = 'public' AND proname LIKE '%updated_at%'
    LOOP
        RAISE NOTICE 'WARNING: Function % still exists after migration rollback', r.proname;
    END LOOP;
END $$;

-- Note: We don't drop extensions as they might be used by other applications
-- If you need to drop them in a clean environment, uncomment these lines:
-- DROP EXTENSION IF EXISTS "btree_gin";
-- DROP EXTENSION IF EXISTS "uuid-ossp";
