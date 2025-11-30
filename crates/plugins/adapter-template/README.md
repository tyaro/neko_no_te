# adapter-template

This template shows how to implement a `ModelAdapter` for neko_no_te.

Quick start
1. Copy this directory to `crates/plugins/<your-adapter>`.
2. Update `Cargo.toml` metadata (name, authors, description).
3. Implement your model-specific serialization in `src/lib.rs` (see `TODO`).
4. Run `cargo test -p <your-adapter>` to verify.

Plugin metadata
- Place a `plugin.toml` file at the crate root with the following required fields:
	- `name`: the plugin / crate name
	- `description`: short one-line description shown in the UI
	- `version`: semver string (e.g. "0.1.0")
	- `author`: name and contact (e.g. "Alice <alice@example.com>")

Publishing to crates.io
- Before publishing, update `Cargo.toml` with a unique `name`, `description`, proper `authors`, and `license`.
- Ensure tests pass: `cargo test -p <your-adapter>`.
- To publish:

```powershell
# login once (if not already):
cargo login <your-api-token>
# then publish from workspace root or crate folder:
cd crates/plugins/<your-adapter>
cargo publish --allow-dirty
```

Note: `--allow-dirty` can be useful during development, but remove it when publishing a final release. Also ensure `package.metadata` and `README.md` are correct for crates.io display.

Notes
- Use `model-adapter` and `model-provider` crates from the workspace as references.
- Keep adapter logic focused on formatting/serializing the request and parsing the response; network/transport is the provider's responsibility.
