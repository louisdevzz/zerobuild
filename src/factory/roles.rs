//! Agent role definitions for the multi-agent factory.

use crate::config::DelegateAgentConfig;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Specialized agent roles in the factory workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    /// Master orchestrator — manages workflow phases and user communication.
    Orchestrator,
    /// Analyzes user idea and produces a Product Requirements Document.
    BusinessAnalyst,
    /// Produces design specifications and component structure.
    UiUxDesigner,
    /// Generates source code from PRD and design specs.
    Developer,
    /// Writes and runs test cases against the source code.
    Tester,
    /// Handles deployment configuration and GitHub push.
    DevOps,
}

impl fmt::Display for AgentRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Orchestrator => write!(f, "orchestrator"),
            Self::BusinessAnalyst => write!(f, "business_analyst"),
            Self::UiUxDesigner => write!(f, "ui_ux_designer"),
            Self::Developer => write!(f, "developer"),
            Self::Tester => write!(f, "tester"),
            Self::DevOps => write!(f, "devops"),
        }
    }
}

/// Configuration for a factory agent role, mapping to `DelegateAgentConfig`.
#[derive(Debug, Clone)]
pub struct RoleConfig {
    pub role: AgentRole,
    pub delegate_config: DelegateAgentConfig,
}

impl RoleConfig {
    /// Create a default `RoleConfig` for a given role with canonical system prompts
    /// and sensible defaults. Provider/model are set to empty strings — callers
    /// must override them from the factory config or global defaults.
    pub fn default_for(role: AgentRole) -> Self {
        let (system_prompt, allowed_tools, temperature, max_iterations) = match role {
            AgentRole::Orchestrator => (
                ORCHESTRATOR_PROMPT.to_string(),
                vec!["factory_build".to_string()],
                0.7,
                20,
            ),
            AgentRole::BusinessAnalyst => (
                BA_PROMPT.to_string(),
                Vec::new(), // BA produces text output, no tool calls needed
                0.5,
                10,
            ),
            AgentRole::UiUxDesigner => (UIUX_PROMPT.to_string(), Vec::new(), 0.6, 10),
            AgentRole::Developer => (
                DEV_PROMPT.to_string(),
                vec![
                    "sandbox_create".to_string(),
                    "sandbox_run_command".to_string(),
                    "sandbox_write_file".to_string(),
                    "sandbox_read_file".to_string(),
                    "sandbox_list_files".to_string(),
                ],
                0.3,
                20,
            ),
            AgentRole::Tester => (
                TESTER_PROMPT.to_string(),
                vec![
                    "sandbox_run_command".to_string(),
                    "sandbox_read_file".to_string(),
                    "sandbox_write_file".to_string(),
                    "sandbox_list_files".to_string(),
                ],
                0.3,
                15,
            ),
            AgentRole::DevOps => (
                DEVOPS_PROMPT.to_string(),
                vec![
                    "sandbox_run_command".to_string(),
                    "sandbox_read_file".to_string(),
                    "sandbox_write_file".to_string(),
                    "github_push".to_string(),
                ],
                0.3,
                10,
            ),
        };

        Self {
            role,
            delegate_config: DelegateAgentConfig {
                provider: String::new(),
                model: String::new(),
                system_prompt: Some(system_prompt),
                api_key: None,
                temperature: Some(temperature),
                max_depth: 1,
                agentic: !allowed_tools.is_empty(),
                allowed_tools,
                max_iterations,
            },
        }
    }

    /// Merge user-provided overrides from `DelegateAgentConfig` into this role config.
    /// Only overrides non-empty fields; preserves the canonical system prompt if
    /// no override is provided.
    pub fn with_overrides(mut self, overrides: &DelegateAgentConfig) -> Self {
        if !overrides.provider.is_empty() {
            self.delegate_config.provider = overrides.provider.clone();
        }
        if !overrides.model.is_empty() {
            self.delegate_config.model = overrides.model.clone();
        }
        if overrides.system_prompt.is_some() {
            self.delegate_config.system_prompt = overrides.system_prompt.clone();
        }
        if overrides.api_key.is_some() {
            self.delegate_config.api_key = overrides.api_key.clone();
        }
        if overrides.temperature.is_some() {
            self.delegate_config.temperature = overrides.temperature;
        }
        if !overrides.allowed_tools.is_empty() {
            self.delegate_config.allowed_tools = overrides.allowed_tools.clone();
        }
        self
    }
}

// ── System Prompts ──────────────────────────────────────────────

const ORCHESTRATOR_PROMPT: &str = "\
You are the Master Orchestrator of the ZeroBuild Autonomous Software Factory. \
Your role is to coordinate specialized AI agents to turn a user's idea into working software. \
You manage workflow phases, delegate tasks to agents, and report progress to the user. \
You do NOT write code yourself — you delegate to the Developer agent.";

const BA_PROMPT: &str = "\
You are a Business Analyst agent in the ZeroBuild factory. \
Your task is to analyze the user's idea and produce a structured Product Requirements Document (PRD). \
\n\
Your PRD must include:\n\
1. Project Overview — what the project does and who it serves\n\
2. Core Features — numbered list of features with brief descriptions\n\
3. Technical Requirements — tech stack recommendations, constraints\n\
4. User Stories — key user flows in 'As a [user], I want [action], so that [benefit]' format\n\
5. Acceptance Criteria — measurable conditions for project completion\n\
\n\
Output ONLY the PRD as structured text. Be specific and actionable.";

