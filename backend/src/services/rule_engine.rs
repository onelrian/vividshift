use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{
    Assignment, AssignmentConstraint, AssignmentResult, GenericEntity, ValidationResult,
    ValidationSeverity,
};

/// Rule engine for processing business rules and assignments
pub struct RuleEngine {
    strategies: HashMap<String, Box<dyn AssignmentStrategy + Send + Sync>>,
    validators: HashMap<String, Box<dyn ValidationRule + Send + Sync>>,
    config: RuleEngineConfig,
}

/// Configuration for the rule engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEngineConfig {
    pub default_strategy: String,
    pub max_execution_time_ms: u64,
    pub enable_parallel_processing: bool,
    pub validation_mode: ValidationMode,
}

/// Validation execution mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationMode {
    Strict,      // Fail on any validation error
    Permissive,  // Continue with warnings
    BestEffort,  // Try to fix issues automatically
}

/// Strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub name: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub constraints: Vec<AssignmentConstraint>,
    pub validation_rules: Vec<String>,
}

/// Assignment strategy trait
#[async_trait]
pub trait AssignmentStrategy {
    async fn execute(
        &self,
        participants: &[GenericEntity],
        targets: &[GenericEntity],
        config: &StrategyConfig,
    ) -> Result<Vec<Assignment>>;

    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn validate_config(&self, config: &StrategyConfig) -> Result<()>;
}

/// Validation rule trait
#[async_trait]
pub trait ValidationRule {
    async fn validate(
        &self,
        assignments: &[Assignment],
        participants: &[GenericEntity],
        targets: &[GenericEntity],
        config: &HashMap<String, serde_json::Value>,
    ) -> Result<ValidationResult>;

    fn name(&self) -> &str;
    fn severity(&self) -> ValidationSeverity;
}

/// Rule execution context
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub request_id: Uuid,
    pub user_id: Option<Uuid>,
    pub domain: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl RuleEngine {
    pub fn new(config: RuleEngineConfig) -> Self {
        Self {
            strategies: HashMap::new(),
            validators: HashMap::new(),
            config,
        }
    }

    pub fn register_strategy<S>(&mut self, strategy: S)
    where
        S: AssignmentStrategy + Send + Sync + 'static,
    {
        let name = strategy.name().to_string();
        self.strategies.insert(name, Box::new(strategy));
    }

    pub fn register_validator<V>(&mut self, validator: V)
    where
        V: ValidationRule + Send + Sync + 'static,
    {
        let name = validator.name().to_string();
        self.validators.insert(name, Box::new(validator));
    }

    pub async fn execute_assignment(
        &self,
        strategy_name: &str,
        participants: Vec<GenericEntity>,
        targets: Vec<GenericEntity>,
        config: StrategyConfig,
        context: ExecutionContext,
    ) -> Result<AssignmentResult> {
        let start_time = std::time::Instant::now();

        // Get strategy
        let strategy = self
            .strategies
            .get(strategy_name)
            .ok_or_else(|| anyhow!("Strategy '{}' not found", strategy_name))?;

        // Validate strategy configuration
        strategy.validate_config(&config)?;

        // Execute assignment
        let assignments = strategy.execute(&participants, &targets, &config).await?;

        // Run validation rules
        let validation_results = self
            .run_validations(&assignments, &participants, &targets, &config)
            .await?;

        // Check if validation passed based on mode
        self.check_validation_results(&validation_results)?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        Ok(AssignmentResult {
            id: context.request_id,
            strategy_used: strategy_name.to_string(),
            assignments,
            metadata: crate::models::AssignmentMetadata {
                attempts: 1, // TODO: Implement retry logic
                execution_time_ms: execution_time,
                strategy_parameters: config.parameters,
                validation_results,
            },
            created_at: chrono::Utc::now(),
        })
    }

    async fn run_validations(
        &self,
        assignments: &[Assignment],
        participants: &[GenericEntity],
        targets: &[GenericEntity],
        config: &StrategyConfig,
    ) -> Result<Vec<ValidationResult>> {
        let mut results = Vec::new();

        for rule_name in &config.validation_rules {
            if let Some(validator) = self.validators.get(rule_name) {
                let rule_config = config
                    .parameters
                    .get(&format!("validation_{}", rule_name))
                    .and_then(|v| v.as_object())
                    .map(|obj| {
                        obj.iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect::<HashMap<String, serde_json::Value>>()
                    })
                    .unwrap_or_default();

                match validator
                    .validate(assignments, participants, targets, &rule_config)
                    .await
                {
                    Ok(result) => results.push(result),
                    Err(e) => {
                        tracing::error!("Validation rule '{}' failed: {}", rule_name, e);
                        results.push(ValidationResult {
                            rule_name: rule_name.clone(),
                            passed: false,
                            message: Some(format!("Validation error: {}", e)),
                            severity: ValidationSeverity::Error,
                        });
                    }
                }
            } else {
                tracing::warn!("Validation rule '{}' not found", rule_name);
            }
        }

        Ok(results)
    }

    fn check_validation_results(&self, results: &[ValidationResult]) -> Result<()> {
        match self.config.validation_mode {
            ValidationMode::Strict => {
                for result in results {
                    if !result.passed
                        && (result.severity == ValidationSeverity::Error
                            || result.severity == ValidationSeverity::Critical)
                    {
                        return Err(anyhow!(
                            "Validation failed: {} - {}",
                            result.rule_name,
                            result.message.as_deref().unwrap_or("Unknown error")
                        ));
                    }
                }
            }
            ValidationMode::Permissive => {
                // Log warnings but continue
                for result in results {
                    if !result.passed {
                        tracing::warn!(
                            "Validation warning: {} - {}",
                            result.rule_name,
                            result.message.as_deref().unwrap_or("Unknown warning")
                        );
                    }
                }
            }
            ValidationMode::BestEffort => {
                // TODO: Implement automatic fixing logic
                tracing::info!("Running in best-effort mode, attempting to fix issues");
            }
        }

        Ok(())
    }

    pub fn list_strategies(&self) -> Vec<String> {
        self.strategies.keys().cloned().collect()
    }

    pub fn list_validators(&self) -> Vec<String> {
        self.validators.keys().cloned().collect()
    }
}

impl Default for RuleEngineConfig {
    fn default() -> Self {
        Self {
            default_strategy: "balanced_rotation".to_string(),
            max_execution_time_ms: 30000, // 30 seconds
            enable_parallel_processing: true,
            validation_mode: ValidationMode::Strict,
        }
    }
}
