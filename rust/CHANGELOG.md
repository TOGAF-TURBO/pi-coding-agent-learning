# Changelog

All notable changes to the piso project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

### Added

### Changed

### Fixed

### Removed

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
