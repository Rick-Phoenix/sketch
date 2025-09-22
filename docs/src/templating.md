# Templating

Sketch uses the [Tera](https://keats.github.io/tera/docs/) templating engine to render custom templates. Every template, whether it's defined in a file, in the config file, or inlined, can take advantage of all of Tera's functionalities, like using filters, functions, defining and using macros, performing operations and importing other templates.

<div class="warning">
Since all autoescaping is disabled in templates, you should always only use templates that you made or trust.
</div>

# Special variables

Sketch provides some special variables prefixed with to get access to some commonly used information. All of the following variables are available in templates, prefixed with `sketch_` (i.e. `sketch_os` and so on).

- `cwd`
- `tmp_dir` (obtained with env::temp_dir)
- `home` (obtained with env::home_dir)
- `os` (env OS)
- `user` (env USER)
- `hostname` (env HOSTNAME)
- `arch` (env HOSTTYPE)
- `xdg_config` (env XDG_CONFIG_HOME)
- `xdg_data` (env XDG_DATA_HOME)
- `xdg_state` (env XDG_STATE_HOME)
- `xdg_cache` (env XDG_CACHE_HOME)

# Filters and functions

All of the builtin filters and functions for [Tera](https://keats.github.io/tera/docs/) are available. 

On top of that, Sketch adds a `uuid` function that can be used to generate a v4 UUID.

## Example

Config:

```yaml
{{#include ../../examples/templating.yaml}}
```

Cmd:

>`{{#include ../../sketch/tests/output/templating_examples/cmd}}`

Output:

```
{{#include ../../sketch/tests/output/templating_examples/output}}
```

# Global and Local Context

Variables are evaluated based on the locality of their context. Variables set via cli with the `--set` flag have the highest priority, followed by variables defined in a local context (the `context` field in a template preset) and by global variables defined in the config file.

<div class="warning">
Variables defined with the <code>--set</code> flag must be formatted in valid json. This means that, for example, strings must be wrapped in escaped quotes.
</div>

