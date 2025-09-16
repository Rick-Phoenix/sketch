# Maintainers Information

# Reusable Maintainers Information

If you are working in a group, and you frequently find yourself adding the same information to the `author`, `contributors` and `maintainers` field in `package.json` files, you might be interested in using the `people` field of the config file, where you can store information about maintainers that can be reused, by simply copying their id, in the generated `package.json` files.

## Example

```yaml
{{#include ../../examples/typescript/people.yaml:all}}
```

After we run

`{{#include ../../sketch/tests/output/ts_examples/commands/people_cmd}}`

We get this `package.json` file in the root of the new package:

```json
{{#include ../../sketch/tests/output/ts_examples/packages/people-example/package.json}}
```
