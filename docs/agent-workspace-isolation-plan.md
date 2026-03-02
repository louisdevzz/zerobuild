# ZeroBuild Agent Workspace Isolation & Communication Protocol

> **Status**: Planning Phase  
> **Last Updated**: 2026-03-02  
> **Scope**: Major architectural refactor for multi-agent workspace isolation

## Executive Summary

Currently ZeroBuild uses a shared sandbox (`/tmp/zerobuild-sandbox-{uuid}`) for all agents. This proposal transitions to a **workspace-based architecture** where each agent has:
- Isolated workspace (`~/.zerobuild/workspaces/{agent-id}/`)
- Dedicated skills, memory, identity, and agents.md
- Communication protocol to interact with Orchestrator and other agents
- Agent pool for Orchestrator to manage lifecycle

---

## 1. Current Architecture Problems

### 1.1 Shared Sandbox Issues
```
Current: /tmp/zerobuild-sandbox-{uuid}/
├── project/           ← All agents share
│   ├── app/
│   ├── components/
│   └── ...
└── All agents can read/write same files
```

**Problems:**
- ❌ No isolation between agents
- ❌ One agent can accidentally delete another's work
- ❌ Hard to debug (which agent created which file?)
- ❌ Cannot rollback individual agent changes
- ❌ No agent-specific configuration

### 1.2 Missing Agent Identity
- ❌ All agents share same `IDENTITY.md`
- ❌ No agent-specific skills
- ❌ No agent-specific memory/conversation history
- ❌ Cannot customize personality per role

### 1.3 Communication Gaps
- ❌ Agents communicate only via Blackboard (artifacts)
- ❌ No direct message passing
- ❌ No request/response pattern
- ❌ Orchestrator cannot query agent status real-time

---

## 2. Proposed Architecture

### 2.1 Workspace Structure

```
~/.zerobuild/
├── config.toml                    # Global config
├── workspaces/                    # Each agent has workspace
│   ├── orchestrator-{uuid}/       # Orchestrator workspace
│   │   ├── .agent/
│   │   │   ├── identity.md        # Orchestrator identity
│   │   │   ├── agents.md          # Sub-agent definitions
│   │   │   ├── skills/            # Orchestrator skills
│   │   │   │   ├── delegation.md
│   │   │   │   ├── planning.md
│   │   │   │   └── monitoring.md
│   │   │   ├── memory/            # Conversation history
│   │   │   │   ├── conversations/
│   │   │   │   └── embeddings/
│   │   │   └── state.json         # Agent state
│   │   └── projects/              # Project references
│   │
│   ├── ba-{uuid}/                 # Business Analyst
│   │   ├── .agent/
│   │   │   ├── identity.md        # BA personality
│   │   │   ├── skills/
│   │   │   │   ├── requirements-analysis.md
│   │   │   │   ├── prd-writing.md
│   │   │   │   └── user-story.md
│   │   │   ├── memory/
│   │   │   └── state.json
│   │   └── sandbox/               # BA's isolated sandbox
│   │       ├── prd.md
│   │       └── research/
│   │
│   ├── uiux-{uuid}/               # UI/UX Designer
│   │   ├── .agent/
│   │   │   ├── identity.md
│   │   │   ├── skills/
│   │   │   │   ├── wireframing.md
│   │   │   │   ├── design-system.md
│   │   │   │   └── accessibility.md
│   │   │   └── memory/
│   │   └── sandbox/
│   │       ├── design-spec.md
│   │       └── mockups/
│   │
│   ├── dev-{uuid}/                # Developer
│   │   ├── .agent/
│   │   │   ├── identity.md
│   │   │   ├── skills/
│   │   │   │   ├── nextjs.md
│   │   │   │   ├── rust.md
│   │   │   │   ├── testing.md
│   │   │   │   └── debugging.md
│   │   │   └── memory/
│   │   └── sandbox/               # Dev's isolated workspace
│   │       ├── project/           # Source code
│   │       ├── node_modules/
│   │       └── .git/
│   │
│   ├── tester-{uuid}/             # Tester
│   │   ├── .agent/
│   │   │   ├── identity.md
│   │   │   ├── skills/
│   │   │   │   ├── test-writing.md
│   │   │   │   ├── e2e-testing.md
│   │   │   │   └── security-testing.md
│   │   │   └── memory/
│   │   └── sandbox/
│   │       ├── tests/
│   │       └── test-results/
│   │
│   └── devops-{uuid}/             # DevOps
│       ├── .agent/
│       │   ├── identity.md
│       │   ├── skills/
│       │   │   ├── docker.md
│       │   │   ├── ci-cd.md
│       │   │   └── github-actions.md
│       │   └── memory/
│       └── sandbox/
│           └── deployment/
│
└── shared/                        # Shared resources
    ├── blackboard/               # Artifacts database
    ├── protocols/                # Communication protocols
    └── pools/                    # Agent pool state
```

### 2.2 Key Changes

| Component | Current | Proposed |
|-----------|---------|----------|
| **Sandbox Location** | `/tmp/zerobuild-sandbox-{uuid}/` | `~/.zerobuild/workspaces/{agent-id}/sandbox/` |
| **Agent Identity** | Shared `IDENTITY.md` | Per-agent `identity.md` |
| **Skills** | Global skills | Per-agent `skills/` folder |
| **Memory** | Shared memory | Per-agent `memory/` folder |
| **Communication** | Blackboard only | Protocol-based messaging + Blackboard |
| **Agent Lifecycle** | Spawn on demand | Managed pool with warm/cold states |

---

## 3. Inter-Agent Communication Protocol (IACP)

### 3.1 Design Principles: Extensibility First

**Problem with hardcoded enums**: Adding new agent types requires modifying core protocol code, violating Open/Closed Principle.

**Solution**: Generic message envelope + dynamic type registry + capability-based routing.

### 3.2 Protocol Overview

