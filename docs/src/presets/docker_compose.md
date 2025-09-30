# Docker Compose Presets

Sketch can be used to create extensible Docker Compose presets. 

There are presets for entire compose files as well as individual services.

## Example

Config:

```yaml
{{#include ../../../examples/presets.yaml:docker_compose}}
```

Command:

>`{{#include ../../../sketch/tests/output/generated_configs/commands/compose}}`

Output:

```yaml
{{#include ../../../sketch/tests/output/generated_configs/compose.yaml}}
```

