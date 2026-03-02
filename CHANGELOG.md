# Changelog

All notable changes to ZeroBuild will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### GitHub Connector Enhancements
- **New Comment Tools**: Added 3 new tools for commenting on repositories
  - `github_comment_issue`: Add comments to existing issues
  - `github_comment_pr`: Add general comments to PRs (non-inline)
  - `github_reply_comment`: Reply to existing comments (threaded)
  - Use cases: Q&A on issues, PR feedback, code review discussions
- **Auto-Configured Fallback Providers**: Onboarding wizard now suggests fallback providers
  - Based on primary provider selection (e.g., kimi-code → moonshot → openrouter → anthropic)
  - Improves reliability when primary provider fails
  - Configurable via `[reliability].fallback_providers`

#### Documentation
- **AGENTS.md**: New sections 5.14, 5.15, 5.16
  - 5.14: PR Code Review Workflow
  - 5.15: Issue Planner Workflow (structured issue creation)
  - 5.16: GitHub Comment Workflow
- **Hashtag Workflows**: Added `#review`, `#summarize`, `#plan`, `#comment` triggers

#### Vibe Coding UX Improvements
- **Plan Confirmation Flow** (`task_plan` tool): New `propose`, `confirm`, and `reject` actions
  - Agent must propose a build plan and get user confirmation before building
  - Plans include project name, tech stack, features, and steps
  - Standard format: "📝 BUILD PLAN" with structured output
- **Session Resumption**: Active project context is now persisted to memory
  - New `ActiveProject` struct with name, description, tech_stack, github_repo, preview_url, status
  - `save_project_context()` and `load_project_context()` helpers
  - Agent welcomes returning users: "Welcome back! You're building [project]..."
- **Progress Reporting**: Required plain-language status messages before significant tool calls
  - "Starting up the build environment..." (sandbox_create)
  - "Creating your project..." (npx create-next-app)
  - "Installing dependencies..." (npm install)
  - "Starting the dev server..." (npm run dev)
  - "Getting your preview link..." (get_preview_url)
  - "Pushing your code to GitHub..." (github_push)
- **Auto-Test After Build**: Agent automatically verifies dev server responds with HTTP 200
  before reporting build complete
- **Product Advisor Tool** (`product_advisor`): Generates improvement suggestions after deploy
  - Categories: UX, Performance, Features, Security, Monetization, or All
  - Standard output format with 🔴 HIGH / 🟡 MEDIUM / 🔵 LONG-TERM priorities
  - Auto-invoked after successful `github_push`

#### Error Recovery
- **ToolResult Enhancement**: New `error_hint: Option<String>` field for structured error guidance
- **Consecutive Failure Tracking**: Agent tracks failures per tool
  - After 3 consecutive failures, escalates to user: "I'm having trouble with this step..."
  - Prevents silent infinite retry loops

#### Documentation
- **IDENTITY.md Rewritten**: Non-technical persona with plain language guidelines
  - Simple explanations instead of jargon ("Creating your project..." not "Initializing sandbox...")
  - Friendly, proactive, concise tone for Telegram interactions
  - Clear conversation flow: Understand → Plan → Confirm → Build → Deliver
- **AGENTS.md Updated**: Added sections 5.10 and 5.11
  - Auto-invoke product_advisor after deploy rule
  - Error classification and escalation guidelines

### Changed

- **TaskPlanTool Schema**: Extended action enum to include `propose`, `confirm`, `reject`
- **ToolResult Struct**: Added optional `error_hint` field (backward compatible, defaults to None)
- **Agent Loop**: 
  - Loads `active_project` from memory on startup
  - Injects project context into system prompt
  - Tracks consecutive tool failures with escalation
- **Memory Module**: Added project context helpers (`ActiveProject`, `ProjectStatus`)

### Removed

#### Documentation Cleanup
- Removed 19 outdated/duplicate documentation files:
  - Translated docs: `README.{fr,ja,ru,vi,zh-CN}.md`, `SUMMARY.{fr,ja,ru,zh-CN}.md`
  - Proposal docs: `sandboxing.md`, `resource-limits.md`, `audit-logging.md`, `agnostic-security.md`, `frictionless-security.md`, `security-roadmap.md`
  - Snapshot: `project-triage-snapshot-2026-02-18.md`
  - Meta: `docs-inventory.md`, `i18n-coverage.md`
  - Empty dirs: `i18n/`

### Security
- **Legacy XOR cipher migration**: The `enc:` prefix (XOR cipher) is now deprecated. 
  Secrets using this format will be automatically migrated to `enc2:` (ChaCha20-Poly1305 AEAD)
  when decrypted via `decrypt_and_migrate()`. A `tracing::warn!` is emitted when legacy
  values are encountered. The XOR cipher will be removed in a future release.

### Deprecated
- `enc:` prefix for encrypted secrets — Use `enc2:` (ChaCha20-Poly1305) instead.
  Legacy values are still decrypted for backward compatibility but should be migrated.

### Fixed
- All `ToolResult` constructions updated to include new `error_hint` field
- Updated default gateway port to `42617`.
- Removed all user-facing references to port `3000`.

## [0.1.0] - 2026-02-13

### Added
- **Core Architecture**: Trait-based pluggable system for Provider, Channel, Observer, RuntimeAdapter, Tool
- **Provider**: OpenRouter implementation (access Claude, GPT-4, Llama, Gemini via single API)
- **Channels**: CLI channel with interactive and single-message modes
- **Observability**: NoopObserver (zero overhead), LogObserver (tracing), MultiObserver (fan-out)
- **Security**: Workspace sandboxing, command allowlisting, path traversal blocking, autonomy levels (ReadOnly/Supervised/Full), rate limiting
- **Tools**: Shell (sandboxed), FileRead (path-checked), FileWrite (path-checked)
- **Memory (Brain)**: SQLite persistent backend (searchable, survives restarts), Markdown backend (plain files, human-readable)
- **Heartbeat Engine**: Periodic task execution from HEARTBEAT.md
- **Runtime**: Native adapter for Mac/Linux/Raspberry Pi
- **Config**: TOML-based configuration with sensible defaults
- **Onboarding**: Interactive CLI wizard with workspace scaffolding
- **CLI Commands**: agent, gateway, status, cron, channel, tools, onboard
- **CI/CD**: GitHub Actions with cross-platform builds (Linux, macOS Intel/ARM, Windows)
- **Tests**: 159 inline tests covering all modules and edge cases
- **Binary**: 3.1MB optimized release build (includes bundled SQLite)

### Security
- Path traversal attack prevention
- Command injection blocking
- Workspace escape prevention
- Forbidden system path protection (`/etc`, `/root`, `~/.ssh`)

[0.1.0]: https://github.com/theonlyhennygod/zerobuild/releases/tag/v0.1.0