```rust
/// Generic message envelope - all message types use this structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    /// Message metadata
    pub header: MessageHeader,
    /// Type-specific content (generic, not hardcoded)
    pub content: MessageContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    pub message_id: Uuid,
    pub correlation_id: Option<Uuid>,  // For request/response pairing
    pub from: AgentId,
    pub to: Option<AgentId>,           // None = broadcast
    pub message_type: String,          // Dynamic type: "request_clarification", "code_review", etc.
    pub intent: MessageIntent,         // Semantics of the message
    pub timestamp: DateTime<Utc>,
    pub ttl_seconds: Option<u64>,      // Time-to-live for the message
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageIntent {
    /// Request expects a response
    Request { expects_response: bool, timeout_ms: u64 },
    /// Response to a previous request
    Response { request_id: Uuid },
    /// Fire-and-forget event
    Event,
    /// Command from orchestrator
    Command { priority: Priority },
    /// Status update
    Status,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContent {
    /// Schema version for forward compatibility
    pub schema_version: String,
    /// MIME-like content type: "application/vnd.zerobuild.clarification+json"
    pub content_type: String,
    /// Actual payload (validated against registered schema)
    pub payload: serde_json::Value,
    /// Optional: link to artifact in blackboard
    pub artifact_reference: Option<ArtifactRef>,
}

/// Artifact reference for linking to blackboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRef {
    pub artifact_type: String,
    pub artifact_id: Uuid,
    pub version: u64,
}
```

### 3.3 Dynamic Message Type Registry

```rust
/// Registry for message types - agents register what they can handle
pub struct MessageTypeRegistry {
    /// Map message_type string → schema + handlers
    types: DashMap<String, MessageTypeDef>,
    /// Capability map: what can agent X handle?
    agent_capabilities: DashMap<AgentId, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct MessageTypeDef {
    /// JSON schema for validation
    pub schema: JSONSchema,
    /// Human-readable description
    pub description: String,
    /// Default timeout for requests of this type
    pub default_timeout_ms: u64,
    /// Required capabilities to handle this message
    pub required_capabilities: Vec<String>,
    /// Example payload for documentation
    pub example: serde_json::Value,
}

impl MessageTypeRegistry {
    /// Register a new message type dynamically
    pub fn register_type(&self, type_def: MessageTypeDef) -> Result<()> {
        // Validate schema is valid JSON Schema
        self.types.insert(type_def.description.clone(), type_def);
        Ok(())
    }
    
    /// Register what message types an agent can handle
    pub fn register_agent_capabilities(
        &self, 
        agent_id: AgentId, 
        capabilities: Vec<String>
    ) {
        self.agent_capabilities.insert(agent_id, capabilities);
    }
    
    /// Find agents that can handle a specific message type
    pub fn find_capable_agents(&self, message_type: &str) -> Vec<AgentId> {
        self.agent_capabilities
            .iter()
            .filter(|entry| entry.value().contains(&message_type.to_string()))
            .map(|entry| entry.key().clone())
            .collect()
    }
    
    /// Validate message payload against registered schema
    pub fn validate(&self, message: &AgentMessage) -> Result<()> {
        if let Some(type_def) = self.types.get(&message.header.message_type) {
            type_def.schema.validate(&message.content.payload)?;
            Ok(())
        } else {
            bail!("Unknown message type: {}", message.header.message_type)
        }
    }
}
```

### 3.4 Message Handler System

```rust
/// Trait for message handlers - implemented by each agent
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// Return list of message types this handler can process
    fn supported_types(&self) -> Vec<String>;
    
    /// Handle an incoming message
    async fn handle(&self, message: AgentMessage, ctx: HandlerContext) -> Result<Option<AgentMessage>>;
}

/// Context provided to handlers
pub struct HandlerContext {
    pub agent_id: AgentId,
    pub bus: Arc<AgentMessageBus>,
    pub blackboard: Arc<Blackboard>,
    pub workspace: PathBuf,
}

/// Example: BA agent handler
pub struct BusinessAnalystHandler;

#[async_trait]
impl MessageHandler for BusinessAnalystHandler {
    fn supported_types(&self) -> Vec<String> {
        vec![
            "request_clarification".to_string(),
            "provide_requirements".to_string(),
            "review_prd".to_string(),
        ]
    }
    
    async fn handle(&self, msg: AgentMessage, ctx: HandlerContext) -> Result<Option<AgentMessage>> {
        match msg.header.message_type.as_str() {
            "request_clarification" => {
                let req: ClarificationRequest = serde_json::from_value(msg.content.payload)?;
                let answer = self.answer_clarification(req).await?;
                
                Ok(Some(AgentMessage::response(
                    &msg,
                    "clarification_response",
                    serde_json::to_value(answer)?,
                )))
            }
            // ... other types
            _ => Ok(None)
        }
    }
}

/// Message bus with dynamic handler routing
pub struct AgentMessageBus {
    /// Channel for each agent
    channels: DashMap<AgentId, mpsc::UnboundedSender<AgentMessage>>,
    /// Handler registry
    handlers: DashMap<AgentId, Arc<dyn MessageHandler>>,
    /// Type registry
    type_registry: Arc<MessageTypeRegistry>,
}

impl AgentMessageBus {
    /// Send message to specific agent
    pub async fn send(&self, message: AgentMessage) -> Result<()> {
        // Validate message first
        self.type_registry.validate(&message)?;
        
        if let Some(to) = &message.header.to {
            // Direct message
            if let Some(channel) = self.channels.get(to) {
                channel.send(message)?;
            } else {
                // Queue for offline agent
                self.queue_for_later(to.clone(), message).await?;
            }
        } else {
            // Broadcast to all capable agents
            let capable = self.type_registry
                .find_capable_agents(&message.header.message_type);
            
            for agent_id in capable {
                if let Some(channel) = self.channels.get(&agent_id) {
                    let _ = channel.send(message.clone());
                }
            }
        }
        Ok(())
    }
    
    /// Route message to appropriate handler
    pub async fn route(&self, to: AgentId, message: AgentMessage) -> Result<()> {
        if let Some(handler) = self.handlers.get(&to) {
            let ctx = HandlerContext {
                agent_id: to.clone(),
                bus: Arc::new(self.clone()),
                blackboard: self.get_blackboard(),
                workspace: self.get_workspace(&to),
            };
            
            if let Some(response) = handler.handle(message, ctx).await? {
                self.send(response).await?;
            }
        }
        Ok(())
    }
}
```

