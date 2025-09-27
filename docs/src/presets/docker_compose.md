# Docker Compose Presets

Sketch can be used to create extensible Docker Compose presets and generate files from them.

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

