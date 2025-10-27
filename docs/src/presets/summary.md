# Presets

Sketch supports various kinds of presets, which are designed to serve different functionalities. Some presets serve as aggregators for other presets, others provide [extensibility](#extending-presets), while others provide [type safety](../configuration/lsp.md) and lsp integration.

As of now, these presets are available:

- Templating
    - Templating presets (extensible)

- Docker
    - Docker Compose file (extensible)
    - Docker Compose service (extensible)

- Git
    - Git repo
    - `.gitignore` (extensible)
    - `.pre-commit-config.yaml` (extensible)
    - [Github workflow](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax) (extensible)
    - [Github workflow job](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobs) (extensible)
    - [Github workflow step](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idsteps)

- Rust
    - `Cargo.toml` (extensible, with merging of features for dependencies)

- Typescript
    - Typescript package
    - `pnpm-workspace.yaml` (extensible)
    - `package.json` (extensible, with [extra features](../ts/smart_features.md))
    - `tsconfig.json` (extensible, with merging of values for the `references`, `include`, `exclude` and `files` fields)
    - `.oxlintrc.json` (extensible)
    - `vitest` (not a full configuration for `vitest.config.ts`, but a basic testing setup)

These can be generated individually, or as part of another preset. You can find more information about each command in the [cli reference](../cli_docs.md).

## Extending Presets

Some presets can extend other presets. When a preset is extended, the merging strategy for their fields works like this:

- Collections are merged and, in almost all cases, also deduped and sorted (except for cases where order matters such as a list of command arguments). If the collection is a map and the values in it are also maps (as is the case for the `catalogs` field in `package.json` or `pnpm-workspace.yaml`), the inner maps will be merged.
- Values that are also extensible (such as `compilerOptions` in a `tsconfig` preset) will be merged with the same rules as above
- All other values are overwritten, except if the previous value was present and the new value is `null`. This is to avoid merging values that come from partially-defined presets, where the missing fields are all unset. Generally speaking, the correct strategy to extend presets is to define a base and then `add` elements to it, rather than replacing other values.

## Examples

This is a detailed example of the various kinds of presets that are available:

```yaml
{{#include ../../../examples/presets.yaml:all}}
```

