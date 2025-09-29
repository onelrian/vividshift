use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::database::models::{DbParticipant, CreateParticipant};

#[derive(Clone)]
pub struct ParticipantRepository {
    pool: PgPool,
}

impl ParticipantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find participant by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<DbParticipant>> {
        let participant = sqlx::query_as!(
            DbParticipant,
            "SELECT * FROM participants WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(participant)
    }

    /// Find participant by name
    pub async fn find_by_name(&self, name: &str) -> Result<Option<DbParticipant>> {
        let participant = sqlx::query_as!(
            DbParticipant,
            "SELECT * FROM participants WHERE name = $1",
            name
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(participant)
    }

    /// Find participant by email
    pub async fn find_by_email(&self, email: &str) -> Result<Option<DbParticipant>> {
        let participant = sqlx::query_as!(
            DbParticipant,
            "SELECT * FROM participants WHERE email = $1",
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(participant)
    }

    /// Create a new participant
    pub async fn create(&self, participant: &CreateParticipant, created_by: Option<Uuid>) -> Result<DbParticipant> {
        let default_skills = serde_json::json!([]);
        let default_availability = serde_json::json!({});
        let default_preferences = serde_json::json!({});
        let default_metadata = serde_json::json!({});
        
        let new_participant = sqlx::query_as!(
            DbParticipant,
            r#"
            INSERT INTO participants (name, email, phone, skills, availability, preferences, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            participant.name,
            participant.email,
            participant.phone,
            participant.skills.as_ref().unwrap_or(&default_skills),
            participant.availability.as_ref().unwrap_or(&default_availability),
            participant.preferences.as_ref().unwrap_or(&default_preferences),
            participant.metadata.as_ref().unwrap_or(&default_metadata),
            created_by
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(new_participant)
    }

    /// Update participant
    pub async fn update(&self, id: Uuid, participant: &CreateParticipant, updated_by: Option<Uuid>) -> Result<DbParticipant> {
        let default_skills = serde_json::json!([]);
        let default_availability = serde_json::json!({});
        let default_preferences = serde_json::json!({});
        let default_metadata = serde_json::json!({});
        
        let updated_participant = sqlx::query_as!(
            DbParticipant,
            r#"
            UPDATE participants 
            SET name = $2,
                email = $3,
                phone = $4,
                skills = $5,
                availability = $6,
                preferences = $7,
                metadata = $8,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
            id,
            participant.name,
            participant.email,
            participant.phone,
            participant.skills.as_ref().unwrap_or(&default_skills),
            participant.availability.as_ref().unwrap_or(&default_availability),
            participant.preferences.as_ref().unwrap_or(&default_preferences),
            participant.metadata.as_ref().unwrap_or(&default_metadata)
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_participant)
    }

    /// Delete participant (soft delete)
    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query!(
            "UPDATE participants SET is_active = false WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get all active participants
    pub async fn find_all_active(&self) -> Result<Vec<DbParticipant>> {
        let participants = sqlx::query_as!(
            DbParticipant,
            "SELECT * FROM participants WHERE is_active = true ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(participants)
    }

    /// Find participants by skill
    pub async fn find_by_skill(&self, skill: &str) -> Result<Vec<DbParticipant>> {
        let participants = sqlx::query_as!(
            DbParticipant,
            r#"
            SELECT * FROM participants 
            WHERE is_active = true 
            AND skills ? $1
            ORDER BY name
            "#,
            skill
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(participants)
    }

    /// Find available participants for a specific date/time
    pub async fn find_available(&self, date_key: &str) -> Result<Vec<DbParticipant>> {
        let participants = sqlx::query_as!(
            DbParticipant,
            r#"
            SELECT * FROM participants 
            WHERE is_active = true 
            AND (availability->$1 IS NULL OR availability->$1 = 'true'::jsonb)
            ORDER BY name
            "#,
            date_key
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(participants)
    }

    /// Search participants by name (case-insensitive)
    pub async fn search_by_name(&self, name_pattern: &str) -> Result<Vec<DbParticipant>> {
        let participants = sqlx::query_as!(
            DbParticipant,
            r#"
            SELECT * FROM participants 
            WHERE is_active = true 
            AND name ILIKE $1
            ORDER BY name
            "#,
            format!("%{}%", name_pattern)
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(participants)
    }

    /// Count total active participants
    pub async fn count_active(&self) -> Result<i64> {
        let count = sqlx::query!(
            "SELECT COUNT(*) as count FROM participants WHERE is_active = true"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count.count.unwrap_or(0))
    }

    /// Bulk create participants
    pub async fn bulk_create(&self, participants: &[CreateParticipant], created_by: Option<Uuid>) -> Result<Vec<DbParticipant>> {
        let mut created_participants = Vec::new();
        
        for participant in participants {
            let created = self.create(participant, created_by).await?;
            created_participants.push(created);
        }

        Ok(created_participants)
    }

    /// Update participant skills
    pub async fn update_skills(&self, id: Uuid, skills: &serde_json::Value) -> Result<DbParticipant> {
        let updated_participant = sqlx::query_as!(
            DbParticipant,
            r#"
            UPDATE participants 
            SET skills = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
            id,
            skills
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_participant)
    }

    /// Update participant availability
    pub async fn update_availability(&self, id: Uuid, availability: &serde_json::Value) -> Result<DbParticipant> {
        let updated_participant = sqlx::query_as!(
            DbParticipant,
            r#"
            UPDATE participants 
            SET availability = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
            id,
            availability
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_participant)
    }
}
