# model-adapter

Adapter abstraction and a default adapter for `llama3.1:8b`.

Purpose
- Provide a small plugin-style API for model-specific invocation formats
  (function-calling / tools / prompt formatting) while keeping the core
  application provider-agnostic.

Built-in default
- `Llama3DefaultAdapter` — supports `llama3.1:8b`. It serializes provided
  `ToolSpec` entries into a JSON snippet appended to the prompt, then
  delegates to the configured `ModelProvider`.

Extending
- Create a new crate under `crates/plugins/` or `plugins/` implementing
  `ModelAdapter` to support additional models (e.g. `qwen3`, `phi4-mini`).
