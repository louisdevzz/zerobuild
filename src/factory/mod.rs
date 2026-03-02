//! Multi-agent factory module for the Autonomous Software Factory.
//!
//! This module implements a hierarchical multi-agent workflow where a Master
//! Orchestrator spawns specialized AI agents (Business Analyst, UI/UX Designer,
//! Developer, Tester, DevOps) that collaborate through phased execution to turn
//! a user's idea into working software.
//!
//! # Architecture
//!
//! - [`roles`]: Agent role definitions with canonical system prompts
//! - [`blackboard`]: Shared state management built on `InMemoryMessageBus`
//! - [`workflow`]: Workflow state machine with phased execution
//! - [`orchestrator_tool`]: `factory_build` tool implementing the `Tool` trait
//!
//! # Configuration
//!
//! The factory is opt-in via `[factory]` config section. When `factory.enabled`
//! is false (default), no factory tools are registered and existing single-agent
//! mode is unaffected.

pub mod blackboard;
pub mod orchestrator_tool;
pub mod roles;
pub mod workflow;

pub use blackboard::Blackboard;
pub use orchestrator_tool::FactoryOrchestratorTool;
pub use roles::{AgentRole, RoleConfig};
pub use workflow::{FactoryWorkflow, WorkflowPhase};
