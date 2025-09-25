# Rendering A Preset

Rendering a single template is for cases when we need a simpler or more flexible setup. But things become more interesting with templating presets, which can allow you to define entire project structures and then replicate them very easily (while also being able to modify the context at different levels). 

Presets can be rendered with the `render-preset` command, or generated automatically with another preset, such as a git repo preset.

# Types Of Presets

### Collection Presets

There are two kinds of templating presets. One is what I call the `collection` preset, which is for cases when you need more granular control about the templates that you want to select, their output paths and context.

```yaml
{{#include ../../../examples/templating/templating.yaml:prop_name}}
{{#include ../../../examples/templating/templating.yaml:collection_preset}}
```

Command:

>`{{#include ../../../sketch/tests/output/custom_templates/commands/collection_preset}}`

Tree output:

```
{{#include ../../../sketch/tests/output/custom_templates/lotr/tree_output.txt:2:}}
```
### Structured Preset

The other type are what I call `structured` presets, where you do not define an output path for every single file, but rather, you select an entire directory within your `templates_dir`, and then all of the templates inside that dir (and its descendants, recursively) will be rendered with the same exact structure in the output directory. 

>ℹ️ If the template files end with a `.j2`, `.jinja` or `.jinja2` extension, it gets stripped automatically.

Let's say that this is our `templates_dir`:

```
{{#include ../../../sketch/tests/output/custom_templates/tree:2:}}
```

And we define this preset, which is meant to reproduce the entire file structure of `subdir` in the target directory.

```yaml
{{#include ../../../examples/templating/templating.yaml:prop_name}}
{{#include ../../../examples/templating/templating.yaml:structured_preset}}
```

Command:

>`{{#include ../../../sketch/tests/output/custom_templates/commands/structured_preset}}`

Output tree:

```
{{#include ../../../sketch/tests/output/custom_templates/structured/tree_output.txt:2:}}
```
