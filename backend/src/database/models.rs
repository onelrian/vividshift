use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, types::ipnetwork::IpNetwork};
use uuid::Uuid;
use validator::Validate;

// ============================================================================
// AUTHENTICATION MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbUserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub user_agent: Option<String>,
    pub ip_address: Option<IpNetwork>,
    pub is_revoked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbUserProfile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub avatar_url: Option<String>,
    pub timezone: Option<String>,
    pub preferences: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// BUSINESS LOGIC MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbDomain {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub version: String,
    pub is_active: bool,
    pub configuration: serde_json::Value,
    pub business_rules: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbEntityDefinition {
    pub id: Uuid,
    pub domain_id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub version: String,
    pub fields: serde_json::Value,
    pub relationships: serde_json::Value,
    pub constraints: serde_json::Value,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbEntity {
    pub id: Uuid,
    pub entity_definition_id: Uuid,
    pub entity_type: String,
    pub attributes: serde_json::Value,
    pub metadata: serde_json::Value,
    pub status: String,
    pub version: String,
    pub tags: Option<Vec<String>>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbParticipant {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub skills: serde_json::Value,
    pub availability: serde_json::Value,
    pub preferences: serde_json::Value,
    pub metadata: serde_json::Value,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbAssignmentTarget {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub required_count: i32,
    pub required_skills: serde_json::Value,
    pub constraints: serde_json::Value,
    pub metadata: serde_json::Value,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbAssignment {
    pub id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub assignment_date: NaiveDate,
    pub strategy_used: String,
    pub configuration: serde_json::Value,
    pub metadata: serde_json::Value,
    pub status: String,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbAssignmentDetail {
    pub id: Uuid,
    pub assignment_id: Uuid,
    pub participant_id: Uuid,
    pub target_id: Uuid,
    pub position: Option<i32>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbAssignmentHistory {
    pub id: Uuid,
    pub assignment_id: Uuid,
    pub action: String,
    pub changes: serde_json::Value,
    pub performed_by: Option<Uuid>,
    pub performed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbRuleConfiguration {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub domain_id: Option<Uuid>,
    pub strategy_type: String,
    pub configuration: serde_json::Value,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// CREATE/UPDATE STRUCTS
// ============================================================================

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUser {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    pub password_hash: String,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUser {
    #[validate(length(min = 3, max = 50))]
    pub username: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub role: Option<String>,
    pub is_active: Option<bool>,
    pub email_verified: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateParticipant {
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    #[validate(email)]
    pub email: Option<String>,
    pub phone: Option<String>,
    pub skills: Option<serde_json::Value>,
    pub availability: Option<serde_json::Value>,
    pub preferences: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateAssignmentTarget {
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    pub description: Option<String>,
    #[validate(range(min = 1))]
    pub required_count: i32,
    pub required_skills: Option<serde_json::Value>,
    pub constraints: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateAssignment {
    pub name: Option<String>,
    pub description: Option<String>,
    pub assignment_date: Option<NaiveDate>,
    pub strategy_used: String,
    pub configuration: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

// ============================================================================
// CONVERSION IMPLEMENTATIONS
// ============================================================================

impl From<DbUser> for crate::auth::models::User {
    fn from(db_user: DbUser) -> Self {
        use crate::auth::models::UserRole;
        
        let role = match db_user.role.as_str() {
            "admin" => UserRole::Admin,
            "viewer" => UserRole::Viewer,
            _ => UserRole::User,
        };

        Self {
            id: db_user.id,
            username: db_user.username,
            email: db_user.email,
            password_hash: db_user.password_hash,
            role,
            created_at: db_user.created_at,
            updated_at: db_user.updated_at,
        }
    }
}

impl From<crate::auth::models::User> for DbUser {
    fn from(user: crate::auth::models::User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            password_hash: user.password_hash,
            role: user.role.to_string(),
            is_active: true,
            email_verified: false,
            last_login_at: None,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
