//! Workflow state machine for the multi-agent factory.
//!
//! Defines the phased execution model: Analysis → ParallelBuild →
//! IntegrationLoop → Deployment → Completed/Failed.

use super::blackboard::{Artifact, Blackboard};
use super::roles::{AgentRole, RoleConfig};
use crate::agent::loop_::run_tool_call_loop;
use crate::config::DelegateAgentConfig;
use crate::observability::traits::{Observer, ObserverEvent, ObserverMetric};
use crate::providers::{self, ChatMessage, Provider};
use crate::tools::traits::Tool;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

/// Workflow phases in the factory pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowPhase {
    /// BA agent analyzes user idea and writes PRD.
    Analysis,
    /// UI/UX, Developer, and Tester agents run concurrently.
    ParallelBuild,
    /// Developer-Tester ping-pong loop until tests pass.
    IntegrationLoop,
    /// DevOps agent deploys the project.
    Deployment,
    /// Workflow completed successfully.
    Completed,
    /// Workflow failed.
    Failed,
}

/// Factory workflow orchestrator.
///
/// Manages the lifecycle of a multi-agent build session, coordinating
/// specialized agents through phased execution.
pub struct FactoryWorkflow {
    blackboard: Blackboard,
    idea: String,
    max_ping_pong: usize,
    phase: WorkflowPhase,
    /// Provider/model overrides per role (from factory config).
    role_overrides: HashMap<String, DelegateAgentConfig>,
    /// Global provider config for creating providers.
    provider_runtime_options: providers::ProviderRuntimeOptions,
    /// Fallback API credential.
    fallback_credential: Option<String>,
    /// Default provider name when no override is set.
    default_provider: String,
    /// Default model name when no override is set.
    default_model: String,
    /// Parent tool registry for agentic agent runs.
    parent_tools: Arc<Vec<Arc<dyn Tool>>>,
    /// Multimodal config for agent loops.
    multimodal_config: crate::config::MultimodalConfig,
}

