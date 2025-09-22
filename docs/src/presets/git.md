# Generating A Git Repo

The `sketch repo` command allows you to generate a new git repository, starting from a preset stored in one of your configuration files.

A git preset uses (or defines) a preset for its `gitignore` file and, optionally, for [`pre-commit`](https://pre-commit.com), as well as a list of templates that will be generated inside the root of the new repo when the command is triggered.

```yaml
{{#include ../../../examples/typescript/presets.yaml:templates}}

{{#include ../../../examples/typescript/presets.yaml:git}}
```
Starting from this config, we can run this command:

>`{{#include ../../../sketch/tests/output/presets/cmd}}`

To generate a new git repo. 

With cli flags, we can override the `gitignore` and `pre-commit` presets, as well as adding new templates to generate to the list.

Tree output:

```
{{#include ../../../sketch/tests/output/presets/tree_output.txt}}
```
