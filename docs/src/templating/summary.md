# Templating

Sketch uses the [Tera](https://keats.github.io/tera/docs/) templating engine to render custom templates. Every template, whether it's defined in a file, in the config file, or inlined, can take advantage of all of Tera's functionalities, like using filters, functions, defining and using macros, performing operations and importing other templates.

<div class="warning">
Since all autoescaping is disabled in templates, you should always only use templates that you made or trust.
</div>

# Special Variables

Sketch provides some special variables to get access to commonly used values such as the home directory. 

All of the following variables are available in templates, prefixed with `sketch_` (i.e. `sketch_os` and so on).

- `cwd`
- `tmp_dir` (`env::temp_dir`)
- `home` (`env::home_dir`)
- `os` (`CARGO_CFG_TARGET_OS` or `OS`)
- `os_family` (`CARGO_CFG_TARGET_FAMILY`)
- `arch` (`CARGO_CFG_TARGET_ARCH` or `HOSTTYPE`)
- `user` (`USER`)
- `hostname` (`HOSTNAME`)
- `xdg_config` (`XDG_CONFIG_HOME`)
- `xdg_data` (`XDG_DATA_HOME`)
- `xdg_state` (`XDG_STATE_HOME`)
- `xdg_cache` (`XDG_CACHE_HOME`)
- `is_windows` (`cfg!(windows)`)
- `is_unix` (`cfg!(unix)`)
- `is_wsl` (checks `/proc/sys/kernel/osrelease`)

# Filters And Functions

All of the builtin functionalities for [Tera](https://keats.github.io/tera/docs/) are available. 

On top of that, Sketch adds some extra filters and functions.

## Functions

- `uuid` (generates a v4 UUID)

## Filters

### Strings

- `capture(regex=REGEX)` (matches a regex once and returns the named capture groups)
- `capture_many(regex=REGEX)` (matches a regex repetitively and returns the list of named capture groups)
- `semver` (parses a cargo-style semver and returns the segments)
- `matches_semver(target=TARGET)` (checks if a cargo-style semver matches a target)
- `strip_prefix(prefix=PREFIX)` (strips a prefix from a string, if present)
- `strip_suffix(suffix=SUFFIX)` (strips a suffix from a string, if present)

### Filesystem

- `basename` (gets the basename of a directory/file)
- `parent_dir` (gets the parent directory of a directory/file)
- `is_file` (checks if a path is a file)
- `is_dir` (checks if a path is a directory)
- `is_absolute` (checks if a path is absolute)
- `is_relative` (checks if a path is relative)
- `absolute` (makes a path absolute)
- `relative(from=PATH)` (returns the relative path between two paths)
- `read_dir` (returns the list of the files contained in a directory and its subdirectories)
- `glob(pattern=GLOB)` (returns the glob matching entries in a directory and its subdirectories)
- `matches_glob(pattern=GLOB)` (checks if a path matches a glob pattern)

# Examples

Template

```jinja
{{#include ../../../examples/templating/templates/example.j2}}
```

Cmd:

>`{{#include ../../../sketch/tests/output/templating_examples/cmd}}`

Output:

```
{{#include ../../../sketch/tests/output/templating_examples/output}}
```

# Context Priority

Variables can be set at many different stages, with different degrees of priority which roughly follow this schema (from lowest to highest):

- Global variables
- Preset context
- Single template context
- Cli-set variables

<div class="warning">
Variables defined with the <code>--set</code> flag must be formatted in valid json. This means that, for example, strings must be wrapped in escaped quotes.
</div>