impl FactoryWorkflow {
    /// Create a new factory workflow for the given idea.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        idea: String,
        max_ping_pong: usize,
        role_overrides: HashMap<String, DelegateAgentConfig>,
        provider_runtime_options: providers::ProviderRuntimeOptions,
        fallback_credential: Option<String>,
        default_provider: String,
        default_model: String,
        parent_tools: Arc<Vec<Arc<dyn Tool>>>,
        multimodal_config: crate::config::MultimodalConfig,
    ) -> Self {
        Self {
            blackboard: Blackboard::new(),
            idea,
            max_ping_pong,
            phase: WorkflowPhase::Analysis,
            role_overrides,
            provider_runtime_options,
            fallback_credential,
            default_provider,
            default_model,
            parent_tools,
            multimodal_config,
        }
    }

    /// Execute the full factory workflow, returning a summary of the result.
    pub async fn run(&mut self) -> Result<String> {
        // Phase 1: Analysis
        self.phase = WorkflowPhase::Analysis;
        let prd = self
            .run_agent_simple(
                AgentRole::BusinessAnalyst,
                &format!(
                    "Analyze the following project idea and produce a PRD:\n\n{}",
                    self.idea
                ),
            )
            .await?;

        self.blackboard
            .publish_artifact(Artifact::Prd, json!(prd), "business_analyst");

        // Phase 2: Parallel Build
        self.phase = WorkflowPhase::ParallelBuild;

        let design_prompt = format!("Based on this PRD, produce a design specification:\n\n{prd}");
        let dev_prompt = format!("Based on this PRD, implement the project:\n\n{prd}");
        let test_prompt = format!("Based on this PRD, write test cases:\n\n{prd}");

        let (design_result, dev_result, test_result) = tokio::join!(
            self.run_agent_simple(AgentRole::UiUxDesigner, &design_prompt),
            self.run_agent_agentic(AgentRole::Developer, &dev_prompt),
            self.run_agent_simple(AgentRole::Tester, &test_prompt),
        );

        let design_spec = design_result?;
        self.blackboard.publish_artifact(
            Artifact::DesignSpec,
            json!(design_spec),
            "ui_ux_designer",
        );

        let dev_output = dev_result?;
        self.blackboard
            .publish_artifact(Artifact::SourceCode, json!(dev_output), "developer");

        let test_cases = test_result?;
        self.blackboard
            .publish_artifact(Artifact::TestCases, json!(test_cases), "tester");

        // Phase 3: Integration Loop (Dev-Tester ping-pong)
        self.phase = WorkflowPhase::IntegrationLoop;
        let mut tests_passed = false;

        for iteration in 1..=self.max_ping_pong {
            // Tester runs tests
            let test_prompt = format!(
                "Run the test cases against the current source code. \
                 Test cases:\n{test_cases}\n\n\
                 Report which tests pass and which fail. \
                 If all tests pass, say 'ALL TESTS PASSED'. \
                 Iteration {iteration}/{max}.",
                max = self.max_ping_pong,
            );

            let test_result = self
                .run_agent_agentic(AgentRole::Tester, &test_prompt)
                .await?;

            self.blackboard
                .publish_artifact(Artifact::TestResults, json!(test_result), "tester");

            if test_result.to_uppercase().contains("ALL TESTS PASSED") {
                tests_passed = true;
                break;
            }

            // Developer fixes based on test results
            if iteration < self.max_ping_pong {
                let fix_prompt = format!(
                    "The following tests failed. Fix the code:\n\n{test_result}\n\n\
                     Iteration {iteration}/{max}.",
                    max = self.max_ping_pong,
                );

                let fix_output = self
                    .run_agent_agentic(AgentRole::Developer, &fix_prompt)
                    .await?;
                self.blackboard.publish_artifact(
                    Artifact::SourceCode,
                    json!(fix_output),
                    "developer",
                );
            }
        }

        // Phase 4: Deployment
        if tests_passed {
            self.phase = WorkflowPhase::Deployment;

            let deploy_result = self
                .run_agent_agentic(
                    AgentRole::DevOps,
                    "Deploy the project. Push the code to GitHub.",
                )
                .await?;

            self.blackboard.publish_artifact(
                Artifact::DeployConfig,
                json!(deploy_result),
                "devops",
            );

            self.phase = WorkflowPhase::Completed;

            Ok(format!(
                "Factory build completed successfully.\n\n\
                 PRD: {prd}\n\n\
                 Design: {design_spec}\n\n\
                 Deployment: {deploy_result}"
            ))
        } else {
            self.phase = WorkflowPhase::Failed;

            let test_results = self
                .blackboard
                .read_artifact(&Artifact::TestResults)
                .unwrap_or(json!("No test results"));

            Ok(format!(
                "Factory build completed with test failures after {} iterations.\n\n\
                 PRD: {prd}\n\n\
                 Last test results: {test_results}",
                self.max_ping_pong
            ))
        }
    }

    /// Current workflow phase.
    pub fn phase(&self) -> WorkflowPhase {
        self.phase
    }

    // ── Agent execution helpers ──────────────────────────────────

    /// Run a non-agentic agent (single prompt → single response, no tool calls).
    async fn run_agent_simple(&self, role: AgentRole, prompt: &str) -> Result<String> {
        let config = self.resolve_config(role);
        let provider = self.create_provider(&config)?;

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(120),
            provider.chat_with_system(
                config.system_prompt.as_deref(),
                prompt,
                &config.model,
                config.temperature.unwrap_or(0.7),
            ),
        )
        .await;

        match result {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(e)) => bail!("Agent {role} failed: {e}"),
            Err(_) => bail!("Agent {role} timed out"),
        }
    }

    /// Run an agentic agent (multi-turn tool-call loop with filtered tools).
    async fn run_agent_agentic(&self, role: AgentRole, prompt: &str) -> Result<String> {
        let config = self.resolve_config(role);

        if config.allowed_tools.is_empty() {
            // Fall back to simple mode if no tools are configured
            return self.run_agent_simple(role, prompt).await;
        }

        let provider = self.create_provider(&config)?;

        let allowed: std::collections::HashSet<&str> =
            config.allowed_tools.iter().map(|s| s.as_str()).collect();

        let sub_tools: Vec<Box<dyn Tool>> = self
            .parent_tools
            .iter()
            .filter(|tool| allowed.contains(tool.name()))
            .map(|tool| Box::new(ToolArcRef(tool.clone())) as Box<dyn Tool>)
            .collect();

        if sub_tools.is_empty() {
            return self.run_agent_simple(role, prompt).await;
        }

        let mut history = Vec::new();
        if let Some(sys) = config.system_prompt.as_ref() {
            history.push(ChatMessage::system(sys.clone()));
        }
        history.push(ChatMessage::user(prompt.to_string()));

        let noop = NoopObserver;

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(300),
            run_tool_call_loop(
                &*provider,
                &mut history,
                &sub_tools,
                &noop,
                &config.provider,
                &config.model,
                config.temperature.unwrap_or(0.7),
                true,
                None,
                &format!("factory:{role}"),
                &self.multimodal_config,
                config.max_iterations,
                None,
                None,
                None,
                &[],
            ),
        )
        .await;

        match result {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(e)) => bail!("Agent {role} failed: {e}"),
            Err(_) => bail!("Agent {role} timed out after 300s"),
        }
    }

    /// Resolve the effective config for a role (defaults + overrides).
    fn resolve_config(&self, role: AgentRole) -> DelegateAgentConfig {
        let mut role_config = RoleConfig::default_for(role);

        // Apply per-role overrides from factory config
        if let Some(overrides) = self.role_overrides.get(&role.to_string()) {
            role_config = role_config.with_overrides(overrides);
        }

        // Fill in provider/model from defaults if still empty
        let mut config = role_config.delegate_config;
        if config.provider.is_empty() {
            config.provider = self.default_provider.clone();
        }
        if config.model.is_empty() {
            config.model = self.default_model.clone();
        }
        // Use fallback credential if no per-role key
        if config.api_key.is_none() {
            config.api_key = self.fallback_credential.clone();
        }

        config
    }

    /// Create a provider instance from a delegate config.
    fn create_provider(&self, config: &DelegateAgentConfig) -> Result<Box<dyn Provider>> {
        providers::create_provider_with_options(
            &config.provider,
            config.api_key.as_deref(),
            &self.provider_runtime_options,
        )
    }
}

