# Package.json Presets

You can use the config file to store some `package.json` presets that you can easily reuse among different projects or packages with similar characteristics, like scripts or dependencies.

Just like [tsconfigs](./tsconfig_presets.md) and [global configurations](./configuration.md), `package.json` presets can also extend one another.

## Example

```yaml
{{#include ../../examples/typescript/extending_package_json.yaml:all}}
```

After we run

`{{#include ../../sketch/tests/output/example_cmds/package_json_cmd}}`

We get this output in the package.json file: 

(typescript and oxlint are always added to the dev dependencies, along with vitest for packages that are not root packages)

```json
{{#include ../../sketch/tests/output/ts_examples/packages/svelte_frontend/package.json}}
```

