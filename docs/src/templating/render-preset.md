# Rendering A Preset

Rendering a single template is for cases when we need a simpler or more flexible setup. But things become more interesting with templating presets, which can allow you to define entire project structures and then replicate them very easily (while also being able to modify the context at different levels). 

Presets can be rendered with the `render-preset` command, or generated automatically with [another preset](../presets/git.md#adding-templates), such as a git repo preset.

# Creating A Preset

A preset contains an optional context (which overrides the global context), and a list that contains any of these elements:

## Individual Presets

- An individual template, with manually controlled output path and context

### Example

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

## Structured Template

- A path to a directory inside `templates_dir`, to extract all the files inside of it recursively and render them in the output directory, with the same file tree structure

### Example

>ℹ️ Any template files that end with the `.j2`, `.jinja` or `.jinja2` extensions will have them automatically removed. So `myfile.json.j2` will just become `myfile.json`.

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

Tree output:

```
{{#include ../../../sketch/tests/output/custom_templates/structured/tree_output.txt:2:}}
```

# Remote Template

- A special kind of structured preset which points to a git repository. Every file inside of it will be rendered in the output directory, just like structured templates.

```yaml
{{#include ../../../examples/templating/templating.yaml:prop_name}}
{{#include ../../../examples/templating/templating.yaml:remote_preset}}
```

## Example

We start from this basic [example](https://github.com/Rick-Phoenix/sketch-remote-preset-example)

Command:

>`{{#include ../../../sketch/tests/output/custom_templates/commands/remote}}`

Tree output:

```
{{#include ../../../sketch/tests/output/custom_templates/remote/tree_output.txt:2:}}
```

File output:

```
{{#include ../../../sketch/tests/output/custom_templates/remote/some_file}}
```

# Extending Templating Presets

Templating presets are extensible. When a preset is being extended, its templates will be added to the receiving preset, and the two context maps will be merged, with the new context overwriting the previous context in case of conflicting variables.

```yaml
{{#include ../../../examples/templating/templating.yaml:extended_preset}}
```


