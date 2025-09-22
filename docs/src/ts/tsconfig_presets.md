# Tsconfig.json Presets

We can define some `tsconfig` presets, which can be extended.

<div class="warning">

Unlike what happens when you merge two `tsconfig` files by using the `extends` field, extending presets will merge all collections, including `files`, `include`, `exclude` and `references`, which would normally be overwritten, rather than merged.
</div>

```yaml
{{#include ../../../examples/typescript/presets.yaml:tsconfig}}

```

