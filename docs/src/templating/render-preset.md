# Rendering A Preset

Rendering a single template is for cases when we need a simpler or more flexible setup. But things become more interesting with templating presets, which can allow you to define entire project structures and then replicate them very easily (while also being able to modify the context at different levels). 

Presets can be rendered with the `render-preset` command, or generated automatically with [another preset](../presets/git.md#adding-templates), such as a git repo preset.

# Types Of Presets

There are two kinds of templating presets, which are best used under different kinds of situations.

### 1. Collection Preset

**Features**:

- Aggregates individual templates
- Can define individual output path
- Can provide group and individual context

**Use case**:

For cases when you need more granular control about the templates that you want to select, their output paths, or their local context.

**How it works**:

You select an individual template, and provide an output path and an optional local context. You can provide a group context which can be overridden by a template's individual context.

Relative paths will resolve from the root of the target directory (which can be defined manually via command for custom templates, or is inferred automatically when the template is being rendered as part of another preset)

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
### 2. Structured Preset

**Features**:

- Recursively collects all templates in a directory
- Replicates the file tree structure in the output path

**Use case**:

When you want to have a 1:1 replica of a specific file tree structure.

The other type are what I call `structured` presets, where you do not define an output path for every single file, but rather, you select an entire directory within your `templates_dir`, and then all of the templates inside that dir (and its descendants, recursively) will be rendered with the same exact structure in the output directory. 

#### Example

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
