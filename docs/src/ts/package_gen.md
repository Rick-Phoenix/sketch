# Generating A Package

You can use the command `sketch ts package` to generate a new Typescript package.

This is an example of a package-related configuration:

```yaml
{{#include ../../../examples/typescript/new_package.yaml:all}}
```

After setting things up, you can run

>`{{#include ../../../sketch/tests/output/ts_examples/commands/package_gen_cmd}}`

To generate the new package.

## Adding Templates

You can also use the `with_templates` field (or the `--with-template` cli flag) to specify a list of templates that should be generated whenever a package preset is being used.

Let's say for example that in a type of package, you always need a `schemas` directory, where you import some common schemas from a shared package and define new ones. 

You can use this feature to generate a file inside `src/schemas/index.ts` automatically like this:

```yaml
{{#include ../../../examples/typescript/new_package.yaml:template_example}}
```

So the final tree structure of the output directory will look like this:

```
{{#include ../../../sketch/tests/output/ts_examples/packages/frontend/tree_output.txt:2:}}
```

> ℹ️ You can also use the `-i` flag to automatically install the dependencies with your selected package manager whenever a new package is created.