### 3.5 Example Message Definitions (JSON Schema)

Instead of hardcoded Rust enums, message types are defined as JSON Schema:

```yaml
# ~/.zerobuild/protocols/messages/clarification_request.yaml
message_type: "request_clarification"
description: "Request clarification on requirements"
version: "1.0.0"

schema:
  type: object
  required: [question, context]
  properties:
    question:
      type: string
      description: "Specific question to answer"
    context:
      type: string
      description: "Background context"
    urgency:
      type: string
      enum: [low, medium, high]
      default: medium

handlers:
  - role: business_analyst
    capability: requirements_expert
  - role: product_manager
    capability: domain_expert

response_type: "clarification_response"
default_timeout_ms: 30000
```

```yaml
# ~/.zerobuild/protocols/messages/code_review_request.yaml
message_type: "code_review"
description: "Request code review from another agent"
version: "1.0.0"

schema:
  type: object
  required: [file_path, code_snippet]
  properties:
    file_path:
      type: string
    code_snippet:
      type: string
    focus_areas:
      type: array
      items:
        type: string
        enum: [security, performance, readability, testing]

handlers:
  - role: tester
    capability: quality_assurance
  - role: developer
    capability: senior_developer
  - role: security_specialist  # Can add new roles without changing protocol!
    capability: security_audit

response_type: "code_review_feedback"
default_timeout_ms: 60000
```

### 3.6 Protocol Evolution & Versioning

```rust
/// Protocol versioning for backward compatibility
pub struct ProtocolVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl AgentMessage {
    /// Create message with schema version
    pub fn new_v2(
        from: AgentId,
        to: Option<AgentId>,
        message_type: &str,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            header: MessageHeader {
                // ...
            },
            content: MessageContent {
                schema_version: "2.1.0".to_string(),
                content_type: format!("application/vnd.zerobuild.{}+json", message_type),
                payload,
                artifact_reference: None,
            },
        }
    }
}

/// Migration support for protocol versions
pub struct ProtocolMigrator;

impl ProtocolMigrator {
    /// Upgrade message from old version to current
    pub fn upgrade(message: AgentMessage) -> Result<AgentMessage> {
        match message.content.schema_version.as_str() {
            "1.0.0" => Self::v1_to_v2(message),
            "2.0.0" => Self::v2_to_v2_1(message),
            _ => Ok(message),  // Already current
        }
    }
}
```

### 3.7 Benefits of Generic Protocol

| Aspect | Hardcoded Enum | Generic Protocol |
|--------|---------------|------------------|
| **Adding new agent** | Modify core code | Add YAML schema file |
| **New message type** | Add enum variant | Register new type dynamically |
| **Schema validation** | Compile-time | Runtime with JSON Schema |
| **Cross-language** | Rust only | Any language (JSON) |
| **Versioning** | Hard | Built-in schema_version |
| **Documentation** | Rust docs | Auto-generated from YAML |
| **Testing** | Recompile | Load new schema at runtime |

### 3.2 Communication Patterns

```
┌─────────────────────────────────────────────────────────────┐
│ Pattern 1: Request/Response (Synchronous)                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   Developer ──Request: Review code──→ Tester               │
│      ↑                                │                     │
│      │                                │ (process)           │
│      │                                ↓                     │
│      └────Response: Feedback────────┘                      │
│                                                             │
│   Timeout: 30s                                              │
│   Retry: 3 times                                            │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ Pattern 2: Event Broadcasting (Asynchronous)                │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   Developer ──Event: CodeReady──→ [Broadcast]              │
│                                     │                       │
│                    ┌───────────────┼───────────────┐       │
│                    ▼               ▼               ▼       │
│                 Tester          UI/UX          DevOps      │
│                                                             │
│   No response required                                       │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ Pattern 3: Command from Orchestrator                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   Orchestrator ──Command: Stop──→ Developer                │
│        ↑                           │                        │
│        │                           │ (ack)                  │
│        │                           ↓                        │
│        └────Status: Stopped──────┘                         │
│                                                             │
│   Must acknowledge within 5s                                │
└─────────────────────────────────────────────────────────────┘
```

### 3.3 Message Bus Implementation

```rust
// Agent Communication Bus
pub struct AgentMessageBus {
    // In-memory channels for active agents
    channels: DashMap<AgentId, mpsc::UnboundedSender<AgentMessage>>,
    
    // Persistent queue for offline agents
    message_queue: Arc<Mutex<Vec<QueuedMessage>>>,
    
    // Protocol handlers
    handlers: HashMap<MessageType, Box<dyn MessageHandler>>,
}

impl AgentMessageBus {
    /// Register agent with the bus
    pub async fn register(&self, agent_id: AgentId, tx: mpsc::UnboundedSender<AgentMessage>) {
        self.channels.insert(agent_id, tx);
    }
    
    /// Send message to specific agent
    pub async fn send_to(&self, to: AgentId, msg: AgentMessage) -> Result<()> {
        if let Some(channel) = self.channels.get(&to) {
            channel.send(msg)?;
            Ok(())
        } else {
            // Queue for later if agent offline
            self.queue_message(to, msg).await
        }
    }
    
    /// Broadcast to all agents of specific role
    pub async fn broadcast_to_role(&self, role: AgentRole, msg: AgentMessage) {
        for entry in self.channels.iter() {
            if entry.key().role == role {
                let _ = entry.value().send(msg.clone());
            }
        }
    }
    
    /// Request/Response with timeout
    pub async fn request<Req, Resp>(
        &self,
        to: AgentId,
        request: Req,
        timeout: Duration,
    ) -> Result<Resp> {
        let request_id = Uuid::new_v4();
        let msg = AgentMessage::Request {
            from: self.agent_id.clone(),
            to,
            request_id,
            payload: request.into(),
            timeout_ms: timeout.as_millis() as u64,
        };
        
        self.send_to(to, msg).await?;
        
        // Wait for response with timeout
        timeout(timeout, self.wait_for_response(request_id)).await?
    }
}
```

