# Handling Dependencies

## Converting `latest` to a range

By default, all `latest` versions will be converted to a version range (which can be specified in the config) that starts from the actual latest version for that package.

This means that you can easily use your presets and ensure that they're always going to be updated, without using `latest`, which is not very good practice.

>⚠️ Sketch uses the npm api to fetch the latest version for any given package. Depending on how many requests are made in a given timeframe, you might be rate limited by the api.


```yaml
{{#include ../../examples/typescript/packages_versions.yaml:all}}
```


## Adding dependencies to the catalog

If you set up your package manager to be `pnpm` (which is the default) and set `catalog` to `true`, whenever you try to generate a new package that has dependencies marked with `catalog:` (either the default catalog or a named one), each package that is not present in the target catalog inside `pnpm-workspace.yaml` will be added automatically to it.

### Example

Let's say that we are starting with a basic `pnpm-workspace.yaml` file, and we generate a package that has `catalog` dependencies, which are currently absent from their target catalogs:

```yaml
{{#include ../../examples/typescript/catalog.yaml:all}}
```

After running 

>`{{#include ../../sketch/tests/output/ts_examples/commands/catalog_cmd}}`

We can see that the pnpm-workspace.yaml file has been updated with the new named catalog and dependencies:

```yaml
{{#include ../../sketch/tests/output/ts_examples/pnpm-workspace.yaml}}

