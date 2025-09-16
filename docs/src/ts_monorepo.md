# Monorepo Generation

Once we have our settings (or not, if we want to use defaults), we can run

`{{#include ../../sketch/tests/output/ts_examples/commands/monorepo_cmd}}`

to create our new Typescript monorepo.

For example, if we use this config:
```yaml
{{#include ../../examples/typescript/root_package.yaml:all}}
```


The tree structure of the output dir will look like this:
```
{{#include ../../sketch/tests/output/ts_examples/tree_output.txt}}
```

And the package.json file of the root package will be like this:

```json
{{#include ../../sketch/tests/output/ts_examples/package.json}}
```
