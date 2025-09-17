# Configuration

Sketch supports `yaml`, `json` and `toml` formats for its configuration file.

The configuration file can be manually specified with the `-c` or `--config` flag. If none is provided, the cli will automatically look for a file named `sketch.{yaml, json, toml}` in the cwd. If none is found, then it will look in the `XDG_CONFIG_HOME` directory. If no config is found there, it will use default values.

You can also use the `--ignore-config-file` flag to temporarily ignore configuration files and only use cli-set values.
