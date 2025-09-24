# Smart Features

Sketch adds a few smart features that may be useful when dealing with packages dependencies and metadata.

## Converting `latest` to a range

By default, all versions marked as `latest` will be converted to a version range (`minor` by default, but can be customized) that starts from the actual latest version for that package.

This means that you can easily reuse your presets over time to start new projects, without needing to manually bump all versions, and while also avoiding `latest`, which is not suitable for stability.

<div class="warning">

Sketch uses the npm api to fetch the latest version for any given package. Depending on how many requests are made in a given timeframe, you might be rate limited by the api, causing an error.
</div>


```yaml
{{#include ../../../examples/typescript/packages_versions.yaml:all}}
```


## Adding dependencies to the catalog

If you set up your package manager to be `pnpm` (which is the default) and set `catalog` to `true`, whenever you try to generate a new package that has dependencies marked with `catalog:` (either the default catalog or a named one), each package that is not present in the target catalog inside `pnpm-workspace.yaml` will be added automatically to it.

### Example

Let's say that we are starting with a basic `pnpm-workspace.yaml` config like this one:

```yaml
{{#include ../../../examples/typescript/root_package.yaml:pnpm}}
```

and we generate a package that has `catalog` dependencies, which are currently absent from their target catalogs:

```yaml
{{#include ../../../examples/typescript/catalog.yaml:all}}
```

After running 

>`{{#include ../../../sketch/tests/output/ts_examples/commands/catalog_cmd}}`

We can see that the pnpm-workspace.yaml file has been updated with the new named catalog and dependencies:

```yaml
{{#include ../../../sketch/tests/output/ts_examples/pnpm-workspace.yaml}}
```

## Storing And Reusing Contributors' Data

When working with a team on a monorepo that is made of several individual packages, it's very common that the same people have to be added to the `author`, `contributors` and `maintainers` fields in the `package.json` files for these packages. 

To make this job a bit easier, you can use the `people` field in the typescript config to store information about contributors which you can then simply refer to by their ID.

### Example

```yaml
{{#include ../../../examples/typescript/people.yaml:all}}
```

After we run

>`{{#include ../../../sketch/tests/output/ts_examples/commands/people_cmd}}`

We get this `package.json` file in the root of the new package:

```json
{{#include ../../../sketch/tests/output/ts_examples/packages/people-example/package.json}}
```
