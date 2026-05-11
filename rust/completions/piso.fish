complete -c piso -l provider -d 'LLM provider (anthropic, openai, google, etc.)' -r
complete -c piso -s m -l model -d 'Model to use' -r
complete -c piso -l api-key -d 'API key for the provider' -r
complete -c piso -l base-url -d 'Base URL for the provider API' -r
complete -c piso -l thinking -d 'Thinking level (off, minimal, low, medium, high, xhigh)' -r
complete -c piso -l system-prompt -d 'System prompt override' -r
complete -c piso -l append-system-prompt -d 'Append to system prompt (repeatable)' -r
complete -c piso -l fork -d 'Fork a session at a specific point' -r
complete -c piso -l session -d 'Session ID to use' -r
complete -c piso -l mode -d 'Run mode: interactive, text, json, rpc' -r
complete -c piso -l list-models -d 'List available models' -r
complete -c piso -l export -d 'Export session to HTML' -r
complete -c piso -s e -l extension -d 'Load extensions from paths' -r
complete -c piso -s c -l continue -d 'Continue the most recent session'
complete -c piso -s r -l resume -d 'Resume a previous session (interactive picker)'
complete -c piso -l no-session -d 'Don\'t persist session'
complete -c piso -s n -l no-tools -d 'Disable all tools'
complete -c piso -l no-builtin-tools -d 'Disable built-in tools only'
complete -c piso -l no-extensions -d 'Disable extensions'
complete -c piso -l no-skills -d 'Disable skills'
complete -c piso -l no-prompt-templates -d 'Disable prompt templates'
complete -c piso -l no-context-files -d 'Disable context files (AGENTS.md etc)'
complete -c piso -l list-sessions -d 'List previous sessions'
complete -c piso -l offline -d 'Run offline (skip version check)'
complete -c piso -s v -l verbose -d 'Verbose logging'
complete -c piso -s p -l print -d 'Print mode: non-interactive, output to stdout'
complete -c piso -s h -l help -d 'Print help'
complete -c piso -s V -l version -d 'Print version'