// ── Internal helpers ──────────────────────────────────────────

struct ToolArcRef(Arc<dyn Tool>);

#[async_trait::async_trait]
impl Tool for ToolArcRef {
    fn name(&self) -> &str {
        self.0.name()
    }

    fn description(&self) -> &str {
        self.0.description()
    }

    fn parameters_schema(&self) -> serde_json::Value {
        self.0.parameters_schema()
    }

    async fn execute(&self, args: serde_json::Value) -> Result<crate::tools::traits::ToolResult> {
        self.0.execute(args).await
    }
}

struct NoopObserver;

impl Observer for NoopObserver {
    fn record_event(&self, _event: &ObserverEvent) {}
    fn record_metric(&self, _metric: &ObserverMetric) {}
    fn name(&self) -> &str {
        "noop"
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workflow_phases_serialize() {
        let phase = WorkflowPhase::ParallelBuild;
        let json = serde_json::to_string(&phase).unwrap();
        assert_eq!(json, "\"parallel_build\"");

        let parsed: WorkflowPhase = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, WorkflowPhase::ParallelBuild);
    }

    #[test]
    fn resolve_config_uses_defaults() {
        let wf = FactoryWorkflow::new(
            "test idea".into(),
            5,
            HashMap::new(),
            providers::ProviderRuntimeOptions::default(),
            Some("test-key".into()),
            "openrouter".into(),
            "test-model".into(),
            Arc::new(Vec::new()),
            crate::config::MultimodalConfig::default(),
        );

        let config = wf.resolve_config(AgentRole::Developer);
        assert_eq!(config.provider, "openrouter");
        assert_eq!(config.model, "test-model");
        assert_eq!(config.api_key.as_deref(), Some("test-key"));
        assert!(config.system_prompt.is_some());
    }

    #[test]
    fn resolve_config_applies_overrides() {
        let mut overrides = HashMap::new();
        overrides.insert(
            "developer".to_string(),
            DelegateAgentConfig {
                provider: "anthropic".to_string(),
                model: "claude-sonnet".to_string(),
                system_prompt: None,
                api_key: Some("dev-key".to_string()),
                temperature: Some(0.1),
                max_depth: 1,
                agentic: true,
                allowed_tools: Vec::new(),
                max_iterations: 10,
            },
        );

        let wf = FactoryWorkflow::new(
            "test".into(),
            5,
            overrides,
            providers::ProviderRuntimeOptions::default(),
            None,
            "openrouter".into(),
            "default-model".into(),
            Arc::new(Vec::new()),
            crate::config::MultimodalConfig::default(),
        );

        let config = wf.resolve_config(AgentRole::Developer);
        assert_eq!(config.provider, "anthropic");
        assert_eq!(config.model, "claude-sonnet");
        assert_eq!(config.api_key.as_deref(), Some("dev-key"));
        // System prompt should be preserved (override was None)
        assert!(config.system_prompt.is_some());
    }

    #[test]
    fn initial_phase_is_analysis() {
        let wf = FactoryWorkflow::new(
            "test".into(),
            5,
            HashMap::new(),
            providers::ProviderRuntimeOptions::default(),
            None,
            "openrouter".into(),
            "model".into(),
            Arc::new(Vec::new()),
            crate::config::MultimodalConfig::default(),
        );

        assert_eq!(wf.phase(), WorkflowPhase::Analysis);
    }
}
