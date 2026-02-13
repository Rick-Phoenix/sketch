# Typescript Presets

One of the goals of Sketch is to bring more structure and order when dealing with the somewhat chaotic typescript ecosystem.

Many typical components of a typescript project have their own dedicated preset, and there are also dedicated commands for common operations such as generating barrel files. 

## Package.json Presets
```yaml
{{#include ../../../examples/presets.yaml:package_json}}
```

Just like in actual `package.json` files, custom fields are allowed.

>ℹ️ `package.json` presets come with extra features. [Dedicated section](../ts/smart_features.md)

## Tsconfig Presets
```yaml
{{#include ../../../examples/presets.yaml:tsconfig}}
```

<div class="warning">

Unlike what happens when you merge two `tsconfig` files by using the `extends` field, extending presets will merge all collections, including `files`, `include`, `exclude` and `references`, which would normally be overwritten, rather than merged.
</div>

## Oxlint Presets
```yaml
{{#include ../../../examples/presets.yaml:oxlint}}
```

## Pnpm-workspace Presets
```yaml
{{#include ../../../examples/typescript/root_package.yaml:pnpm}}
```

## Vitest Presets
```yaml
{{#include ../../../examples/presets.yaml:vitest}}
```

## Package Presets

A package preset can be used to collect other presets, such as in this example:

```yaml
{{#include ../../../examples/presets.yaml:package}}
```
