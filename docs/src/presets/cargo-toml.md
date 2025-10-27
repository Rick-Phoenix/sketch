# Cargo.toml Presets

You can use sketch to define and generate extensible `Cargo.toml` presets.

When presets are extended, each dependency will also get merged according to the [merging rules](./summary.md#extending-presets).

## Example

Config:

```yaml
{{#include ../../../examples/presets.yaml:cargo}}
```

Output:

```yaml
{{#include ../../../sketch/tests/output/generated_configs/Cargo.toml}}
```

