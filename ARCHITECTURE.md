# ARCHITECTURE.md — ZeroBuild: The Autonomous Software Factory

## Vision

ZeroBuild is a **Virtual Software Company** powered entirely by AI. Through a **Hierarchical Multi-Agent System**, a user provides a raw idea in natural language, and ZeroBuild automatically assembles a team of AI specialists — Orchestrator (CEO), Business Analyst, UI/UX Designer, Developer, Tester, and DevOps Engineer — that coordinate autonomously to automate the entire software development lifecycle and deliver a production-ready product.

**Core promise:** From idea to working software in minutes, not months. Zero coding. Zero management. Ultra-low cost.

---

## Multi-Agent Factory Architecture

### Agent Role Hierarchy — The Virtual Software Company

```
User provides idea (any channel: Telegram / Discord / Slack / CLI)
    │
    ▼
┌───────────────────────────────────────┐
│  🏢 Orchestrator (CEO) │
│  • Receives idea, analyzes feasibility│
│  • Creates project plan               │
│  • Spawns specialized sub-agents      │
│  • Delegates tasks & supervises       │
│  • Reports progress to user           │
└───────┬───────────────────────────────┘
        │
        ├── Phase 1: Analysis
        │   └── Business Analyst Agent
        │       → Produces PRD (Product Requirements Document)
        │
        ├── Phase 2: Parallel Build
        │   ├── UI/UX Designer Agent  → Design spec + wireframes
        │   ├── Developer Agent       → Source code
        │   └── Tester Agent          → Test cases
        │
        ├── Phase 3: Integration Loop
        │   ├── Developer Agent  ←→  Tester Agent
        │   │   (ping-pong until tests pass, max N iterations)
        │   └── Blackboard: test results, code patches
        │
        └── Phase 4: Deployment
            └── DevOps Agent → Deploy config, push to GitHub
```

### Agent Roles

| Role | Responsibility | Inputs | Outputs |
|------|---------------|--------|---------|
| **Orchestrator** | Workflow coordination, phase management, user communication | User idea | Final deliverable |
| **Business Analyst** | Requirements analysis, PRD generation | User idea | PRD artifact |
| **UI/UX Designer** | Design specifications, component structure | PRD | Design spec artifact |
| **Developer** | Code generation, implementation | PRD + Design spec | Source code artifact |
| **Tester** | Test case generation, test execution | PRD + Source code | Test cases + results |
| **DevOps** | Deployment configuration, GitHub push | Source code + passing tests | Deploy config |

---

## Workflow Phases

### Phase 0: Planning & User Confirmation

Before any agents are spawned, the **Orchestrator analyzes the user request and creates a detailed execution plan** with timeline estimates. This plan must be **approved by the user** before proceeding.

```
User Idea → [Orchestrator] → Execution Plan → User Approval → Factory Execution
                              ↓
                         ┌────┴──────────────────────────────────────────────┐
                         │ 📋 EXECUTION PLAN                                  │
                         ├────────────────────────────────────────────────────┤
                         │ Project: E-commerce Landing Page                  │
                         │ Complexity: High                                   │
                         │                                                    │
                         │ 👥 Team (5 agents):                               │
                         │ 1. Business Analyst (15 min)                      │
                         │    └─ Write PRD with user stories                 │
                         │                                                    │
                         │ 2. UI/UX Designer (20 min)                        │
                         │    └─ Create design spec + wireframes             │
                         │                                                    │
                         │ 3. Developer (30 min)                             │
                         │    └─ Implement Next.js app with components       │
                         │                                                    │
                         │ 4. Tester (15 min)                                │
                         │    └─ Write + run test cases                      │
                         │                                                    │
                         │ 5. DevOps Engineer (5 min)                        │
                         │    └─ Deploy to GitHub                            │
                         │                                                    │
                         │ ⏱️ Total Estimated Time: 85 minutes               │
                         │ 💰 Estimated Cost: $0.85 (API tokens)             │
                         │                                                    │
                         │ Type "START" to proceed or "CHANGE" to modify     │
                         └────────────────────────────────────────────────────┘
```