---

## 3.8 Heartbeat Protocol - Real-Time Status Monitoring

### Problem
Without heartbeat, Orchestrator doesn't know:
- Which agents are alive or crashed
- What each agent is currently doing
- Progress of long-running tasks
- When to notify user about status

### Solution

```rust
/// Heartbeat message sent by agents periodically
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatMessage {
    pub agent_id: AgentId,
    pub timestamp: DateTime<Utc>,
    pub status: AgentStatus,
    pub current_task: Option<TaskInfo>,
    pub metrics: AgentMetrics,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub task_id: Uuid,
    pub task_type: String,
    pub description: String,
    pub progress_percent: u8,
    pub started_at: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub blocked_on: Option<BlockedReason>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockedReason {
    WaitingForDependency { agent_id: AgentId, artifact: String },
    WaitingForUserInput { question: String },
    ResourceUnavailable { resource: String },
    RateLimited { retry_after: Duration },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub cpu_usage: f32,
    pub memory_usage_mb: u64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub avg_task_duration_secs: f64,
}
```

### Heartbeat Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    Heartbeat Protocol Flow                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Every 5 seconds:                                              │
│                                                                 │
│  Developer Agent ──Heartbeat──→ Orchestrator                   │
│  {                                                             │
│    status: Busy,                                               │
│    current_task: {                                             │
│      type: "implement_feature",                                │
│      progress: 65%,                                            │
│      blocked_on: WaitingForDependency {                        │
│        agent_id: uiux-designer,                                │
│        artifact: "design_spec"                                 │
│      }                                                         │
│    }                                                           │
│  }                                                             │
│                                                                 │
│  Orchestrator detects blockage → Notify User                   │
│  "Developer waiting for UI/UX design (estimated: 10 min left)"  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Implementation

```rust
/// Heartbeat manager runs in each agent
pub struct HeartbeatManager {
    agent_id: AgentId,
    bus: Arc<AgentMessageBus>,
    interval: Duration,
    last_sent: Instant,
}

impl HeartbeatManager {
    pub async fn run(mut self) {
        let mut ticker = tokio::time::interval(self.interval);
        
        loop {
            ticker.tick().await;
            
            let heartbeat = self.collect_heartbeat().await;
            
            if let Err(e) = self.send_heartbeat(heartbeat).await {
                error!("Failed to send heartbeat: {}", e);
                
                // Retry with backoff
                if self.consecutive_failures > 3 {
                    self.enter_recovery_mode().await;
                }
            }
        }
    }
    
    async fn collect_heartbeat(&self) -> HeartbeatMessage {
        HeartbeatMessage {
            agent_id: self.agent_id.clone(),
            timestamp: Utc::now(),
            status: self.get_current_status(),
            current_task: self.get_current_task(),
            metrics: self.collect_metrics(),
            capabilities: self.get_capabilities(),
        }
    }
}

/// Orchestrator's heartbeat monitor
pub struct HeartbeatMonitor {
    /// Track last heartbeat from each agent
    last_heartbeats: DashMap<AgentId, Instant>,
    /// Agents that missed heartbeats
    missed_heartbeats: DashMap<AgentId, u32>,
    /// Timeout threshold
    timeout: Duration,
}

impl HeartbeatMonitor {
    pub async fn monitor(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        
        loop {
            interval.tick().await;
            
            let now = Instant::now();
            
            for entry in self.last_heartbeats.iter() {
                let elapsed = now.duration_since(*entry.value());
                
                if elapsed > self.timeout {
                    let agent_id = entry.key();
                    let missed = self.missed_heartbeats
                        .entry(agent_id.clone())
                        .or_insert(0);
                    *missed += 1;
                    
                    if *missed >= 3 {
                        // Agent considered dead
                        self.handle_agent_timeout(agent_id).await;
                    } else {
                        warn!("Agent {} missed heartbeat ({}/3)", agent_id, missed);
                    }
                }
            }
        }
    }
    
    async fn handle_agent_timeout(&self, agent_id: &AgentId) {
        error!("Agent {} considered dead after missed heartbeats", agent_id);
        
        // Notify user
        self.notify_user(Notification::AgentTimeout {
            agent_id: agent_id.clone(),
            action: "Attempting to respawn agent".to_string(),
        }).await;
        
        // Trigger recovery
        self.orchestrator.respawn_agent(agent_id).await;
    }
}
```

### User Notifications via Heartbeat

```rust
impl Orchestrator {
    /// Process heartbeat and notify user of significant events
    pub async fn process_heartbeat(&self, heartbeat: HeartbeatMessage) {
        // Store heartbeat
        self.heartbeat_store.insert(
            heartbeat.agent_id.clone(), 
            heartbeat.clone()
        );
        
        // Check for blocked agents
        if let Some(task) = &heartbeat.current_task {
            if let Some(blocked) = &task.blocked_on {
                match blocked {
                    BlockedReason::WaitingForDependency { agent_id, artifact } => {
                        self.notify_user(format!(
                            "⏳ {} is waiting for {} to complete {} (ETA: {})",
                            heartbeat.agent_id.role,
                            agent_id.role,
                            artifact,
                            task.estimated_completion
                                .map(|t| t.to_rfc3339())
                                .unwrap_or("unknown".to_string())
                        )).await;
                    }
                    BlockedReason::WaitingForUserInput { question } => {
                        self.notify_user(format!(
                            "❓ {} needs your input: {}",
                            heartbeat.agent_id.role, question
                        )).await;
                    }
                    _ => {}
                }
            }
        }
        
        // Report progress to user periodically
        if self.should_report_progress(&heartbeat) {
            self.report_progress_to_user(&heartbeat).await;
        }
    }
}
```

