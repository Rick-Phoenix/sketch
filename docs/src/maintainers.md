# Maintainers Information

When working with a team on a monorepo that is made of several individual packages, it's very common that the same people have to be added to the `author`, `contributors` and `maintainers` fields in the `package.json` files for these packages. 

To make this job easier and faster, you can use the `people` field in the typescript config to store information about the contributors that are referred in multiple places, and then you can simply refer them by their ID.

## Example

```yaml
{{#include ../../examples/typescript/people.yaml:all}}
```

After we run

>`{{#include ../../sketch/tests/output/ts_examples/commands/people_cmd}}`

We get this `package.json` file in the root of the new package:

```json
{{#include ../../sketch/tests/output/ts_examples/packages/people-example/package.json}}
```
