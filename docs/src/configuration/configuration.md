# Configuration

Sketch supports `yaml`, `json` and `toml` formats for its configuration file.

The strategy for detecting the configuration file works like this:
- If a file is manually specified with `--config`, that is used.
- Any file named `sketch.{yaml, json, toml}` in the cwd is used.
- Any file named `sketch.{yaml, json, toml}` in `XDG_CONFIG_HOME` or `$HOME/.config` is used.
- No file detected, using default settings.

Some of the values from configuration files can also be set via cli flags. When a value is set in a config file but also in a command, the value from the command has the higher priority.

You can also use the `--ignore-config` flag to temporarily ignore configuration files and only use cli-set values.

# Generating Config Files

You can use the `sketch new <OUTPUT>` command to generate a new configuration file in the desired output file and format (the default output is `sketch.yaml`).

# Extending Configurations

Configuration files can extend one another by using the `extends` field:

```yaml
extends: ["other_config.yaml"]
```

Where the path being used can be either an absolute path or a relative path starting from the original config file.

The [merging strategy](../presets/summary.md#extending-presets) for config files is the same as for all the other presets.

# Top Level Configuration

These are some of the default settings for the top level configuration values:

```yaml
{{#include ../../../examples/top_level_config.yaml:all}}
```