---

## 3.9 Busy State Handling - Queue & Priority Management

### Problem
When Developer sends message to Designer but Designer is busy:
- ❌ Message lost or timeout
- ❌ Developer doesn't know why no response
- ❌ No queue mechanism
- ❌ No priority handling

### Solution: Message Queue with Priority

```rust
/// Message queue for each agent
pub struct AgentMessageQueue {
    /// High priority messages (commands, urgent requests)
    high_priority: Arc<Mutex<VecDeque<QueuedMessage>>>,
    /// Normal priority messages (standard requests)
    normal_priority: Arc<Mutex<VecDeque<QueuedMessage>>>,
    /// Low priority messages (notifications, updates)
    low_priority: Arc<Mutex<VecDeque<QueuedMessage>>>,
    /// Messages waiting for specific condition
    waiting: Arc<Mutex<Vec<WaitingMessage>>>,
}

#[derive(Debug, Clone)]
pub struct QueuedMessage {
    pub message: AgentMessage,
    pub priority: Priority,
    pub enqueued_at: Instant,
    pub retry_count: u32,
}

#[derive(Debug, Clone)]
pub struct WaitingMessage {
    pub message: AgentMessage,
    pub condition: WaitCondition,
    pub timeout: Option<Duration>,
}

#[derive(Debug, Clone)]
pub enum WaitCondition {
    /// Wait for agent to become available
    AgentAvailable(AgentId),
    /// Wait for artifact to be published
    ArtifactPublished(ArtifactType),
    /// Wait for specific time
    TimeReached(DateTime<Utc>),
    /// Wait for custom condition
    Custom(Box<dyn Fn() -> bool + Send + Sync>),
}
```

### Queue Management

```rust
impl AgentMessageQueue {
    /// Enqueue message with appropriate priority
    pub async fn enqueue(&self, message: AgentMessage) -> Result<()> {
        let priority = self.calculate_priority(&message);
        let queued = QueuedMessage {
            message,
            priority,
            enqueued_at: Instant::now(),
            retry_count: 0,
        };
        
        match priority {
            Priority::High => self.high_priority.lock().await.push_back(queued),
            Priority::Normal => self.normal_priority.lock().await.push_back(queued),
            Priority::Low => self.low_priority.lock().await.push_back(queued),
        }
        
        Ok(())
    }
    
    /// Dequeue next message (respects priority)
    pub async fn dequeue(&self) -> Option<QueuedMessage> {
        // Check high priority first
        if let Some(msg) = self.high_priority.lock().await.pop_front() {
            return Some(msg);
        }
        
        // Then normal
        if let Some(msg) = self.normal_priority.lock().await.pop_front() {
            return Some(msg);
        }
        
        // Finally low
        self.low_priority.lock().await.pop_front()
    }
    
    /// Wait for specific condition before delivering
    pub async fn wait_for(
        &self, 
        message: AgentMessage, 
        condition: WaitCondition,
        timeout: Option<Duration>
    ) {
        let waiting = WaitingMessage {
            message,
            condition,
            timeout,
        };
        
        self.waiting.lock().await.push(waiting);
    }
    
    /// Process waiting messages when conditions change
    pub async fn process_waiting(&self, event: ConditionEvent) {
        let mut waiting = self.waiting.lock().await;
        let ready: Vec<_> = waiting
            .extract_if(|w| self.check_condition(&w.condition, &event))
            .collect();
        drop(waiting);
        
        for msg in ready {
            self.enqueue(msg.message).await.ok();
        }
    }
}
```

### Busy State Communication Flow

```
┌────────────────────────────────────────────────────────────────────┐
│           Busy State Handling Example                               │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  Dev: "I need the design spec"                                      │
│    │                                                               │
│    ▼                                                               │
│  Designer Queue: [Busy - implementing hero section]                │
│    │                                                               │
│    ▼                                                               │
│  Designer responds:                                                │
│    "I'm busy with hero section (ETA: 5 min).                       │
│     I'll send design spec when done or queue your request?"        │
│    │                                                               │
│    ▼                                                               │
│  Options:                                                          │
│    1. Wait (block until designer free)                             │
│    2. Queue (continue other work, get notified when ready)         │
│    3. Alternative (ask UI/UX-2 if available)                       │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

### Implementation: Smart Busy Handling

```rust
impl AgentMessageBus {
    /// Send message with busy state handling
    pub async fn send_with_busy_handling(
        &self,
        to: AgentId,
        message: AgentMessage,
        strategy: BusyStrategy,
    ) -> Result<SendResult> {
        // Check if agent is busy
        let agent_status = self.get_agent_status(&to).await?;
        
        match agent_status {
            AgentStatus::Available => {
                // Agent free, send immediately
                self.send(message).await?;
                Ok(SendResult::Delivered)
            }
            AgentStatus::Busy { current_task, eta } => {
                match strategy {
                    BusyStrategy::Wait { timeout } => {
                        // Block and wait
                        info!("Waiting for agent {} to become available (ETA: {:?})", to, eta);
                        self.wait_for_agent(to, timeout).await?;
                        self.send(message).await?;
                        Ok(SendResult::DeliveredAfterWait)
                    }
                    BusyStrategy::Queue { priority } => {
                        // Queue for later
                        self.queue_message(to, message, priority).await?;
                        Ok(SendResult::Queued { 
                            position: self.queue_position(&to).await 
                        })
                    }
                    BusyStrategy::Alternative { fallback_agents } => {
                        // Try alternative agents
                        for alt in fallback_agents {
                            if self.get_agent_status(&alt).await? == AgentStatus::Available {
                                let mut alt_message = message.clone();
                                alt_message.header.to = Some(alt);
                                self.send(alt_message).await?;
                                return Ok(SendResult::DeliveredToAlternative(alt));
                            }
                        }
                        // All alternatives busy, fallback to queue
                        self.queue_message(to, message, Priority::Normal).await?;
                        Ok(SendResult::Queued { position: 1 })
                    }
                    BusyStrategy::FailFast => {
                        Err(anyhow!("Agent {} is busy with: {}", to, current_task))
                    }
                }
            }
            AgentStatus::Offline => {
                // Queue for when agent comes back
                self.queue_message(to, message, Priority::Normal).await?;
                Ok(SendResult::QueuedForReconnect)
            }
        }
    }
}

