# Generating A Barrel File

Sometimes it is useful to keep one or many barrel files in a typescript project, to make it easier to import/export elements from a group of modules. 

Sketch has a dedicated command to make this process easier.

## Examples

Target directory:

```
{{#include ../../../sketch/tests/output/ts_barrel/tree:2:}}
```

Command:

>`{{#include ../../../sketch/tests/output/ts_barrel/commands/barrel}}`

Output:

```typescript
{{#include ../../../sketch/tests/output/ts_barrel/index.ts}}
```
