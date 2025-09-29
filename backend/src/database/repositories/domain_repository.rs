use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::database::models::{DbDomain, DbEntityDefinition, DbEntity, DbRuleConfiguration};

#[derive(Clone)]
pub struct DomainRepository {
    pool: PgPool,
}

impl DomainRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ============================================================================
    // DOMAIN OPERATIONS
    // ============================================================================

    /// Find domain by ID
    pub async fn find_domain_by_id(&self, id: Uuid) -> Result<Option<DbDomain>> {
        let domain = sqlx::query_as!(
            DbDomain,
            "SELECT * FROM domains WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(domain)
    }

    /// Find domain by name
    pub async fn find_domain_by_name(&self, name: &str) -> Result<Option<DbDomain>> {
        let domain = sqlx::query_as!(
            DbDomain,
            "SELECT * FROM domains WHERE name = $1",
            name
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(domain)
    }

    /// Create domain
    pub async fn create_domain(
        &self,
        name: &str,
        display_name: &str,
        description: Option<&str>,
        configuration: &serde_json::Value,
        business_rules: &serde_json::Value,
        created_by: Option<Uuid>
    ) -> Result<DbDomain> {
        let domain = sqlx::query_as!(
            DbDomain,
            r#"
            INSERT INTO domains (name, display_name, description, configuration, business_rules, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
            name,
            display_name,
            description,
            configuration,
            business_rules,
            created_by
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(domain)
    }

    /// Get all active domains
    pub async fn find_all_active_domains(&self) -> Result<Vec<DbDomain>> {
        let domains = sqlx::query_as!(
            DbDomain,
            "SELECT * FROM domains WHERE is_active = true ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(domains)
    }

    /// Update domain
    pub async fn update_domain(
        &self,
        id: Uuid,
        display_name: Option<&str>,
        description: Option<&str>,
        configuration: Option<&serde_json::Value>,
        business_rules: Option<&serde_json::Value>
    ) -> Result<DbDomain> {
        let domain = sqlx::query_as!(
            DbDomain,
            r#"
            UPDATE domains 
            SET display_name = COALESCE($2, display_name),
                description = COALESCE($3, description),
                configuration = COALESCE($4, configuration),
                business_rules = COALESCE($5, business_rules),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
            id,
            display_name,
            description,
            configuration,
            business_rules
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(domain)
    }

    // ============================================================================
    // ENTITY DEFINITION OPERATIONS
    // ============================================================================

    /// Find entity definition by ID
    pub async fn find_entity_definition_by_id(&self, id: Uuid) -> Result<Option<DbEntityDefinition>> {
        let entity_def = sqlx::query_as!(
            DbEntityDefinition,
            "SELECT * FROM entity_definitions WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity_def)
    }

    /// Find entity definition by domain and name
    pub async fn find_entity_definition_by_name(&self, domain_id: Uuid, name: &str) -> Result<Option<DbEntityDefinition>> {
        let entity_def = sqlx::query_as!(
            DbEntityDefinition,
            "SELECT * FROM entity_definitions WHERE domain_id = $1 AND name = $2",
            domain_id,
            name
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity_def)
    }

    /// Create entity definition
    pub async fn create_entity_definition(
        &self,
        domain_id: Uuid,
        name: &str,
        display_name: &str,
        description: Option<&str>,
        fields: &serde_json::Value,
        relationships: &serde_json::Value,
        constraints: &serde_json::Value,
        created_by: Option<Uuid>
    ) -> Result<DbEntityDefinition> {
        let entity_def = sqlx::query_as!(
            DbEntityDefinition,
            r#"
            INSERT INTO entity_definitions (domain_id, name, display_name, description, fields, relationships, constraints, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            domain_id,
            name,
            display_name,
            description,
            fields,
            relationships,
            constraints,
            created_by
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(entity_def)
    }

    /// Get entity definitions for a domain
    pub async fn find_entity_definitions_by_domain(&self, domain_id: Uuid) -> Result<Vec<DbEntityDefinition>> {
        let entity_defs = sqlx::query_as!(
            DbEntityDefinition,
            "SELECT * FROM entity_definitions WHERE domain_id = $1 AND is_active = true ORDER BY name",
            domain_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(entity_defs)
    }

    // ============================================================================
    // ENTITY OPERATIONS
    // ============================================================================

    /// Find entity by ID
    pub async fn find_entity_by_id(&self, id: Uuid) -> Result<Option<DbEntity>> {
        let entity = sqlx::query_as!(
            DbEntity,
            "SELECT * FROM entities WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity)
    }

    /// Create entity
    pub async fn create_entity(
        &self,
        entity_definition_id: Uuid,
        entity_type: &str,
        attributes: &serde_json::Value,
        metadata: &serde_json::Value,
        tags: &[String],
        created_by: Option<Uuid>
    ) -> Result<DbEntity> {
        let entity = sqlx::query_as!(
            DbEntity,
            r#"
            INSERT INTO entities (entity_definition_id, entity_type, attributes, metadata, tags, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
            entity_definition_id,
            entity_type,
            attributes,
            metadata,
            tags as &[String],
            created_by
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(entity)
    }

    /// Find entities by type
    pub async fn find_entities_by_type(&self, entity_type: &str) -> Result<Vec<DbEntity>> {
        let entities = sqlx::query_as!(
            DbEntity,
            "SELECT * FROM entities WHERE entity_type = $1 AND status = 'active' ORDER BY created_at DESC",
            entity_type
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(entities)
    }

    /// Find entities by definition
    pub async fn find_entities_by_definition(&self, entity_definition_id: Uuid) -> Result<Vec<DbEntity>> {
        let entities = sqlx::query_as!(
            DbEntity,
            "SELECT * FROM entities WHERE entity_definition_id = $1 AND status = 'active' ORDER BY created_at DESC",
            entity_definition_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(entities)
    }

    /// Update entity
    pub async fn update_entity(
        &self,
        id: Uuid,
        attributes: &serde_json::Value,
        metadata: &serde_json::Value,
        tags: &[String],
        updated_by: Option<Uuid>
    ) -> Result<DbEntity> {
        let entity = sqlx::query_as!(
            DbEntity,
            r#"
            UPDATE entities 
            SET attributes = $2,
                metadata = $3,
                tags = $4,
                updated_by = $5,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
            id,
            attributes,
            metadata,
            tags as &[String],
            updated_by
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(entity)
    }

    /// Delete entity (soft delete)
    pub async fn delete_entity(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query!(
            "UPDATE entities SET status = 'archived' WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Search entities by attributes
    pub async fn search_entities_by_attributes(&self, entity_type: &str, search_criteria: &serde_json::Value) -> Result<Vec<DbEntity>> {
        let entities = sqlx::query_as!(
            DbEntity,
            r#"
            SELECT * FROM entities 
            WHERE entity_type = $1 
            AND status = 'active'
            AND attributes @> $2
            ORDER BY created_at DESC
            "#,
            entity_type,
            search_criteria
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(entities)
    }

    // ============================================================================
    // RULE CONFIGURATION OPERATIONS
    // ============================================================================

    /// Find rule configuration by ID
    pub async fn find_rule_config_by_id(&self, id: Uuid) -> Result<Option<DbRuleConfiguration>> {
        let rule_config = sqlx::query_as!(
            DbRuleConfiguration,
            "SELECT * FROM rule_configurations WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(rule_config)
    }

    /// Find rule configuration by name
    pub async fn find_rule_config_by_name(&self, name: &str) -> Result<Option<DbRuleConfiguration>> {
        let rule_config = sqlx::query_as!(
            DbRuleConfiguration,
            "SELECT * FROM rule_configurations WHERE name = $1",
            name
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(rule_config)
    }

    /// Create rule configuration
    pub async fn create_rule_config(
        &self,
        name: &str,
        description: Option<&str>,
        domain_id: Option<Uuid>,
        strategy_type: &str,
        configuration: &serde_json::Value,
        created_by: Option<Uuid>
    ) -> Result<DbRuleConfiguration> {
        let rule_config = sqlx::query_as!(
            DbRuleConfiguration,
            r#"
            INSERT INTO rule_configurations (name, description, domain_id, strategy_type, configuration, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
            name,
            description,
            domain_id,
            strategy_type,
            configuration,
            created_by
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(rule_config)
    }

    /// Get active rule configurations
    pub async fn find_active_rule_configs(&self) -> Result<Vec<DbRuleConfiguration>> {
        let rule_configs = sqlx::query_as!(
            DbRuleConfiguration,
            "SELECT * FROM rule_configurations WHERE is_active = true ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rule_configs)
    }

    /// Get rule configurations by strategy type
    pub async fn find_rule_configs_by_strategy(&self, strategy_type: &str) -> Result<Vec<DbRuleConfiguration>> {
        let rule_configs = sqlx::query_as!(
            DbRuleConfiguration,
            "SELECT * FROM rule_configurations WHERE strategy_type = $1 AND is_active = true ORDER BY name",
            strategy_type
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rule_configs)
    }

    /// Get rule configurations by domain
    pub async fn find_rule_configs_by_domain(&self, domain_id: Uuid) -> Result<Vec<DbRuleConfiguration>> {
        let rule_configs = sqlx::query_as!(
            DbRuleConfiguration,
            "SELECT * FROM rule_configurations WHERE domain_id = $1 AND is_active = true ORDER BY name",
            domain_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rule_configs)
    }
}
