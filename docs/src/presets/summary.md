# Presets

Templating presets are not the only way to create reusable project structures and configurations. Sketch also supports presets for some of the most common elements that you would find in a typical project.

Some of these presets are used just for defining an easily reproducible configuration, while others also include extra features such as extensibility.

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
    - Github workflow (extensible)
    - Github workflow job (extensible)

- Rust
    - `Cargo.toml` (extensible)

- Typescript
    - Typescript package
    - `pnpm-workspace.yaml` (extensible)
    - `package.json` (extensible, with [extra features](../ts/smart_features.md))
    - `tsconfig.json` (extensible, with merging of values for the `references`, `include`, `exclude` and `files` fields)
    - `.oxlintrc.json` (extensible)
    - `vitest` (not a full configuration for `vitest.config.ts`, but a basic testing setup)

These can be generated either as part of another preset, or with a command (or both, in some cases) you can find more information in the rest of the documentation and in the [cli reference](../cli_docs.md).

# Extending Presets

Some presets can be extended by inheriting the value from one or multiple presets.

When a preset is extended, the merging strategy for their fields works like this:

- Collections are merged and in almost all cases, deduped and sorted (except for cases where order matters such as a list of command arguments)
- Values that are also extensible (such as `compilerOptions` in tsconfig) themselves will be merged with the same rules as above
- All other values are overwritten, except if the previous value was present and the new value is `null`. This is to avoid merging values that come from partially-defined presets, where the missing fields are all unset. Generally speaking, the correct strategy to extend presets is to define a base and then `add` elements to it, rather than replacing them.

# Examples

This is an in-depth example of the various kinds of presets that are available:

```yaml
{{#include ../../../examples/presets.yaml:all}}
```

