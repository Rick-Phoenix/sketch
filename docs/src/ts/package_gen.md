# Generating A Package

You can use the command `sketch ts package` to generate a new Typescript package.

## Adding Templates

You can also use the `with_templates` field (or the `--with-template` cli flag) to specify a list of templates or templating presets that should be generated whenever a package preset is being used.

Let's say for example that in a type of package, you always need a `schemas` directory, where you import some common schemas from a shared package and define new ones. 

You can use this feature to generate a file inside `src/schemas/index.ts` automatically like this:

```yaml
{{#include ../../../examples/typescript/new_package.yaml:template_example}}
```

## Example

We start from this configuration:

```yaml
{{#include ../../../examples/typescript/new_package.yaml:all}}
```

And then run

>`{{#include ../../../sketch/tests/output/ts_example/commands/package_gen_cmd}}`

To obtain this output in the designated directory:

```
{{#include ../../../sketch/tests/output/ts_example/packages/frontend/tree_output.txt:2:}}
```

> ℹ️ You can also use the `-i` flag to automatically install the dependencies with your selected package manager whenever a new package is created.

With the following `package.json` file

```json
{{#include ../../../sketch/tests/output/ts_example/packages/frontend/package.json}}
```
