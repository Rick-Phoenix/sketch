# Presets

Sketch allows you to define presets for some of the most common configuration files present in a Typescript project.

Some of these presets are used just for defining specific structures and then referring to them by their ID, while others also include extra features such as extensibility.

As of now, these presets are available, with the specified characteristics:

- Package preset
- `package.json` preset (extensible, extra feature for storing contributors' info)
- `tsconfig.json` preset (extensible, with merging of values for `references`, `include`, `exclude` and `files`)
- `.oxlintrc.json` preset (extensible)

```yaml
{{#include ../../../examples/typescript/presets.yaml:all}}
```
It is also possible to define the configuration for the `pnpm-workspace.yaml` file.
