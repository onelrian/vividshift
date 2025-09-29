use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::database::models::{DbUser, CreateUser, UpdateUser, DbUserSession, DbUserProfile};

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find user by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<DbUser>> {
        let user = sqlx::query_as!(
            DbUser,
            "SELECT * FROM users WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find user by username
    pub async fn find_by_username(&self, username: &str) -> Result<Option<DbUser>> {
        let user = sqlx::query_as!(
            DbUser,
            "SELECT * FROM users WHERE username = $1",
            username
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find user by email
    pub async fn find_by_email(&self, email: &str) -> Result<Option<DbUser>> {
        let user = sqlx::query_as!(
            DbUser,
            "SELECT * FROM users WHERE email = $1",
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Create a new user
    pub async fn create(&self, user: &CreateUser) -> Result<DbUser> {
        let new_user = sqlx::query_as!(
            DbUser,
            r#"
            INSERT INTO users (username, email, password_hash, role)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
            user.username,
            user.email,
            user.password_hash,
            user.role.as_deref().unwrap_or("user")
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(new_user)
    }

    /// Update user
    pub async fn update(&self, id: Uuid, user: &UpdateUser) -> Result<DbUser> {
        let updated_user = sqlx::query_as!(
            DbUser,
            r#"
            UPDATE users 
            SET username = COALESCE($2, username),
                email = COALESCE($3, email),
                role = COALESCE($4, role),
                is_active = COALESCE($5, is_active),
                email_verified = COALESCE($6, email_verified),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
            id,
            user.username,
            user.email,
            user.role,
            user.is_active,
            user.email_verified
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_user)
    }

    /// Update last login timestamp
    pub async fn update_last_login(&self, id: Uuid) -> Result<()> {
        sqlx::query!(
            "UPDATE users SET last_login_at = NOW() WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete user (soft delete by setting is_active = false)
    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query!(
            "UPDATE users SET is_active = false WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Hard delete user (for testing/admin purposes)
    pub async fn hard_delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query!(
            "DELETE FROM users WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get all active users
    pub async fn find_all_active(&self) -> Result<Vec<DbUser>> {
        let users = sqlx::query_as!(
            DbUser,
            "SELECT * FROM users WHERE is_active = true ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    /// Count total users
    pub async fn count(&self) -> Result<i64> {
        let count = sqlx::query!("SELECT COUNT(*) as count FROM users")
            .fetch_one(&self.pool)
            .await?;

        Ok(count.count.unwrap_or(0))
    }

    /// Create user session
    pub async fn create_session(&self, user_id: Uuid, token_hash: &str, expires_at: DateTime<Utc>, user_agent: Option<&str>, ip_address: Option<sqlx::types::ipnetwork::IpNetwork>) -> Result<DbUserSession> {
        let session = sqlx::query_as!(
            DbUserSession,
            r#"
            INSERT INTO user_sessions (user_id, token_hash, expires_at, user_agent, ip_address)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            user_id,
            token_hash,
            expires_at,
            user_agent,
            ip_address
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(session)
    }

    /// Find session by token hash
    pub async fn find_session_by_token(&self, token_hash: &str) -> Result<Option<DbUserSession>> {
        let session = sqlx::query_as!(
            DbUserSession,
            "SELECT * FROM user_sessions WHERE token_hash = $1 AND is_revoked = false AND expires_at > NOW()",
            token_hash
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(session)
    }

    /// Revoke session
    pub async fn revoke_session(&self, token_hash: &str) -> Result<bool> {
        let result = sqlx::query!(
            "UPDATE user_sessions SET is_revoked = true WHERE token_hash = $1",
            token_hash
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Revoke all user sessions
    pub async fn revoke_all_user_sessions(&self, user_id: Uuid) -> Result<u64> {
        let result = sqlx::query!(
            "UPDATE user_sessions SET is_revoked = true WHERE user_id = $1",
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> Result<u64> {
        let result = sqlx::query!(
            "DELETE FROM user_sessions WHERE expires_at < NOW() OR is_revoked = true"
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Get or create user profile
    pub async fn get_or_create_profile(&self, user_id: Uuid) -> Result<DbUserProfile> {
        // Try to find existing profile
        if let Some(profile) = sqlx::query_as!(
            DbUserProfile,
            "SELECT * FROM user_profiles WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await? {
            return Ok(profile);
        }

        // Create new profile if not found
        let profile = sqlx::query_as!(
            DbUserProfile,
            r#"
            INSERT INTO user_profiles (user_id, preferences, metadata)
            VALUES ($1, '{}', '{}')
            RETURNING *
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(profile)
    }

    /// Update user profile
    pub async fn update_profile(&self, user_id: Uuid, first_name: Option<&str>, last_name: Option<&str>, phone: Option<&str>, timezone: Option<&str>) -> Result<DbUserProfile> {
        let profile = sqlx::query_as!(
            DbUserProfile,
            r#"
            UPDATE user_profiles 
            SET first_name = COALESCE($2, first_name),
                last_name = COALESCE($3, last_name),
                phone = COALESCE($4, phone),
                timezone = COALESCE($5, timezone),
                updated_at = NOW()
            WHERE user_id = $1
            RETURNING *
            "#,
            user_id,
            first_name,
            last_name,
            phone,
            timezone
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(profile)
    }
}