**Why User Confirmation Matters:**
- ✅ **Transparency** - User knows exactly what will happen and how long it takes
- ✅ **Control** - User can modify requirements or scope before work begins
- ✅ **Cost awareness** - User sees estimated API cost upfront
- ✅ **Expectation management** - Clear timeline prevents "why is it taking so long?"

**Orchestrator Planning Algorithm:**

```rust
pub struct ExecutionPlan {
    pub project_name: String,
    pub complexity: Complexity,
    pub required_agents: Vec<AgentAssignment>,
    pub parallel_groups: Vec<Vec<AgentRole>>,
    pub dependencies: Vec<Dependency>,
    pub total_estimated_duration: Duration,
    pub estimated_cost_usd: f64,
    pub risks: Vec<RiskAssessment>,
}

pub struct AgentAssignment {
    pub role: AgentRole,
    pub estimated_duration: Duration,
    pub deliverables: Vec<String>,
    pub dependencies: Vec<AgentRole>,
}

impl Orchestrator {
    pub async fn create_execution_plan(&self, user_idea: &str) -> Result<ExecutionPlan> {
        // 1. Analyze complexity via LLM
        let analysis = self.analyze_complexity(user_idea).await?;
        
        // 2. Determine required agents
        let required_agents = self.determine_team_composition(&analysis);
        
        // 3. Calculate parallelization
        let parallel_groups = self.optimize_parallelization(&required_agents);
        
        // 4. Estimate time and cost
        let (duration, cost) = self.estimate_resources(&required_agents);
        
        // 5. Identify risks
        let risks = self.assess_risks(&analysis);
        
        Ok(ExecutionPlan {
            project_name: analysis.project_name,
            complexity: analysis.complexity,
            required_agents,
            parallel_groups,
            dependencies: analysis.dependencies,
            total_estimated_duration: duration,
            estimated_cost_usd: cost,
            risks,
        })
    }
    
    pub async fn present_plan_to_user(&self, plan: &ExecutionPlan) -> Result<UserDecision> {
        let formatted = self.format_plan(plan);
        
        self.notify_user(formatted).await;
        
        // Wait for user response
        loop {
            let response = self.wait_for_user_input(Duration::from_secs(300)).await?;
            
            match response.trim().to_lowercase().as_str() {
                "start" | "yes" | "go" => return Ok(UserDecision::Proceed),
                "change" | "modify" => {
                    let modifications = self.gather_modifications().await?;
                    return Ok(UserDecision::Modify(modifications));
                }
                "cancel" | "no" => return Ok(UserDecision::Cancel),
                _ => {
                    self.notify_user("Please respond with: START, CHANGE, or CANCEL").await;
                }
            }
        }
    }
}
```

**User Interaction Flow:**

```
User: "Build me an e-commerce site"

Orchestrator:
"I've analyzed your request. Here's my plan:

📋 EXECUTION PLAN - E-commerce Platform
═══════════════════════════════════════

Complexity: HIGH (estimated 2-3 hours)

Team Required:
  👔 Business Analyst (20 min) - Write detailed PRD
  🎨 UI/UX Designer (30 min) - Design 5 pages  
  💻 Developer (60 min) - Build with Next.js + Stripe
  🧪 Tester (20 min) - Test checkout flow
  🚀 DevOps (10 min) - Deploy to Vercel

⏱️ Timeline: 2 hours 20 minutes
💰 Est. Cost: $2.50 in API tokens

⚠️  Note: Payment integration may require your Stripe API keys

Options:
  [START] - Begin with this plan
  [SCOPE] - Reduce features (1 hour, $1.20)
  [CHANGE] - Modify requirements
  [CANCEL] - Do nothing

What would you like to do?"

User: "SCOPE"

Orchestrator:
"Got it! Here's the reduced scope:

📋 EXECUTION PLAN - E-commerce MVP
══════════════════════════════════

Complexity: MEDIUM (1 hour)

Team Required:
  👔 Business Analyst (10 min) - Core features only
  🎨 UI/UX Designer (15 min) - 2 pages (home + product)
  💻 Developer (25 min) - Next.js without payments
  🧪 Tester (10 min) - Basic tests
  🚀 DevOps (5 min) - Deploy

⏱️ Timeline: 1 hour 5 minutes  
💰 Est. Cost: $1.20

✅ Includes: Product catalog, shopping cart, responsive design
❌ Excludes: Payment processing, user accounts, admin panel

Ready to start? (START / CHANGE / CANCEL)"

User: "START"

Orchestrator: "🚀 Launching team! I'll keep you updated on progress..."
[Spawns agents and begins execution]
```