/// Strategy for handling busy agents
pub enum BusyStrategy {
    /// Wait for agent to become available
    Wait { timeout: Duration },
    /// Queue message for later processing
    Queue { priority: Priority },
    /// Try alternative agents
    Alternative { fallback_agents: Vec<AgentId> },
    /// Fail immediately if agent busy
    FailFast,
}
```

### Progress Broadcasting

```rust
/// Agent broadcasts progress updates during long tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub agent_id: AgentId,
    pub task_id: Uuid,
    pub task_name: String,
    pub percent_complete: u8,
    pub current_step: String,
    pub completed_steps: Vec<String>,
    pub remaining_steps: Vec<String>,
    pub eta_seconds: Option<u64>,
}

/// Example: Designer broadcasts progress
impl UiUxDesignerAgent {
    async fn design_homepage(&self) -> Result<()> {
        let steps = vec![
            "Research competitor designs",
            "Create wireframe",
            "Design hero section",
            "Design feature grid",
            "Design footer",
            "Export assets",
        ];
        
        for (i, step) in steps.iter().enumerate() {
            // Do work...
            self.work_on_step(step).await?;
            
            // Broadcast progress
            self.broadcast_progress(ProgressUpdate {
                agent_id: self.id.clone(),
                task_id: self.current_task.id,
                task_name: "Homepage Design".to_string(),
                percent_complete: ((i + 1) * 100 / steps.len()) as u8,
                current_step: step.to_string(),
                completed_steps: steps[..i].to_vec(),
                remaining_steps: steps[i+1..].to_vec(),
                eta_seconds: Some((steps.len() - i) * 600), // 10 min per step
            }).await?;
        }
        
        Ok(())
    }
}
```

---

## 4. Agent Pool Management

### 4.1 Pool Architecture

```rust
/// Managed pool of agents
pub struct AgentPool {
    /// Orchestrator reference
    orchestrator: Arc<Orchestrator>,
    
    /// Agent instances by ID
    agents: DashMap<AgentId, AgentInstance>,
    
    /// Agent instances by role (for quick lookup)
    agents_by_role: DashMap<AgentRole, Vec<AgentId>>,
    
    /// Warm agents (pre-initialized, ready to use)
    warm_pool: Arc<Mutex<Vec<AgentId>>>,
    
    /// Pool configuration
    config: PoolConfig,
    
    /// Metrics
    metrics: PoolMetrics,
}

pub struct AgentInstance {
    pub id: AgentId,
    pub role: AgentRole,
    pub state: AgentState,
    pub workspace: PathBuf,
    pub process: Option<Child>,  // If running as separate process
    pub handle: Option<JoinHandle<()>>,  // If running as async task
    pub last_activity: Instant,
    pub task_count: u64,
    pub health: HealthStatus,
}

pub enum AgentState {
    /// Agent initialized but not running
    Cold,
    /// Agent warming up (loading skills, etc.)
    Warming,
    /// Agent ready to accept tasks
    Warm,
    /// Agent busy with task
    Busy { task_id: Uuid, since: Instant },
    /// Agent paused
    Paused { reason: String },
    /// Agent encountered error
    Error { error: String },
    /// Agent shutting down
    ShuttingDown,
    /// Agent terminated
    Terminated,
}

pub struct PoolConfig {
    /// Max agents per role
    pub max_agents_per_role: HashMap<AgentRole, usize>,
    /// Min warm agents to keep ready
    pub min_warm_agents: HashMap<AgentRole, usize>,
    /// Idle timeout before moving to cold
    pub idle_timeout: Duration,
    /// Max agent lifetime before recycle
    pub max_lifetime: Duration,
    /// Auto-scale enabled
    pub auto_scale: bool,
}
```

### 4.2 Pool Operations

```rust
impl AgentPool {
    /// Spawn new agent of specific role
    pub async fn spawn(&self, role: AgentRole, project_id: Uuid) -> Result<AgentId> {
        // Check if can spawn more
        let current_count = self.agents_by_role.get(&role).map(|v| v.len()).unwrap_or(0);
        let max_allowed = self.config.max_agents_per_role.get(&role).copied().unwrap_or(1);
        
        if current_count >= max_allowed {
            bail!("Max agents for role {:?} reached: {}/{}", role, current_count, max_allowed);
        }
        
        // Create workspace
        let agent_id = AgentId::new(role);
        let workspace = self.create_workspace(&agent_id).await?;
        
        // Initialize agent
        let instance = AgentInstance::new(agent_id.clone(), role, workspace);
        
        // Register with pool
        self.agents.insert(agent_id.clone(), instance);
        self.agents_by_role.entry(role).or_default().push(agent_id.clone());
        
        // Start agent
        self.start_agent(&agent_id).await?;
        
        info!("Spawned agent {} of role {:?}", agent_id, role);
        Ok(agent_id)
    }
    
