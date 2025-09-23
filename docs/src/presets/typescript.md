# Typescript Presets

## Package.json Presets
```yaml
{{#include ../../../examples/typescript/presets.yaml:package_json}}
```

>ℹ️ `package.json` presets come with extra features. [Dedicated section](../ts/smart_features.md)

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


<details>
<summary>Tsconfig Output</summary>

```json
{{#include ../../../sketch/tests/output/presets/packages/presets_example/tsconfig.json}}
```
</details>

<details>
<summary>Package.json output</summary>

```json
{{#include ../../../sketch/tests/output/presets/packages/presets_example/package.json}}
```
</details>

<details>
<summary>Oxlintrc.json output</summary>

```json
{{#include ../../../sketch/tests/output/presets/packages/presets_example/.oxlintrc.json}}
```
</details>

# Adding Templates

We can also use the `with_templates` setting (or `--with-template <id=TEMPLATE_ID,output=PATH>` in the cli) to automatically generate one or many templates when the preset is used.

```yaml
{{#include ../../../examples/typescript/presets.yaml:templates}}

# We add this part to the package preset

{{#include ../../../examples/typescript/presets.yaml:ts_templates}}
```
