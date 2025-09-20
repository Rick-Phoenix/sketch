# Lsp Integration

In order to ensure the best development experience, it is highly recommended to set up your IDE to load the json schema for Sketch's configuration file.

This will provide type-safety, autocompletion and additional documentation when writing config files.

Every time a new version is released, the json schema for the configuration file will be updated in the [github repo](https://github.com/Rick-Phoenix/sketch/tree/main/schemas). Each version will have its own distinct schema file, and the `latest.json` file will track the schema for the latest version.

You can then use this schema with the `yaml`, `toml` or `json` language servers to get autocompletion, type information and documentation for each element of the config file.

This can be done by configuring the specific LSP in your editor, or simply by using a special comment at the top of your file that links to the config's json schema.


## Examples

Yaml:
```yaml
# yaml-language-server: $schema=https://github.com/Rick-Phoenix/sketch/blob/main/schemas/latest.json
```

Json:

```json
{
  "$schema": "https://github.com/Rick-Phoenix/sketch/blob/main/schemas/latest.json"
}
```

Toml:

```toml
#:schema https://github.com/Rick-Phoenix/sketch/blob/main/schemas/latest.json
```
# External Configs

All sub-configs such as those belonging to `pnpm`, `oxlint`, as well as `package.json` or `tsconfig.json` files are all **fully typed**, which means that autocompletion and type safety for them is available when writing a configuration file and using the json schema as described above.