---

### Phase 1: Analysis (Sequential)

```
User Idea → [BA Agent] → PRD → Blackboard
```

The Business Analyst agent analyzes the user's idea and produces a structured Product Requirements Document. This is the foundation for all subsequent work.

### Phase 2: Parallel Build

```
PRD → [UI/UX Agent]  → Design Spec  → Blackboard
    → [Developer]     → Source Code  → Blackboard
    → [Tester]        → Test Cases   → Blackboard
```

Three agents run concurrently via `tokio::join!`. Each reads the PRD from the blackboard and produces their respective artifacts.

### Phase 3: Integration Loop

```
Source Code + Test Cases → [Developer] ←→ [Tester]
                           (max N iterations, configurable)
```

The Developer and Tester agents enter a ping-pong loop:
1. Tester runs test cases against source code
2. If tests fail → Developer reads test results, fixes code
3. Repeat until all tests pass or iteration cap reached

Default cap: 5 iterations (configurable via `factory.max_ping_pong_iterations`).

### Phase 4: Deployment (Sequential)

```
Passing Source Code → [DevOps Agent] → Deploy Config → GitHub Push
```

Only triggers when Phase 3 produces passing test results.

---

## Blackboard Data Flow

The **Blackboard** is the shared state layer for inter-agent communication. It is built on top of the existing `InMemoryMessageBus` and `SharedContextEntry` coordination primitives.

```
┌──────────────────────────────────────────┐
│              Blackboard                  │
│                                          │
│  artifact:prd          → PRD JSON        │
│  artifact:design_spec  → Design Spec     │
│  artifact:source_code  → Code Manifest   │
│  artifact:test_cases   → Test Suite      │
│  artifact:test_results → Pass/Fail       │
│  artifact:deploy_config→ Deploy Config   │
│                                          │
│  Uses ContextPatch envelopes with        │
│  optimistic-locking versioning           │
└──────────────────────────────────────────┘
```

Agents publish artifacts via `publish_artifact()` and read them via `read_artifact()`. Version conflicts are handled by the existing `ContextVersionMismatch` error in the coordination protocol.

---

## Agent Workspace Isolation (Future)

### Current Limitation
Currently all agents share the same sandbox (`/tmp/zerobuild-sandbox-{uuid}/`), which can lead to:
- Accidental file overwrites between agents
- Difficulty debugging which agent created which file
- No per-agent rollback capability
- Shared identity and skills across all agents

### Proposed Workspace Architecture

```
~/.zerobuild/
├── config.toml                    # Global config
├── workspaces/                    # Each agent has isolated workspace
│   ├── orchestrator-{uuid}/
│   │   ├── .agent/
│   │   │   ├── identity.md        # Orchestrator personality
│   │   │   ├── skills/            # Per-agent skills
│   │   │   ├── memory/            # Conversation history
│   │   │   └── state.json         # Agent state
│   │   └── projects/
│   ├── ba-{uuid}/
│   │   ├── .agent/
│   │   │   ├── identity.md        # BA-specific identity
│   │   │   └── skills/
│   │   │       ├── requirements-analysis.md
│   │   │       └── prd-writing.md
│   │   └── sandbox/               # Isolated work area
│   ├── dev-{uuid}/
│   │   ├── .agent/
│   │   └── sandbox/
│   │       └── project/           # Source code isolated
│   └── [other agents...]
│
└── shared/
    ├── blackboard/               # Shared artifacts
    └── protocols/                # Message type definitions
```

