# Github Workflow Presets

Sketch supports presets for [Github workflows](https://docs.github.com/en/actions/reference/workflows-and-actions) and for jobs within a github workflow.

## Example

We use this configuration:

```yaml
{{#include ../../../examples/presets.yaml:workflow_presets}}
```

Run the command

>`{{#include ../../../sketch/tests/output/generated_configs/commands/workflow}}`

And get this output:

```yaml
{{#include ../../../sketch/tests/output/generated_configs/workflow.yaml}}
```
