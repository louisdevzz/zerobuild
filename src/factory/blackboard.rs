//! Shared state management for the multi-agent factory.
//!
//! The Blackboard provides typed artifact storage for inter-agent communication
//! during factory workflow execution. Artifacts are versioned and thread-safe.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Artifact types produced and consumed by factory agents.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Artifact {
    /// Product Requirements Document from Business Analyst.
    Prd,
    /// UI/UX design specification.
    DesignSpec,
    /// Source code manifest (file listing or summary).
    SourceCode,
    /// Test case definitions.
    TestCases,
    /// Test execution results (pass/fail).
    TestResults,
    /// Deployment configuration.
    DeployConfig,
}

/// A versioned artifact entry on the blackboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactEntry {
    pub value: Value,
    pub version: u64,
    pub updated_by: String,
}

/// Shared state layer for inter-agent communication in the factory workflow.
///
/// Thread-safe, cloneable. Agents publish artifacts via `publish_artifact()` and
/// read them via `read_artifact()`.
#[derive(Debug, Clone)]
pub struct Blackboard {
    entries: Arc<Mutex<HashMap<Artifact, ArtifactEntry>>>,
}

impl Blackboard {
    /// Create a new empty Blackboard.
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Publish an artifact to the blackboard.
    ///
    /// Overwrites any existing value for the same artifact type.
    /// The version is incremented automatically.
    pub fn publish_artifact(&self, artifact: Artifact, value: Value, from: &str) {
        let mut entries = self.entries.lock().unwrap();
        let version = entries.get(&artifact).map(|e| e.version + 1).unwrap_or(1);

        entries.insert(
            artifact,
            ArtifactEntry {
                value,
                version,
                updated_by: from.to_string(),
            },
        );
    }

    /// Read an artifact value from the blackboard.
    ///
    /// Returns `None` if the artifact has not been published yet.
    pub fn read_artifact(&self, artifact: &Artifact) -> Option<Value> {
        self.entries
            .lock()
            .unwrap()
            .get(artifact)
            .map(|e| e.value.clone())
    }

    /// Read a full artifact entry (includes version metadata).
    pub fn read_entry(&self, artifact: &Artifact) -> Option<ArtifactEntry> {
        self.entries.lock().unwrap().get(artifact).cloned()
    }

    /// Check if an artifact has been published.
    pub fn has_artifact(&self, artifact: &Artifact) -> bool {
        self.entries.lock().unwrap().contains_key(artifact)
    }
}

impl Default for Blackboard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn publish_and_read_artifact() {
        let board = Blackboard::new();
        let prd = json!({
            "title": "Todo App",
            "features": ["add tasks", "delete tasks"]
        });

        board.publish_artifact(Artifact::Prd, prd.clone(), "business_analyst");

        let read = board.read_artifact(&Artifact::Prd);
        assert_eq!(read, Some(prd));
    }

    #[test]
    fn read_missing_artifact_returns_none() {
        let board = Blackboard::new();
        assert!(board.read_artifact(&Artifact::DesignSpec).is_none());
    }

    #[test]
    fn has_artifact_tracks_publication() {
        let board = Blackboard::new();
        assert!(!board.has_artifact(&Artifact::TestResults));

        board.publish_artifact(Artifact::TestResults, json!({"passed": true}), "tester");

        assert!(board.has_artifact(&Artifact::TestResults));
    }

    #[test]
    fn overwrite_artifact_increments_version() {
        let board = Blackboard::new();

        board.publish_artifact(Artifact::SourceCode, json!({"v": 1}), "developer");
        let entry1 = board.read_entry(&Artifact::SourceCode).unwrap();
        assert_eq!(entry1.version, 1);

        board.publish_artifact(Artifact::SourceCode, json!({"v": 2}), "developer");
        let entry2 = board.read_entry(&Artifact::SourceCode).unwrap();
        assert_eq!(entry2.version, 2);
        assert_eq!(entry2.value, json!({"v": 2}));
    }

    #[test]
    fn artifact_keys_are_distinct() {
        let board = Blackboard::new();

        board.publish_artifact(Artifact::Prd, json!("prd"), "ba");
        board.publish_artifact(Artifact::DesignSpec, json!("design"), "uiux");

        assert_eq!(board.read_artifact(&Artifact::Prd), Some(json!("prd")));
        assert_eq!(
            board.read_artifact(&Artifact::DesignSpec),
            Some(json!("design"))
        );
    }

    #[test]
    fn entry_tracks_author() {
        let board = Blackboard::new();
        board.publish_artifact(Artifact::TestCases, json!("tests"), "tester");

        let entry = board.read_entry(&Artifact::TestCases).unwrap();
        assert_eq!(entry.updated_by, "tester");
    }

    #[test]
    fn thread_safe_clone() {
        let board = Blackboard::new();
        let board2 = board.clone();

        board.publish_artifact(Artifact::Prd, json!("shared"), "ba");
        assert_eq!(board2.read_artifact(&Artifact::Prd), Some(json!("shared")));
    }
}
