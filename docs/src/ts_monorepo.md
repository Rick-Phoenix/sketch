# Monorepo Generation

Once we have our settings (or not, if we want to use defaults), we can run

>`{{#include ../../sketch/tests/output/ts_examples/commands/monorepo_cmd}}`

to create our new Typescript monorepo.

For example, if we use this config:
```yaml
{{#include ../../examples/typescript/root_package.yaml:all}}
```


And the package.json file of the root package will be like this:

```json
{{#include ../../sketch/tests/output/ts_examples/package.json}}
```

You can also use the `generate_templates` field to automatically generate a certain file structure when the monorepo is generated. 

Let's say, for example, that every time that you create a new monorepo, you always want to create a `docker` directory with a basic `dev.dockerfile` inside of it, so that you can quickly create a dev environment using docker. 

For the root package, you would do so like this:

```yaml
{{#include ../../examples/typescript/root_package.yaml:template_example}}
```

So the final tree structure of the output directory will look like this:

```
{{#include ../../sketch/tests/output/ts_examples/tree_output.txt}}
```


