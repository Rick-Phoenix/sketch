# Presets

Sketch allows you to define presets for some of the most common configuration files present in a Typescript project.

Some of these presets are used just for defining a group of files or settings, while others also include extra features such as extensibility.

As of now, these presets are available, with the specified characteristics:

- Git repos
    - `pre-commit` (extensible)

- Typecript
    - Package
    - `package.json` (extensible, extra feature for storing contributors' info)
    - `tsconfig.json` (extensible, with merging of values for `references`, `include`, `exclude` and `files`)
    - `.oxlintrc.json` (extensible)

```yaml
{{#include ../../examples/typescript/presets.yaml:all}}
```
