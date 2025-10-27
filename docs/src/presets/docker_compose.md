# Docker Compose Presets

Sketch can be used to create extensible presets for entire compose files as well as individual services.

## Example

Config:

```yaml
{{#include ../../../examples/presets.yaml:docker_compose}}
```

Command:

>`{{#include ../../../sketch/tests/output/generated_configs/commands/compose}}`

>ℹ️ With the `--service` flag, extra service presets can be added to the output file.

Output:

```yaml
{{#include ../../../sketch/tests/output/generated_configs/compose.yaml}}
```

