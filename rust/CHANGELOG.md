# Changelog

All notable changes to the piso project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

### Added

- **Parallel tool execution** (G1): ExecutionMode enum (Parallel/Sequential), futures::join_all for concurrent tool calls, ordered result collection
- **Structured compaction** (G2): Goal/Progress/Done/Blocked/NextSteps template matching TS version, UPDATE prompt for incremental merge with <previous-summary>
- **Runtime parameter validation** (G4): ToolDefinition::validate_input() checks required fields and JSON Schema type matching before execution
- **Process tree cleanup** (G3): setsid() for process groups, kill_process_tree() via libc::kill(-pgid) on Unix, taskkill /F /T on Windows
- **Terminate signal** (G5): ToolResult.terminate field, agent loop breaks ReAct when tool requests termination
- **System prompt date injection** (G7): {date} placeholder in default_role_prompt(), chrono::Utc::now()
- **Dual-loop steering + followUp** (G6): inner loop polls steering_rx (try_recv), outer loop blocks on follow_up_rx (recv.await), matching TS dual-while pattern
- **Image size limit** (G8): 1MB limit with graceful degradation, oversized images emit text placeholder
- **Model change JSONL event** (G9): ModelChangeEntry in pi-types/session.rs, AgentLoop::set_model() appends model_change to JSONL for replay fidelity
- **Runtime skill expansion** (G10): /skill:<name> slash command, resolve_skill() searches .piso/skills/ and ~/.piso/skills/
- **JSONL compaction write** (G11): CompactionEntry.archived_range field, JsonlSession::active_entries() skips archived entries, build_messages() uses active context
- **Extension transformContext hook** (G12): on_transform_context/on_convert_to_llm in ExtensionApi, fire_transform_context() called before LLM request
- 18 new tests across 6 crates

### Changed

- compaction.rs: log-only stub replaced with actual CompactionEntry write to JSONL
- loop_engine.rs: single-loop restructured to dual-loop (outer followUp + inner ReAct+steering)
- handle_slash_command: 8 params refactored into SlashContext struct

### Fixed

- Clippy clean: 0 warnings on release + check

## [0.1.0] — 2026-05-12

### Added

- **Phase 1**: Type system, JSONL sessions, Anthropic SSE, bash tool, Agent loop, print mode
- **Phase 2**: 6 tools (read/write/edit/find/grep/ls) + 3 LLM drivers (OpenAI/Gemini/Anthropic) + session management + context + skills + compaction
- **Phase 3**: TUI with ratatui + interactive mode + streaming + Markdown + session restore + session/model selector + configurable keybindings + theme
- **Phase 4**: Extensions system + RPC protocol + slash commands + @file references + autocomplete + 7 LLM providers (Anthropic/OpenAI/Gemini/Azure/Bedrock/Vertex/OpenAI Responses)
- 8 LLM providers: anthropic, openai (+12 compatible), google/gemini, azure-openai, amazon-bedrock, google-vertex, openai-responses, cloudflare-workers-ai
- Image support: @file detects png/jpg/gif/webp, base64 encodes in messages
- Interactive diff viewer: unified diff parsing, multi-file navigation, color rendering
- OAuth device flow for GitHub Copilot: piso login/logout commands
- Extension management: install/remove/update/list subcommands
- Extension intercept events: on_before_agent_start, on_message_update, on_tool_execution_update
- Extension UI API: select/confirm/input/set_widget/set_status trait methods
- RPC extension UI protocol: 5 request types + 4 response types
- Centralized message transform: to_anthropic_messages, to_openai_messages, to_gemini_contents
- JSONL export: /export detects .jsonl extension
- File mutation queue: tokio Mutex serializes write/edit tool calls
- Visual truncate: messages >300 lines show head/tail with truncation marker
- CLI: --model provider/id:thinking syntax, multi-value --skill/--theme/--prompt-template
- Slash commands: /new /reload /copy /fork /session /name /diff /login /logout /export
- 306 tests across 8 crates

### Changed

- Enter=Submit, Shift+Enter=NewLine (matches TS version)
- 4-region TUI layout (chat/status/editor/footer), no header
- Braille spinner animation (10 frames, 250ms tick)
- Agent running allows input (only Submit blocked)
- Thinking content rendered as separate dim+italic blocks
- 16 RGB colors matching TS dark.json theme
