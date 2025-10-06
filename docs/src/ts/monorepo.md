# Monorepo Generation

To set up a basic monorepo with default settings, we can just run the command `sketch ts monorepo`. But we can also create a package preset and use that for the root package of the monorepo.

This is a more customized configuration for the root package:

```yaml
{{#include ../../../examples/typescript/root_package.yaml:all}}
```

## Adding Templates

You can also use the `with_templates` field to automatically generate a certain file structure when the monorepo is generated. 

Let's say, for example, that every time that you create a new monorepo, you always want to create a `docker` directory with a basic `dev.dockerfile` inside of it, so that you can quickly create a dev environment using docker. 

To do that, we add this to the root package's definition:

```yaml
{{#include ../../../examples/typescript/root_package.yaml:template_example}}
```

>ℹ️ You can also use `--with-template <PRESET_ID|id=TEMPLATE_ID,output=PATH>` as a flag to add more templates or templating presets when generating a new package.

# Hooks

We can define some commands (which can also be templates) to execute before and/or after generating the new monorepo:

```yaml
{{#include ../../../examples/presets.yaml:hooks}}
```

# Example

So after setting everything up, we run the command

>`{{#include ../../../sketch/tests/output/ts_examples/commands/monorepo_cmd}}`

And get this tree output:

```
{{#include ../../../sketch/tests/output/ts_examples/tree_output.txt:2:}}
```

>ℹ️ You can use the `-i` flag to install dependencies for the root package after creating the new monorepo.