**Benefits:**
- ✅ True agent isolation - one agent cannot affect another's files
- ✅ Per-agent identity, skills, and memory
- ✅ Independent rollback per agent
- ✅ Better debugging and audit trails

---

## Inter-Agent Communication Protocol (IACP)

### Generic Message Protocol

Instead of hardcoded Rust enums, the new protocol uses generic message envelopes with dynamic type registration:

```rust
pub struct AgentMessage {
    pub header: MessageHeader {
        pub message_id: Uuid,
        pub message_type: String,  // Dynamic: "code_review", "request_clarification", etc.
        pub from: AgentId,
        pub to: Option<AgentId>,
        pub intent: MessageIntent,
    },
    pub content: MessageContent {
        pub schema_version: String,
        pub content_type: String,
        pub payload: serde_json::Value,  // Validated against JSON Schema
    }
}
```

### Message Type Registry

Message types are defined in YAML and loaded dynamically:

```yaml
# ~/.zerobuild/protocols/messages/code_review.yaml
message_type: "code_review"
schema:
  type: object
  required: [file_path, code_snippet]
  properties:
    file_path: { type: string }
    code_snippet: { type: string }

handlers:
  - role: tester
    capability: quality_assurance
  - role: security_specialist  # Easy to add new roles!
    capability: security_audit
```

**Benefits:**
- ✅ Add new message types without recompiling
- ✅ Capability-based routing (not hardcoded role pairs)
- ✅ JSON Schema validation
- ✅ Cross-language support

---

## Heartbeat Protocol & Status Monitoring

### Problem
Without heartbeat, the Orchestrator cannot:
- Detect crashed or hung agents
- Know what each agent is currently doing
- Report progress to users
- Handle blocked dependencies

### Solution

```rust
pub struct HeartbeatMessage {
    pub agent_id: AgentId,
    pub status: AgentStatus,
    pub current_task: Option<TaskInfo> {
        pub task_type: String,
        pub progress_percent: u8,
        pub blocked_on: Option<BlockedReason>,
    },
    pub timestamp: DateTime<Utc>,
}

pub enum BlockedReason {
    WaitingForDependency { agent_id: AgentId, artifact: String },
    WaitingForUserInput { question: String },
    ResourceUnavailable { resource: String },
}
```

### Heartbeat Flow

```
Every 5 seconds:

Agent → Heartbeat → Orchestrator
       { status: Busy, 
         task: { progress: 65%, 
                 blocked_on: WaitingForDesigner } }
                    ↓
         Orchestrator notifies user:
         "⏳ Developer waiting for UI/UX design 
            (estimated: 10 min left)"
```

### User Notifications

The Orchestrator uses heartbeat data to:
- Report real-time progress: "Designer is 65% done with homepage"
- Notify about blockages: "Developer waiting for design spec"
- Alert on failures: "Tester found 3 bugs, Developer fixing..."
- ETA updates: "Estimated completion in 15 minutes"

---

## Busy State Handling & Message Queue

### Problem
When Agent A sends a message to busy Agent B:
- Message may timeout or get lost
- Agent A doesn't know why no response
- No mechanism to queue or retry

### Solution: Priority Queue with Smart Routing

```rust
pub struct AgentMessageQueue {
    pub high_priority: VecDeque<QueuedMessage>,   // Urgent commands
    pub normal_priority: VecDeque<QueuedMessage>, // Standard requests
    pub low_priority: VecDeque<QueuedMessage>,    // Notifications
    pub waiting: Vec<WaitingMessage>,             // Conditional delivery
}

pub enum BusyStrategy {
    Wait { timeout: Duration },
    Queue { priority: Priority },
    Alternative { fallback_agents: Vec<AgentId> },
    FailFast,
}
```

### Usage Example

