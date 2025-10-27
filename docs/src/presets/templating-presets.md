# Templating Presets

Rendering a single template is for cases when we need a simpler or more flexible setup. But things become more interesting with **templating presets**, where different kinds of templates can be aggregated and used in order to define easily reproducible project structures. 

Templating presets can be rendered with the `render-preset` command, or as part of [another preset](../presets/git.md#adding-templates), such as a git repo preset.

A templating preset contains an optional context (which overrides the global context), and one or many of these items:

### 1. Individual Templates

- Individual templates, which provide a manually controlled output path and their own local context

#### Example

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

### 2. Template Directory

- A path to a directory inside `templates_dir`, where all templates will be recursively rendered in the output directory, with the same file tree structure

>ℹ️ Any template files that end with the `.j2`, `.jinja` or `.jinja2` extensions will have them automatically removed. So `myfile.json.j2` will just become `myfile.json`.

#### Example

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

### 3. Remote Template

- A special kind of template which points to a git repository. Every file inside of it will be rendered in the output directory.

```yaml
{{#include ../../../examples/templating/templating.yaml:prop_name}}
{{#include ../../../examples/templating/templating.yaml:remote_preset}}
```

#### Example

We start from this basic [example](https://github.com/Rick-Phoenix/sketch-remote-preset-example)

Command:

>`{{#include ../../../sketch/tests/output/custom_templates/commands/remote}}`

Tree output:

```
{{#include ../../../sketch/tests/output/custom_templates/remote/tree_output.txt:2:}}
```

File output for `some_file`:

```
{{#include ../../../sketch/tests/output/custom_templates/remote/some_file}}
```

## Extending Templating Presets

Templating presets are extensible. When a preset is being extended, its templates will be added to the receiving preset, and the two context maps will be merged, with the new context overwriting the previous context in case of conflicting variables.

```yaml
{{#include ../../../examples/templating/templating.yaml:extended_preset}}
```
