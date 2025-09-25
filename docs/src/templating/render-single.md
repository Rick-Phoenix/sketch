# Rendering A Template

We can render a template to a file or to stdout with the command `sketch render`.

## From A Configuration File

A template can be defined as simple text in a config file, inside the `templates` map. In this case, you can refer to this template by using its map key. This is the easiest method, but it also means slightly worse IDE integration for things like snippets and syntax highlighting.

```yaml
{{#include ../../../examples/templating/templating.yaml:template_definition}}
```

Command:

>`{{#include ../../../sketch/tests/output/custom_templates/commands/from_id}}`

## From `templates_dir`

The best method is to use a file inside `templates_dir`. With this method, the template's ID becomes the relative path from `templates_dir`. This method provides better IDE integration if you have set up snippets or syntax highlighting for `jinja` files (Tera, the templating engine used by `sketch` is based on jinja).

### Example

Tree structure of `templates_dir`:

```
{{#include ../../../sketch/tests/output/custom_templates/tree:2:}}
```

Command:

>`{{#include ../../../sketch/tests/output/custom_templates/commands/from_template_file}}`

## Using A File

You can also use the `-f` flag to render a template from any file, even outside `templates_dir`.

## From Literal Definition

Or alternatively, a template can also be defined directly within a command:

>`{{#include ../../../sketch/tests/output/custom_templates/commands/literal_template_cmd}}`