    /// Get or create warm agent
    pub async fn acquire(&self, role: AgentRole) -> Result<AgentId> {
        // Try to find warm agent
        if let Some(agent_id) = self.find_warm_agent(role).await {
            self.mark_busy(&agent_id).await?;
            return Ok(agent_id);
        }
        
        // Try to warm up cold agent
        if let Some(agent_id) = self.find_cold_agent(role).await {
            self.warm_up(&agent_id).await?;
            self.mark_busy(&agent_id).await?;
            return Ok(agent_id);
        }
        
        // Spawn new agent
        self.spawn(role, self.orchestrator.project_id()).await
    }
    
    /// Release agent back to pool
    pub async fn release(&self, agent_id: &AgentId) -> Result<()> {
        if let Some(mut agent) = self.agents.get_mut(agent_id) {
            agent.state = AgentState::Warm;
            agent.last_activity = Instant::now();
            
            // Clear task-specific memory but keep skills/context
            self.clear_task_memory(agent_id).await?;
        }
        Ok(())
    }
    
    /// Terminate agent
    pub async fn terminate(&self, agent_id: &AgentId, reason: &str) -> Result<()> {
        info!("Terminating agent {}: {}", agent_id, reason);
        
        if let Some((id, mut agent)) = self.agents.remove(agent_id) {
            agent.state = AgentState::ShuttingDown;
            
            // Graceful shutdown
            if let Some(handle) = agent.handle {
                handle.abort();
            }
            
            // Save final state
            self.save_agent_state(&id, &agent).await?;
            
            // Cleanup if needed
            if self.config.preserve_workspaces {
                // Archive workspace
                self.archive_workspace(&agent.workspace).await?;
            } else {
                // Delete workspace
                fs::remove_dir_all(&agent.workspace).await?;
            }
        }
        
        Ok(())
    }
    
    /// Health check all agents
    pub async fn health_check(&self) -> PoolHealthReport {
        let mut report = PoolHealthReport::default();
        
        for entry in self.agents.iter() {
            let agent = entry.value();
            match self.check_agent_health(agent).await {
                Ok(health) => {
                    if !health.healthy {
                        report.unhealthy.push(agent.id.clone());
                        // Attempt recovery
                        self.attempt_recovery(&agent.id).await;
                    }
                }
                Err(e) => {
                    report.errors.push((agent.id.clone(), e.to_string()));
                }
            }
        }
        
        report
    }
}
```

### 4.3 Warm/Cold Pool Management

```rust
/// Background task to manage pool
pub async fn pool_manager(pool: Arc<AgentPool>) {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    
    loop {
        interval.tick().await;
        
        // 1. Check idle agents
        pool.recycle_idle_agents().await;
        
        // 2. Ensure minimum warm agents
        pool.ensure_warm_agents().await;
        
        // 3. Health check
        pool.health_check().await;
        
        // 4. Auto-scale if enabled
        if pool.config.auto_scale {
            pool.auto_scale().await;
        }
        
        // 5. Metrics update
        pool.update_metrics().await;
    }
}

impl AgentPool {
    /// Move idle agents to cold state
    async fn recycle_idle_agents(&self) {
        let now = Instant::now();
        let timeout = self.config.idle_timeout;
        
        for mut entry in self.agents.iter_mut() {
            let agent = entry.value_mut();
            
            if let AgentState::Warm = agent.state {
                if now.duration_since(agent.last_activity) > timeout {
                    info!("Recycling idle agent {} to cold state", agent.id);
                    agent.state = AgentState::Cold;
                    
                    // Persist state
                    self.save_agent_state(&agent.id, agent).await.ok();
                    
                    // Release resources
                    if let Some(handle) = agent.handle.take() {
                        handle.abort();
                    }
                }
            }
        }
    }
    
    /// Ensure minimum warm agents per role
    async fn ensure_warm_agents(&self) {
        for (role, min_count) in &self.config.min_warm_agents {
            let warm_count = self.count_warm_agents(*role).await;
            let needed = *min_count - warm_count;
            
            if needed > 0 {
                info!("Warming up {} agents for role {:?}", needed, role);
                
                for _ in 0..needed {
                    // Try to warm existing cold agents first
                    if let Some(agent_id) = self.find_cold_agent(*role).await {
                        self.warm_up(&agent_id).await.ok();
                    } else {
                        // Spawn new agents
                        self.spawn(*role, self.orchestrator.project_id()).await.ok();
                    }
                }
            }
        }
    }
}
```

---

## 5. Agent Lifecycle

### 5.1 Lifecycle States

```
┌──────────┐
│  Cold    │ ← Initial state (workspace exists, agent not running)
└────┬─────┘
     │ spawn()
     ▼
┌──────────┐
│ Warming  │ ← Loading skills, initializing memory, connecting to bus
└────┬─────┘
     │
     ▼
┌──────────┐
│  Warm    │ ← Ready to accept tasks (keep in warm pool)
└────┬─────┘
     │ acquire()
     ▼
┌──────────┐
│  Busy    │ ← Executing task
└────┬─────┘
     │ release() / complete
     ▼
┌──────────┐
│  Warm    │ ← Return to warm pool or go cold after timeout
└────┬─────┘
     │ idle timeout / error
     ▼
┌──────────┐
│  Cold    │ ← Persist state, release resources
└────┬─────┘
     │ terminate()
     ▼
┌──────────┐
│Terminated│ ← Cleanup workspace
└──────────┘
```

### 5.2 State Transitions

```rust
impl AgentInstance {
    /// Transition to new state
    pub async fn transition(&mut self, new_state: AgentState) -> Result<()> {
        let old_state = std::mem::replace(&mut self.state, new_state.clone());
        
        info!(
            "Agent {} state transition: {:?} → {:?}",
            self.id, old_state, new_state
        );
        
        match (&old_state, &new_state) {
            (AgentState::Cold, AgentState::Warming) => {
                self.on_warming().await?;
            }
            (AgentState::Warming, AgentState::Warm) => {
                self.on_warm().await?;
            }
            (AgentState::Warm, AgentState::Busy { task_id, .. }) => {
                self.on_task_start(*task_id).await?;
            }
            (AgentState::Busy { .. }, AgentState::Warm) => {
                self.on_task_complete().await?;
            }
            (AgentState::Warm, AgentState::Cold) => {
                self.on_cold().await?;
            }
            (AgentState::Error { error }, _) => {
                self.on_error_recovery(error).await?;
            }
            _ => {}
        }
        
        // Persist state change
        self.save_state().await?;
        
        Ok(())
    }
    
