# Monorepo Generation

These are the top level configuration settings for Typescript projects:

```yaml
{{#include ../../examples/typescript/typescript_settings.yaml}}
```

## Root Package Settings

Setting up the root package is pretty straightforward:

```yaml
{{#include ../../examples/typescript/root_package.yaml:all}}
```

So with this setup, once we run

`{{#include ../../sketch/tests/output/example_cmds/monorepo_cmd}}`

The tree structure of the output dir will look like this:
```
{{#include ../../sketch/tests/output/ts_examples/tree_output.txt}}
```

Output of the package.json file:

```json
{{#include ../../sketch/tests/output/ts_examples/package.json}}
```
