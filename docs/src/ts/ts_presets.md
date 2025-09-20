# Presets

`sketch` allows you to define some extensible presets in your configuration files for some of the most common configuration files, such as `package.json`, `tsconfig.json` or `.oxlintrc.json`.

Presets can be created for entire packages, but they are not extensible. Their purpose is mostly to create identifiers for setups that can be generated via the cli.

```yaml
{{#include ../../../examples/typescript/extending_presets.yaml:all}}
```