```rust
// Developer needs design spec from UI/UX
let result = bus.send_with_busy_handling(
    to: uiux_designer,
    message: design_request,
    strategy: BusyStrategy::Alternative {
        fallback_agents: vec![uiux_designer_2, uiux_lead]
    }
).await?;

// Result could be:
// - Delivered (designer was free)
// - DeliveredToAlternative(uiux_designer_2)
// - Queued { position: 2 }
// - Error (all designers busy)
```

---

## Agent Pool Management

### Warm/Cold Agent Pool

```rust
pub struct AgentPool {
    pub agents: DashMap<AgentId, AgentInstance>,
    pub warm_pool: Vec<AgentId>,     // Ready to use
    pub cold_pool: Vec<AgentId>,     // Initialized but not running
}

pub enum AgentState {
    Cold,       // Workspace exists, agent not running
    Warming,    // Loading skills, connecting to bus
    Warm,       // Ready to accept tasks
    Busy { task_id: Uuid, since: Instant },
    Paused { reason: String },
    Error { error: String },
    ShuttingDown,
    Terminated,
}
```

### Pool Operations

```rust
impl AgentPool {
    // Get or create warm agent
    pub async fn acquire(&self, role: AgentRole) -> Result<AgentId> {
        // 1. Try to find warm agent
        // 2. Try to warm up cold agent
        // 3. Spawn new agent if needed
    }
    
    // Release back to pool
    pub async fn release(&self, agent_id: &AgentId) {
        // Clear task memory, return to warm pool
    }
    
    // Auto-scale based on demand
    pub async fn auto_scale(&self) {
        // Monitor queue lengths, spawn additional agents
    }
}
```

### Benefits

- ✅ **Reduced latency** - Warm agents start immediately (no 5-10s init)
- ✅ **Auto-scaling** - Spawn more agents during high load
- ✅ **Resource efficiency** - Cold agents use minimal resources
- ✅ **Health monitoring** - Automatic recovery from failures

---

## Design Decisions (Updated)

1. **Enabled by default** — `factory.enabled = true`. The `factory_build` tool is always available; the agent autonomously decides when to use it based on task complexity.

2. **Phase 0: User Confirmation** — Orchestrator must create detailed execution plan with timeline/cost estimates and obtain user approval before spawning agents. This provides transparency and control.

3. **Workspace Isolation** (Future) — Each agent gets isolated workspace with dedicated identity, skills, and memory. Enables per-agent rollback and better debugging.

4. **Generic Communication Protocol** — Dynamic message types via YAML + JSON Schema instead of hardcoded enums. Enables adding new agent types without modifying core code.

5. **Heartbeat Protocol** — Agents send heartbeats every 5 seconds with status, progress, and blockages. Enables real-time user notifications and failure detection.

6. **Busy State Handling** — Priority message queues with smart routing (wait, queue, alternative agents). Prevents message loss when agents are busy.

7. **Agent Pool Management** — Warm/cold pool with auto-scaling. Reduces latency by keeping agents ready and scales based on demand.

8. **Hard iteration cap** — Prevents infinite dev-test loops. Configurable, default 5.

9. **No new traits** — Factory uses existing `Tool`, `Provider`, and coordination traits.

10. **Backward compatible** — New features are opt-in via feature flags; existing single-agent mode unaffected.

### Reuse Existing Primitives

The factory module builds on existing infrastructure rather than rewriting:

| Existing Primitive | Factory Usage |
|---|---|
| `DelegateTool` | Pattern for spawning sub-agents with filtered tool access |
| `DelegateAgentConfig` | Per-role provider/model/prompt configuration |
| `InMemoryMessageBus` | Blackboard transport layer |
| `SharedContextEntry` + `ContextPatch` | Artifact storage with versioned writes |
| `run_tool_call_loop` | Agent execution engine for each role |

### Module Structure (Current)

```
src/factory/
├── mod.rs                  # Module exports
├── roles.rs                # AgentRole enum, RoleConfig, system prompts
├── blackboard.rs           # Blackboard struct wrapping InMemoryMessageBus
├── workflow.rs             # WorkflowPhase state machine, FactoryWorkflow
└── orchestrator_tool.rs    # FactoryOrchestratorTool (Tool trait impl)
```

