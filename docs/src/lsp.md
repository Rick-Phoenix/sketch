# Lsp Integration

In order to have the best development experience, it is crucial that you use the json schema for the configuration when writing one.

Every time a new version is released, the json schema for the configuration file will be updated in the [github repo](https://github.com/Rick-Phoenix/sketch/tree/main/schemas). 

Each version will have its own distinct schema file, and the `latest.json` file will track the schema for the latest version.

You can then use this schema with the `yaml`, `toml` or `json` language servers to get autocompletion, type information and documentation for each element of the config file.

This can be done via your IDE's configuration or in most cases simply by using a directive at the top of your file that uses the link to the json schema.
