# Basic Setup

You can use the `sketch new` command to generate a new configuration file in the desired output file and format. 
The default is `sketch.yaml`, but any name is supported as long as the format is either `json`, `toml` or `yaml`, and the configuration file is specified with the `-c` or `--config` flag.

If called with added arguments, it will generate a config with the provided values. 
Since the default `sketch.yaml` file is always automatically detected, if there is such a file in the cwd and you want to create a new config with certain values, you need to use `--no-config-file` to ignore the config file's values.

Command:

```txt
{{#include ../../sketch/tests/output/generated_configs/with_extras_cmd}}
```

Output:

```yaml
{{#include ../../sketch/tests/output/generated_configs/with_extras.yaml}}
```


