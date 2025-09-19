# Configuration

Sketch supports `yaml`, `json` and `toml` formats for its configuration file.

The configuration file can be manually specified with the `-c` or `--config` flag. If none is provided, the cli will automatically look for a file named `sketch.{yaml, json, toml}` in the cwd. If none is found, then it will look in a directory called `sketch` inside `XDG_CONFIG_HOME` (if that is set, otherwise it will look inside `$HOME/.config` by default). If no config is found there, it will use default values.

You can also use the `--ignore-config-file` flag to temporarily ignore configuration files and only use cli-set values.

# Top Level Configuration

These are the defaults for the top level configuration values:

```yaml
{{#include ../../examples/top_level_config.yaml}}
```
