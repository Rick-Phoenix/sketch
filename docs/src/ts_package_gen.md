# Generating A Package

You can use the command `sketch ts package` to generate a new Typescript package.

For setting up the root package in a monorepo, go look in the dedicated section.

This is an example of a package-related configuration:

```yaml
{{#include ../../examples/typescript/new_package.yaml:all}}
```

After setting things up, you can run

>`{{#include ../../sketch/tests/output/ts_examples/commands/package_gen_cmd}}`

To generate the new package.

## Template rendering

You can use the `generate_templates` field to specify a list of templates that should also be generated whenever a package preset is being generated.

Let's say for example that in a type of package, you always need a `schemas` directory, where you import some common schemas from a shared package and define new ones. 

You can use this feature to generate a file inside `src/schemas/index.ts` automatically like this:

```yaml
{{#include ../../examples/typescript/new_package.yaml:template_example}}
```




So the final tree structure of the output directory will look like this:

```
{{#include ../../sketch/tests/output/ts_examples/packages/frontend/tree_output.txt}}
```

> ℹ️ You can also use the `-i` flag to automatically install the dependencies with your selected package manager whenever a new package is created.
