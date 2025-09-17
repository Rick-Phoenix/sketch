# Rendering Templates

Templates can be defined in files or directly within a command. The variables can also be defined in the config file or as part of a command.

```toml
{{#include ../../sketch/tests/custom_templates/custom_templates.toml:config}}
```

We can then render them singularly, or in groups.

>ℹ️ Single templates can be rendered to stdout with the --stdout flag.
# Rendering A Single Template


## Using A File

If `templates_dir` is set, all the files in `templates_dir` can be referenced with their ID, which is the relative path to them from `templates_dir`.

So if we have this directory structure in our `templates_dir`:

```
{{#include ../../sketch/tests/templates/templates_tree.txt}}
```

We can render the contents of `nested.j2` by running

>`{{#include ../../sketch/tests/output/custom_templates/commands/from_template_file_cmd}}`

However, we can also render content from any file, by using the `-f` flag. In this case, the relative paths will be resolved from the cwd.

Example:

>`{{#include ../../sketch/tests/output/custom_templates/commands/from_single_file_cmd}}`

## From A Config Definition

Templates can be defined inside the configuration file:

```yaml
{{#include ../../sketch/tests/custom_templates/custom_templates.toml:lit_template}}
```

>`{{#include ../../sketch/tests/output/custom_templates/commands/from_config_template_cmd}}`

## From Literal Definition

...or directly as part of the command:

>`{{#include ../../sketch/tests/output/custom_templates/commands/literal_template_cmd}}`



# Rendering Presets

Templates can also be defined in groups, which can be rendered all at once starting from the same `root_dir`.

This is how groups are defined:

```toml
{{#include ../../sketch/tests/custom_templates/custom_templates.toml:preset}}
```

When we run the command

>`{{#include ../../sketch/tests/output/custom_templates/commands/render_preset_cmd}}`

These templates will be rendered together, so that the output of the specified `root_dir` will look like this:

```
{{#include ../../sketch/tests/output/custom_templates/render_preset_tree.txt}}
```