    async fn on_warming(&mut self) -> Result<()> {
        // Load skills
        self.load_skills().await?;
        
        // Initialize memory
        self.init_memory().await?;
        
        // Register with message bus
        self.register_with_bus().await?;
        
        // Notify orchestrator
        self.notify_orchestrator(AgentEvent::AgentReady).await?;
        
        Ok(())
    }
}
```

---

## 6. Implementation Plan

### Phase 1: Foundation (Week 1-2)
- [ ] Create workspace structure (`~/.zerobuild/workspaces/`)
- [ ] Implement `AgentWorkspace` struct
- [ ] Create per-agent `identity.md` templates
- [ ] Move sandbox from `/tmp` to workspace
- [ ] Update `SandboxCreateTool` to use workspace path

### Phase 2: Communication Protocol (Week 2-3)
- [ ] Define `AgentMessage` types
- [ ] Implement `AgentMessageBus`
- [ ] Add request/response pattern
- [ ] Add event broadcasting
- [ ] Write protocol tests

### Phase 3: Agent Pool (Week 3-4)
- [ ] Implement `AgentPool` struct
- [ ] Add warm/cold pool management
- [ ] Implement `spawn/acquire/release/terminate` operations
- [ ] Add health checking
- [ ] Create pool manager background task

### Phase 4: Integration (Week 4-5)
- [ ] Refactor `FactoryWorkflow` to use AgentPool
- [ ] Update `run_agent_simple/agentic` to use pooled agents
- [ ] Implement agent-to-agent communication in workflow
- [ ] Add metrics and monitoring

### Phase 5: Testing & Documentation (Week 5-6)
- [ ] Write integration tests
- [ ] Test parallel execution
- [ ] Test communication protocol
- [ ] Update all documentation
- [ ] Migration guide for existing users

---

## 7. Migration Strategy

### 7.1 Backward Compatibility

```rust
/// Migration helper
pub struct WorkspaceMigration;

impl WorkspaceMigration {
    /// Check if migration needed
    pub fn needs_migration() -> bool {
        // Check for old /tmp sandbox structure
        Path::new("/tmp/zerobuild-sandbox-").exists()
    }
    
    /// Migrate from old structure to new
    pub async fn migrate() -> Result<()> {
        info!("Migrating to workspace-based architecture...");
        
        // 1. Create new workspace directory
        fs::create_dir_all(WORKSPACE_ROOT).await?;
        
        // 2. Move existing projects to new structure
        Self::migrate_projects().await?;
        
        // 3. Create agent workspaces for active factory runs
        Self::migrate_active_agents().await?;
        
        // 4. Update config
        Self::update_config().await?;
        
        info!("Migration complete");
        Ok(())
    }
}
```

### 7.2 Feature Flags

```toml
[factory]
enabled = true

# New options
workspace_isolation = true       # Enable per-agent workspaces
agent_pool = true                # Enable agent pool management
communication_protocol = true    # Enable IACP

# Backward compatibility
legacy_sandbox_mode = false      # Use old /tmp sandbox (for rollback)
```

---

## 8. Configuration Example

### 8.1 Minimal Config

```toml
# ~/.zerobuild/config.toml
[factory]
enabled = true

# Agent pool settings
[factory.pool]
auto_scale = true
idle_timeout_seconds = 300  # 5 minutes

[[factory.pool.warm_agents]]
role = "developer"
min_count = 1
max_count = 3

[[factory.pool.warm_agents]]
role = "tester"
min_count = 1
max_count = 2
```

### 8.2 Advanced Config

```toml
[factory]
enabled = true
max_ping_pong_iterations = 5

# Workspace settings
[factory.workspace]
root = "~/.zerobuild/workspaces"
preserve_on_exit = true
archive_after_days = 7

# Communication protocol
[factory.protocol]
request_timeout_ms = 30000
max_retries = 3
enable_encryption = true

# Per-role configuration
[factory.roles.developer]
max_agents = 3
warm_pool_size = 1
skills = ["nextjs", "rust", "testing"]

[factory.roles.tester]
max_agents = 2
warm_pool_size = 1
skills = ["e2e", "unit-testing", "security"]
```

---

## 9. Benefits Summary

| Aspect | Before | After |
|--------|--------|-------|
| **Isolation** | Shared sandbox | Per-agent workspace |
| **Identity** | Single `IDENTITY.md` | Per-agent identity + skills |
| **Communication** | Blackboard only | Protocol + Blackboard |
| **Scalability** | Spawn on demand | Managed pool with warm agents |
| **Debugging** | Hard to trace | Clear agent separation |
| **Rollback** | All or nothing | Per-agent rollback |
| **Customization** | Limited | Per-agent config, skills, memory |

---

## 10. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Disk usage** | High (multiple workspaces) | Auto-cleanup, archiving, size limits |
| **Complexity** | Increased | Clear documentation, gradual rollout |
| **Migration** | Breaking change | Backward compat mode, migration tool |
| **Performance** | Pool overhead | Benchmark, optimize warm-up time |
| **Debugging** | Harder distributed system | Centralized logging, tracing |

---

## Next Steps

1. **Review this plan** with core team
2. **Create RFC** for community feedback
3. **Prototype Phase 1** (workspace isolation)
4. **Benchmark** performance impact
5. **Gradual rollout** with feature flags

---

*Document created for ZeroBuild architecture evolution. Subject to change based on implementation feedback.*
