# Typescript Presets

## Package.json Presets
```yaml
{{#include ../../../examples/typescript/presets.yaml:package_json}}
```

## Tsconfig Presets
```yaml
{{#include ../../../examples/typescript/presets.yaml:tsconfig}}
```

<div class="warning">

Unlike what happens when you merge two `tsconfig` files by using the `extends` field, extending presets will merge all collections, including `files`, `include`, `exclude` and `references`, which would normally be overwritten, rather than merged.
</div>

## Oxlint Presets
```yaml
{{#include ../../../examples/typescript/presets.yaml:oxlint}}
```

## Package Presets

This is what a fully formed package preset looks like. We are going to use the presets defined above in here.
```yaml
{{#include ../../../examples/typescript/presets.yaml:package}}
```


