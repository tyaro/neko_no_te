# adapter-template

This template shows how to implement a `ModelAdapter` for neko_no_te.

Quick start
1. Copy this directory to `crates/plugins/<your-adapter>`.
2. Update `Cargo.toml` metadata (name, authors, description).
3. Implement your model-specific serialization in `src/lib.rs` (see `TODO`).
4. Run `cargo test -p <your-adapter>` to verify.

Notes
- Use `model-adapter` and `model-provider` crates from the workspace as references.
- Keep adapter logic focused on formatting/serializing the request and parsing the response; network/transport is the provider's responsibility.
