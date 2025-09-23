# Monorepo Generation

We can set up a Typescript monorepo by simply running this command:

>`{{#include ../../../sketch/tests/output/ts_examples/commands/monorepo_cmd}}`

This will use all of the default settings and set up a basic structure for a monorepo.

But we can also create a package preset and use that for the root package of the monorepo.

# Example

Let's make this our starting config:

```yaml
{{#include ../../../examples/typescript/root_package.yaml:all}}
```

Tree output:

```
{{#include ../../../sketch/tests/output/ts_examples/tree_output.txt}}
```

>ℹ️ You can use the `-i` flag to install dependencies for the root package after creating the new monorepo.

## Adding Templates

You can also use the `with_templates` field to automatically generate a certain file structure when the monorepo is generated. 

Let's say, for example, that every time that you create a new monorepo, you always want to create a `docker` directory with a basic `dev.dockerfile` inside of it, so that you can quickly create a dev environment using docker. 

To do that, we add this to the root package's definition:

```yaml
{{#include ../../../examples/typescript/root_package.yaml:template_example}}
```

>ℹ️ You can also use `--with-template <id=TEMPLATE_ID,output=PATH>` as a flag to add more templates when generating a new package.
