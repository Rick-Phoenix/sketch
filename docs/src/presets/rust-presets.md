# Rust Presets

Sketch can be used to define and generate presets for rust crates (with the `sketch rust crate` command) and for individual `Cargo.toml` manifests (with the `sketch rust manifest` command).

The `sketch rust crate` command mimics the `cargo new` command in the sense that it detects if these is a workspace manifest in the parent directory of the target output, and if one is detected, the new crate will be added to its members. Also, if certain fields such as lints, keywords, license and so on are set in the workspace manifest but not in the manifest that is generated, they will be added as entries with `workspace = true`.

`Cargo.toml` presets are extensible. Dependencies are deeply merged, so for example the features will be joined.

## Example

Config:

```yaml
{{#include ../../../examples/presets.yaml:cargo}}
```

Command:

>`{{#include ../../../sketch/tests/output/rust_example/rust-example-cmd}}`

Output:

```toml
{{#include ../../../sketch/tests/output/rust_example/cargo-example.toml}}
```

