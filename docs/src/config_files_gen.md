# Generating Config Files

You can use the `sketch new` command to generate a new configuration file in the desired output file and format. 
The default is `sketch.yaml`, but any name is supported as long as the format is either `json`, `toml` or `yaml`, and the configuration file is specified with the `-c` or `--config` flag.

If extra arguments are provided, the generated config will populated with the values of these arguments. 
So if we run `sketch --templates-dir "/path/to/templates"`, `templates_dir` will be set to that path in the generated config.

>⚠️ Since the default `sketch.yaml` file is always automatically detected, if there is such a file in the cwd and you want to create a new config with certain values, you need to use `--ignore-config-file` to ignore the config file's values.

Command:

```txt
{{#include ../../sketch/tests/output/generated_configs/with_extras_cmd}}
```

Output:

```yaml
{{#include ../../sketch/tests/output/generated_configs/with_extras.yaml}}
```

# Repo Setup

You can also use the `init` command to create a new git repo. This command will:

1. Create a new git repo in the specified `out_dir`.
2. If a `--remote` is provided, it will also add that remote as the origin/main for the repo.
3. Unless `pre-commit` is disabled, it will generate a new .pre-commit-config.yaml file in the root, with the repos specified in the config file (if there are any, otherwise it will just add the gitleaks repo). It will then run `pre-commit install` to install the given hooks.
