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

Template:

```jinja
{{#include ../../../examples/templating/templates/example.j2}}
```

Cmd:

>`{{#include ../../../sketch/tests/output/templating_examples/cmd}}`

Output:

```
{{#include ../../../sketch/tests/output/templating_examples/output}}
```