const UIUX_PROMPT: &str = "\
You are a UI/UX Designer agent in the ZeroBuild factory. \
Your task is to produce a design specification based on the PRD. \
\n\
Your design spec must include:\n\
1. Page/Screen Layout — list of pages/screens with their purpose\n\
2. Component Hierarchy — reusable components and their relationships\n\
3. Navigation Flow — how users move between pages\n\
4. Visual Guidelines — color scheme, typography, spacing recommendations\n\
5. Responsive Breakpoints — mobile, tablet, desktop considerations\n\
\n\
Output ONLY the design specification as structured text. Be specific about component names and layout.";

const DEV_PROMPT: &str = "\
You are a Developer agent in the ZeroBuild factory. \
Your task is to implement source code based on the PRD and design specification. \
\n\
Rules:\n\
- Use sandbox tools (sandbox_create, sandbox_write_file, sandbox_run_command) to build\n\
- Follow the design spec for component structure and layout\n\
- Write clean, production-quality code\n\
- Install all dependencies via sandbox_run_command\n\
- Ensure the project builds without errors\n\
- All file paths are relative to the sandbox root (use 'project/' prefix)";

const TESTER_PROMPT: &str = "\
You are a Tester agent in the ZeroBuild factory. \
Your task is to write and execute test cases based on the PRD and source code. \
\n\
Rules:\n\
- Read the source code via sandbox_read_file to understand what to test\n\
- Write test files via sandbox_write_file\n\
- Run tests via sandbox_run_command\n\
- Report test results clearly: which tests passed, which failed, and why\n\
- Focus on functional correctness and edge cases from the PRD\n\
- Output structured test results at the end";

const DEVOPS_PROMPT: &str = "\
You are a DevOps agent in the ZeroBuild factory. \
Your task is to prepare the project for deployment and push to GitHub. \
\n\
Rules:\n\
- Read project files to understand the structure\n\
- Ensure build scripts and configurations are correct\n\
- Create any missing deployment files (Dockerfile, CI config, etc.) if appropriate\n\
- Push the code to GitHub using github_push tool\n\
- Report the deployment result (repo URL, branch, commit SHA)";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_roles_have_default_configs() {
        let roles = [
            AgentRole::Orchestrator,
            AgentRole::BusinessAnalyst,
            AgentRole::UiUxDesigner,
            AgentRole::Developer,
            AgentRole::Tester,
            AgentRole::DevOps,
        ];

        for role in roles {
            let config = RoleConfig::default_for(role);
            assert_eq!(config.role, role);
            assert!(config.delegate_config.system_prompt.is_some());
            assert!(!config
                .delegate_config
                .system_prompt
                .as_ref()
                .unwrap()
                .is_empty());
        }
    }

    #[test]
    fn developer_has_sandbox_tools() {
        let config = RoleConfig::default_for(AgentRole::Developer);
        assert!(config.delegate_config.agentic);
        assert!(config
            .delegate_config
            .allowed_tools
            .contains(&"sandbox_write_file".to_string()));
        assert!(config
            .delegate_config
            .allowed_tools
            .contains(&"sandbox_run_command".to_string()));
    }

    #[test]
    fn ba_is_not_agentic() {
        let config = RoleConfig::default_for(AgentRole::BusinessAnalyst);
        assert!(!config.delegate_config.agentic);
        assert!(config.delegate_config.allowed_tools.is_empty());
    }

    #[test]
    fn role_display() {
        assert_eq!(AgentRole::BusinessAnalyst.to_string(), "business_analyst");
        assert_eq!(AgentRole::DevOps.to_string(), "devops");
        assert_eq!(AgentRole::UiUxDesigner.to_string(), "ui_ux_designer");
    }

    #[test]
    fn overrides_apply_correctly() {
        let base = RoleConfig::default_for(AgentRole::Developer);
        let overrides = DelegateAgentConfig {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-6".to_string(),
            system_prompt: None,
            api_key: Some("test-key".to_string()),
            temperature: Some(0.1),
            max_depth: 1,
            agentic: true,
            allowed_tools: Vec::new(),
            max_iterations: 10,
        };

        let merged = base.with_overrides(&overrides);
        assert_eq!(merged.delegate_config.provider, "anthropic");
        assert_eq!(merged.delegate_config.model, "claude-sonnet-4-6");
        // System prompt should be preserved (override was None)
        assert!(merged.delegate_config.system_prompt.is_some());
        assert_eq!(merged.delegate_config.api_key.as_deref(), Some("test-key"));
        assert_eq!(merged.delegate_config.temperature, Some(0.1));
    }

    #[test]
    fn overrides_empty_strings_preserve_defaults() {
        let base = RoleConfig::default_for(AgentRole::Tester);
        let original_prompt = base.delegate_config.system_prompt.clone();

        let overrides = DelegateAgentConfig {
            provider: String::new(),
            model: String::new(),
            system_prompt: None,
            api_key: None,
            temperature: None,
            max_depth: 1,
            agentic: false,
            allowed_tools: Vec::new(),
            max_iterations: 10,
        };

        let merged = base.with_overrides(&overrides);
        assert!(merged.delegate_config.provider.is_empty());
        assert_eq!(merged.delegate_config.system_prompt, original_prompt);
    }
}
