# Rendering Templates

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

>`{{#include ../../sketch/tests/output/custom_templates/commands/render_preset_cmd}}`

The output of the specified `root_dir` will look like this:

```
{{#include ../../sketch/tests/output/custom_templates/render_preset_tree.txt}}
```

>ℹ️ Templates can also be rendered to stdout with the --stdout flag.

## Defining a template in a command

We can also define a template inside the command itself.

Cmd:

>`{{#include ../../sketch/tests/output/custom_templates/commands/literal_template_cmd}}`

Output:

```txt
{{#include ../../sketch/tests/output/custom_templates/from_literal.txt}}
```





