# Monorepo Generation

Once we have our settings defined (or not, if we want to use defaults), we can run

>`{{#include ../../../sketch/tests/output/ts_examples/commands/monorepo_cmd}}`

to create our new Typescript monorepo.

# Example

Our starting config:

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
