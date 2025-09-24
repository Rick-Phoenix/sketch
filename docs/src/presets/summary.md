# Presets

Sketch allows you to define presets for some of the most common elements used as part of a project.

Some of these presets are used just for defining a group of files or settings, while others also include extra features such as extensibility.

As of now, these presets are available, with the specified characteristics:

- Templating presets (to render a group of files and directories together)

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

# Extending Presets

For those presets that are extensible, merging them works like this:

- Collections are merged (and in almost all cases, deduped and ordered too)
- Conflicting values are replaced
- Values that are also extensible (such as `compilerOptions` in tsconfig) themselves will be merged with the same rules as above

# Examples

```yaml
{{#include ../../../examples/typescript/presets.yaml:all}}
```

