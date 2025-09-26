# Presets

Templating presets are not the only way to create reusable project structures and configurations. Sketch also supports presets for some of the most common elements that you would find in a typical project.

Some of these presets are used just for defining an easily reproducible configuration, while others also include extra features such as extensibility.

As of now, these presets are available:

- Git
    - Git repo
    - `.gitignore` (extensible)
    - `.pre-commit-config.yaml` (extensible)

- Typecript
    - Typescript package
    - `pnpm-workspace.yaml` (extensible)
    - `package.json` (extensible, extra convenience feature for creating a mini-db for contributors)
    - `tsconfig.json` (extensible, with merging of values for the `references`, `include`, `exclude` and `files` fields)
    - `.oxlintrc.json` (extensible)
    - `vitest` (not a full configuration for `vitest.config.ts`, but a basic testing setup)

# Extending Presets

For those presets that are extensible, merging them works like this:

- Collections are merged (and in almost all cases, deduped and sorted)
- Conflicting values are replaced
- Values that are also extensible (such as `compilerOptions` in tsconfig) themselves will be merged with the same rules as above

# Examples

This is an in-depth example of the various kinds of presets that are available:

```yaml
{{#include ../../../examples/typescript/presets.yaml:all}}
```

