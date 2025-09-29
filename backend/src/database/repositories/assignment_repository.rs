use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::NaiveDate;

use crate::database::models::{
    DbAssignment, DbAssignmentTarget, DbAssignmentDetail, DbAssignmentHistory,
    CreateAssignment, CreateAssignmentTarget
};

#[derive(Clone)]
pub struct AssignmentRepository {
    pool: PgPool,
}

impl AssignmentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ============================================================================
    // ASSIGNMENT TARGET OPERATIONS
    // ============================================================================

    /// Find assignment target by ID
    pub async fn find_target_by_id(&self, id: Uuid) -> Result<Option<DbAssignmentTarget>> {
        let target = sqlx::query_as!(
            DbAssignmentTarget,
            "SELECT * FROM assignment_targets WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(target)
    }

    /// Find assignment target by name
    pub async fn find_target_by_name(&self, name: &str) -> Result<Option<DbAssignmentTarget>> {
        let target = sqlx::query_as!(
            DbAssignmentTarget,
            "SELECT * FROM assignment_targets WHERE name = $1",
            name
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(target)
    }

    /// Create assignment target
    pub async fn create_target(&self, target: &CreateAssignmentTarget, created_by: Option<Uuid>) -> Result<DbAssignmentTarget> {
        let default_skills = serde_json::json!([]);
        let default_constraints = serde_json::json!({});
        let default_metadata = serde_json::json!({});
        
        let new_target = sqlx::query_as!(
            DbAssignmentTarget,
            r#"
            INSERT INTO assignment_targets (name, description, required_count, required_skills, constraints, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
            target.name,
            target.description,
            target.required_count,
            target.required_skills.as_ref().unwrap_or(&default_skills),
            target.constraints.as_ref().unwrap_or(&default_constraints),
            target.metadata.as_ref().unwrap_or(&default_metadata),
            created_by
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(new_target)
    }

    /// Get all active assignment targets
    pub async fn find_all_active_targets(&self) -> Result<Vec<DbAssignmentTarget>> {
        let targets = sqlx::query_as!(
            DbAssignmentTarget,
            "SELECT * FROM assignment_targets WHERE is_active = true ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(targets)
    }

    /// Update assignment target
    pub async fn update_target(&self, id: Uuid, target: &CreateAssignmentTarget) -> Result<DbAssignmentTarget> {
        let default_skills = serde_json::json!([]);
        let default_constraints = serde_json::json!({});
        let default_metadata = serde_json::json!({});
        
        let updated_target = sqlx::query_as!(
            DbAssignmentTarget,
            r#"
            UPDATE assignment_targets 
            SET name = $2,
                description = $3,
                required_count = $4,
                required_skills = $5,
                constraints = $6,
                metadata = $7,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
            id,
            target.name,
            target.description,
            target.required_count,
            target.required_skills.as_ref().unwrap_or(&default_skills),
            target.constraints.as_ref().unwrap_or(&default_constraints),
            target.metadata.as_ref().unwrap_or(&default_metadata)
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_target)
    }

    /// Delete assignment target (soft delete)
    pub async fn delete_target(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query!(
            "UPDATE assignment_targets SET is_active = false WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // ============================================================================
    // ASSIGNMENT OPERATIONS
    // ============================================================================

    /// Find assignment by ID
    pub async fn find_assignment_by_id(&self, id: Uuid) -> Result<Option<DbAssignment>> {
        let assignment = sqlx::query_as!(
            DbAssignment,
            "SELECT * FROM assignments WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(assignment)
    }

    /// Create assignment
    pub async fn create_assignment(&self, assignment: &CreateAssignment, created_by: Option<Uuid>) -> Result<DbAssignment> {
        let default_config = serde_json::json!({});
        let default_metadata = serde_json::json!({});
        
        let new_assignment = sqlx::query_as!(
            DbAssignment,
            r#"
            INSERT INTO assignments (name, description, assignment_date, strategy_used, configuration, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
            assignment.name,
            assignment.description,
            assignment.assignment_date.unwrap_or_else(|| chrono::Utc::now().date_naive()),
            assignment.strategy_used,
            assignment.configuration.as_ref().unwrap_or(&default_config),
            assignment.metadata.as_ref().unwrap_or(&default_metadata),
            created_by
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(new_assignment)
    }

    /// Get assignments by date range
    pub async fn find_assignments_by_date_range(&self, start_date: NaiveDate, end_date: NaiveDate) -> Result<Vec<DbAssignment>> {
        let assignments = sqlx::query_as!(
            DbAssignment,
            r#"
            SELECT * FROM assignments 
            WHERE assignment_date >= $1 AND assignment_date <= $2 
            ORDER BY assignment_date DESC, created_at DESC
            "#,
            start_date,
            end_date
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(assignments)
    }

    /// Get recent assignments
    pub async fn find_recent_assignments(&self, limit: i64) -> Result<Vec<DbAssignment>> {
        let assignments = sqlx::query_as!(
            DbAssignment,
            "SELECT * FROM assignments ORDER BY created_at DESC LIMIT $1",
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(assignments)
    }

    /// Get assignments by strategy
    pub async fn find_assignments_by_strategy(&self, strategy: &str) -> Result<Vec<DbAssignment>> {
        let assignments = sqlx::query_as!(
            DbAssignment,
            "SELECT * FROM assignments WHERE strategy_used = $1 ORDER BY created_at DESC",
            strategy
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(assignments)
    }

    // ============================================================================
    // ASSIGNMENT DETAIL OPERATIONS
    // ============================================================================

    /// Add assignment detail
    pub async fn add_assignment_detail(
        &self,
        assignment_id: Uuid,
        participant_id: Uuid,
        target_id: Uuid,
        position: Option<i32>,
        metadata: Option<&serde_json::Value>
    ) -> Result<DbAssignmentDetail> {
        let default_metadata = serde_json::json!({});
        
        let detail = sqlx::query_as!(
            DbAssignmentDetail,
            r#"
            INSERT INTO assignment_details (assignment_id, participant_id, target_id, position, metadata)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            assignment_id,
            participant_id,
            target_id,
            position,
            metadata.unwrap_or(&default_metadata)
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(detail)
    }

    /// Get assignment details by assignment ID
    pub async fn find_assignment_details(&self, assignment_id: Uuid) -> Result<Vec<DbAssignmentDetail>> {
        let details = sqlx::query_as!(
            DbAssignmentDetail,
            "SELECT * FROM assignment_details WHERE assignment_id = $1 ORDER BY position, created_at",
            assignment_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(details)
    }

    /// Get assignment details with participant and target info
    pub async fn find_assignment_details_with_info(&self, assignment_id: Uuid) -> Result<Vec<serde_json::Value>> {
        let details = sqlx::query!(
            r#"
            SELECT 
                ad.*,
                p.name as participant_name,
                p.email as participant_email,
                at.name as target_name,
                at.description as target_description,
                at.required_count as target_required_count
            FROM assignment_details ad
            JOIN participants p ON ad.participant_id = p.id
            JOIN assignment_targets at ON ad.target_id = at.id
            WHERE ad.assignment_id = $1
            ORDER BY ad.position, ad.created_at
            "#,
            assignment_id
        )
        .fetch_all(&self.pool)
        .await?;

        let result: Vec<serde_json::Value> = details
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.id,
                    "assignment_id": row.assignment_id,
                    "participant_id": row.participant_id,
                    "target_id": row.target_id,
                    "position": row.position,
                    "metadata": row.metadata,
                    "created_at": row.created_at,
                    "participant": {
                        "name": row.participant_name,
                        "email": row.participant_email
                    },
                    "target": {
                        "name": row.target_name,
                        "description": row.target_description,
                        "required_count": row.target_required_count
                    }
                })
            })
            .collect();

        Ok(result)
    }

    /// Delete assignment details for an assignment
    pub async fn delete_assignment_details(&self, assignment_id: Uuid) -> Result<u64> {
        let result = sqlx::query!(
            "DELETE FROM assignment_details WHERE assignment_id = $1",
            assignment_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    // ============================================================================
    // ASSIGNMENT HISTORY OPERATIONS
    // ============================================================================

    /// Add assignment history entry
    pub async fn add_history_entry(
        &self,
        assignment_id: Uuid,
        action: &str,
        changes: &serde_json::Value,
        performed_by: Option<Uuid>
    ) -> Result<DbAssignmentHistory> {
        let history = sqlx::query_as!(
            DbAssignmentHistory,
            r#"
            INSERT INTO assignment_history (assignment_id, action, changes, performed_by)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
            assignment_id,
            action,
            changes,
            performed_by
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(history)
    }

    /// Get assignment history
    pub async fn find_assignment_history(&self, assignment_id: Uuid) -> Result<Vec<DbAssignmentHistory>> {
        let history = sqlx::query_as!(
            DbAssignmentHistory,
            "SELECT * FROM assignment_history WHERE assignment_id = $1 ORDER BY performed_at DESC",
            assignment_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(history)
    }

    // ============================================================================
    // BULK OPERATIONS
    // ============================================================================

    /// Bulk create assignment details
    pub async fn bulk_create_assignment_details(
        &self,
        assignment_id: Uuid,
        details: &[(Uuid, Uuid, Option<i32>)] // (participant_id, target_id, position)
    ) -> Result<Vec<DbAssignmentDetail>> {
        let mut created_details = Vec::new();

        for (participant_id, target_id, position) in details {
            let detail = self.add_assignment_detail(
                assignment_id,
                *participant_id,
                *target_id,
                *position,
                None
            ).await?;
            created_details.push(detail);
        }

        Ok(created_details)
    }

    /// Get assignment statistics
    pub async fn get_assignment_stats(&self) -> Result<serde_json::Value> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_assignments,
                COUNT(DISTINCT strategy_used) as unique_strategies,
                COUNT(CASE WHEN assignment_date >= CURRENT_DATE - INTERVAL '30 days' THEN 1 END) as recent_assignments,
                COUNT(CASE WHEN status = 'active' THEN 1 END) as active_assignments
            FROM assignments
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(serde_json::json!({
            "total_assignments": stats.total_assignments.unwrap_or(0),
            "unique_strategies": stats.unique_strategies.unwrap_or(0),
            "recent_assignments": stats.recent_assignments.unwrap_or(0),
            "active_assignments": stats.active_assignments.unwrap_or(0)
        }))
    }
}
