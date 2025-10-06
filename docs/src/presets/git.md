# Generating A Git Repo

The `sketch repo` command allows you to generate a new git repository, starting from a preset stored in one of your configuration files.

A git preset uses (or defines) a preset for its `gitignore` file and, optionally, for [`pre-commit`](https://pre-commit.com), as well as a list of templates that will be generated inside the root of the new repo when the command is triggered.

```yaml
{{#include ../../../examples/presets.yaml:git}}
```
# Adding Templates

We can use the `with_templates` setting to add a group of templates to a git preset. Let's say that we want to automatically generate a basic docker setup whenever we use this preset:

```yaml
# We define a template in a file or in a config file...
{{#include ../../../examples/presets.yaml:templates}}

# ...and then we add it to a preset

{{#include ../../../examples/presets.yaml:git_preset}}
```

# Hooks

We can define some commands (which can also be templates) to execute before and/or after generating the new repo:

```yaml
{{#include ../../../examples/presets.yaml:hooks}}
```

# Putting It All Together

Starting from this config, we can run this command:

>`{{#include ../../../sketch/tests/output/presets/cmd}}`

To get this tree output:

```
{{#include ../../../sketch/tests/output/presets/tree_output.txt:2:}}
```

>ℹ️ With cli flags, we can override the `gitignore` and `pre-commit` presets, as well as adding new templates or hooks to run or generate when the preset is being used.

<details>
<summary>pre-commit-config.yaml output</summary>

```yaml
{{#include ../../../sketch/tests/output/presets/.pre-commit-config.yaml}}
```
</details>


<details>
<summary>gitignore output</summary>

```
{{#include ../../../sketch/tests/output/presets/.gitignore}}
```
</details>