### Module Structure (Future - Workspace Isolation)

```
src/factory/
├── mod.rs
├── roles.rs
├── blackboard.rs
├── workflow.rs
├── orchestrator_tool.rs
├── workspace/              # NEW: Per-agent workspace management
│   ├── mod.rs
│   ├── manager.rs          # Workspace lifecycle
│   └── isolation.rs        # Sandboxing per agent
├── protocol/               # NEW: Inter-agent communication
│   ├── mod.rs
│   ├── message.rs          # Generic message types
│   ├── registry.rs         # Dynamic type registry
│   ├── bus.rs              # Message routing
│   ├── heartbeat.rs        # Health monitoring
│   └── queue.rs            # Busy state handling
└── pool/                   # NEW: Agent pool management
    ├── mod.rs
    ├── pool.rs             # AgentPool implementation
    ├── lifecycle.rs        # Warm/cold state transitions
    └── health.rs           # Health checking
```

### Configuration

```toml
[factory]
enabled = true                     # Default: true. Agent autonomously decides when to use factory
max_ping_pong_iterations = 5       # Dev-Tester loop cap

[factory.provider_overrides.business_analyst]
provider = "openrouter"
model = "anthropic/claude-sonnet-4-6"
# ... per-role DelegateAgentConfig overrides
```

---

## Design Decisions

1. **Enabled by default** — `factory.enabled = true`. The `factory_build` tool is always available; the agent autonomously decides when to use it based on task complexity.
2. **Same sandbox** — All factory agents share the same sandbox filesystem, enabling collaborative file access.
3. **Hard iteration cap** — Prevents infinite dev-test loops. Configurable, default 5.
4. **No new traits** — Factory uses existing `Tool`, `Provider`, and coordination traits.
5. **Backward compatible** — No breaking changes to any existing interface.

---

## Development Roadmap

### Phase A: Foundation (Current ✓)
- [x] Factory module structure (`src/factory/`)
- [x] Role definitions with system prompts
- [x] Blackboard on top of `InMemoryMessageBus`
- [x] Workflow state machine
- [x] `factory_build` tool registration
- [x] Phase 0: Planning with user confirmation

### Phase B: Workspace Isolation (Next)
- [ ] Per-agent workspace structure (`~/.zerobuild/workspaces/`)
- [ ] Move sandbox from `/tmp` to agent workspace
- [ ] Per-agent `identity.md` and `skills/` folder
- [ ] Workspace lifecycle management
- [ ] Migration tool from old structure

### Phase C: Communication Protocol (Next)
- [ ] Generic `AgentMessage` envelope
- [ ] Dynamic message type registry
- [ ] Message bus with routing
- [ ] Request/response pattern
- [ ] Event broadcasting
- [ ] YAML-based message type definitions

### Phase D: Monitoring & Reliability (Next)
- [ ] Heartbeat protocol (5-second intervals)
- [ ] Real-time progress reporting
- [ ] User notification system
- [ ] Agent health monitoring
- [ ] Automatic failure recovery

### Phase E: Queue Management (Next)
- [ ] Priority message queues
- [ ] Busy state handling
- [ ] Smart routing (alternative agents)
- [ ] Message persistence and retry

### Phase F: Pool Management (Next)
- [ ] Warm/cold agent pool
- [ ] Auto-scaling based on demand
- [ ] Pool lifecycle management
- [ ] Resource optimization

### Phase G: Advanced Features (Future)
- [ ] Cross-project learning from previous builds
- [ ] Custom role definitions via config
- [ ] Agent performance metrics and optimization
- [ ] Distributed agent execution
- [ ] Plugin system for third-party agents

---

## Cleanup Notes

The following areas may benefit from refocusing as the factory matures:

- **Single-agent assumptions** in `IDENTITY.md` and system prompts — update to acknowledge factory mode
- **Tool registry** — factory agents need filtered tool access per role
- **Progress reporting** — factory phases should emit structured progress events
- **Error escalation** — factory should aggregate errors across agents for user-facing reports
