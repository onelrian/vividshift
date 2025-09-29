pub mod user_repository;
pub mod participant_repository;
pub mod assignment_repository;
pub mod domain_repository;

pub use user_repository::*;
pub use participant_repository::*;
pub use assignment_repository::*;
pub use domain_repository::*;

use anyhow::Result;
use sqlx::PgPool;

/// Base repository trait for common operations
#[async_trait::async_trait]
pub trait Repository<T, ID> {
    async fn find_by_id(&self, id: ID) -> Result<Option<T>>;
    async fn find_all(&self) -> Result<Vec<T>>;
    async fn create(&self, entity: &T) -> Result<T>;
    async fn update(&self, id: ID, entity: &T) -> Result<T>;
    async fn delete(&self, id: ID) -> Result<bool>;
}

/// Repository manager that holds all repositories
#[derive(Clone)]
pub struct RepositoryManager {
    pub users: UserRepository,
    pub participants: ParticipantRepository,
    pub assignments: AssignmentRepository,
    pub domains: DomainRepository,
}

impl RepositoryManager {
    pub fn new(pool: PgPool) -> Self {
        Self {
            users: UserRepository::new(pool.clone()),
            participants: ParticipantRepository::new(pool.clone()),
            assignments: AssignmentRepository::new(pool.clone()),
            domains: DomainRepository::new(pool),
        }
    }
}
