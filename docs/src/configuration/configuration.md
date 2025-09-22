# Configuration

Sketch supports `yaml`, `json` and `toml` formats for its configuration file.

The configuration file can be manually specified with the `-c` or `--config` flag. If none is provided, the cli will automatically look for a file named `sketch.{yaml, json, toml}` in the cwd. If none is found, then it will look in a directory called `sketch` inside `XDG_CONFIG_HOME` (if that is set, otherwise it will look inside `$HOME/.config` by default). If no config is found there, it will use default values.

You can also use the `--ignore-config` flag to temporarily ignore configuration files and only use cli-set values.

# Generating Config Files

You can use the `sketch new <OUTPUT>` command to generate a new configuration file in the desired output file and format (the default output is `sketch.yaml`).

# Extending Configurations

Configuration files can extend one another by using the `extends` field:

```yaml
extends: ["other_config.yaml"]
```

Where the path being used can be either an absolute path or a relative path starting from the original config file.

## Merging Logic

The merging strategy works as follows:
- For conflicting values, such as opposite booleans, the previous value will be overridden.
- For non-conflicting values such as maps (for example, the global template vars map), the values will be merged.
- For extensible sub-configurations, such as that for `Typescript`, these are merged with one another, with the same principles as above.

# Top Level Configuration

These are the defaults for the top level configuration values:

```yaml
{{#include ../../../examples/top_level_config.yaml}}
```
