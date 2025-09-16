# Custom Templating

Sketch can be used to leverage the Tera templating engine to quickly set up all sorts of files or entire project structures.

This means being able to inject variables into generated content, as well as using all of the functions and filters available in Tera (like extracting envs), plus extra ones added by Sketch (like uuid generation). 

Templates can be defined in files or directly within a command. The variables can also be defined in the config file or as part of a command.

```toml
{{#include ../../sketch/tests/custom_templates/custom_templates.toml:config}}
```

Templates can also be defined in groups, which can be rendered all at once starting from the same root_dir.

This is how groups are defined:

```toml
{{#include ../../sketch/tests/custom_templates/custom_templates.toml:preset}}
```

The output paths defined in the command or in the preset will be joined to the `root_dir` setting.

When we run the command

`{{#include ../../sketch/tests/output/custom_templates/commands/render_preset_cmd}}`

The output of the specified `root-dir` will look like this:

```
{{#include ../../sketch/tests/output/custom_templates/render_preset_tree.txt}}
```

## Global and Local Context

As you can see from the previous example, the variables provided as a local context override global variables.

`with_global_vars.yaml` will be like this:

```yaml
{{#include ../../sketch/tests/output/custom_templates/with_global_var.yaml}}
```

While `with_override.yaml` will be like this:

```yaml
{{#include ../../sketch/tests/output/custom_templates/with_override.yaml}}
```

But the variables defined from the command always have the greatest priority. 

So if we run this command:

`{{#include ../../sketch/tests/output/custom_templates/commands/cli_override_cmd}}`

The output will be:

```yaml
{{#include ../../sketch/tests/output/custom_templates/with_cli_override.yaml}}
```

<div class="warning">
Variables defined with the --set flag must be formatted in valid json. This means that, for example, strings must be wrapped in escaped quotes.
</div>

## Defining a template in the command

We can also define a template inside the command itself.

Cmd:

`{{#include ../../sketch/tests/output/custom_templates/commands/literal_template_cmd}}`

Output:

```txt
{{#include ../../sketch/tests/output/custom_templates/from_literal.txt}}
```


## Rendering to stdout

Templates can also be rendered to stdout with the --stdout flag.
