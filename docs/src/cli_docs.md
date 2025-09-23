# Command-Line Help for `sketch`

This document contains the help content for the `sketch` command-line program.

**Command Overview:**

* [`sketch`↴](#sketch)
* [`sketch ts`↴](#sketch-ts)
* [`sketch ts monorepo`↴](#sketch-ts-monorepo)
* [`sketch ts package`↴](#sketch-ts-package)
* [`sketch repo`↴](#sketch-repo)
* [`sketch new`↴](#sketch-new)
* [`sketch render`↴](#sketch-render)
* [`sketch render-preset`↴](#sketch-render-preset)
* [`sketch exec`↴](#sketch-exec)

## `sketch`

🖌️ Templating made portable. A tool to generate files, project structures or shell commands via custom templates

**Usage:** `sketch [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `ts` — Launches typescript-specific commands
* `repo` — Creates a new git repo with a generated gitignore file and, optionally, it sets up the git remote and the pre-commit config
* `new` — Generates a new config file with some optional initial values defined via cli flags
* `render` — Renders a single template to a file or to stdout
* `render-preset` — Renders a templating preset defined in the configuration file
* `exec` — Renders a template and executes it as a shell command

###### **Options:**

* `-c`, `--config <FILE>` — Sets a custom config file. Any file names `sketch.{yaml,json,toml}` in the cwd or in `XDG_CONFIG_HOME/sketch` will be detected automatically. If no file is found, the default settings are used
* `--ignore-config` — Ignores any automatically detected config files, uses cli instructions only
* `--shell <SHELL>` — The shell to use for commands [default: `cmd.exe` on windows and `sh` elsewhere]
* `--debug` — Activates debugging mode
* `--out-dir <DIR>` — This will be considered as the starting path for the executed commands. If this is a relative path, it will be joined to the cwd (when set via cli) or to the config file's directory [default: `.`]
* `--templates-dir <DIR>` — The path to the templates directory, starting from the cwd (when set via cli) or from the config file (when defined in one of them)
* `--no-overwrite` — Does not overwrite existing files
* `--dry-run` — Aborts before writing any content to disk
* `-s`, `--set <KEY=VALUE>` — Set a variable (as key=value) to use in templates. Overrides global and local variables



## `sketch ts`

Launches typescript-specific commands

**Usage:** `sketch ts [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `monorepo` — Generates a new typescript monorepo inside the `out_dir`
* `package` — Generates a new typescript package

###### **Options:**

* `--package-manager <NAME>` — The package manager being used. [default: pnpm]

  Possible values: `pnpm`, `npm`, `deno`, `bun`, `yarn`

* `--no-default-deps` — Does not add default dependencies to new `package.json` files (typescript and oxlint, plus vitest if enabled)
* `--version-range <KIND>` — The kind of version ranges to use for dependencies that are fetched automatically. [default: minor]

  Possible values: `patch`, `minor`, `exact`

* `--catalog` — Uses the pnpm catalog for default dependencies, and automatically adds dependencies marked with `catalog:` to `pnpm-workspace.yaml`
* `--no-convert-latest` — Does not convert dependencies marked as `latest` to a version range



## `sketch ts monorepo`

Generates a new typescript monorepo inside the `out_dir`

**Usage:** `sketch ts monorepo [OPTIONS] [DIR]`

###### **Arguments:**

* `<DIR>` — The new package's directory, starting from the `out_dir`. Defaults to the name of the package

###### **Options:**

* `-r`, `--root-package <ID>` — The id of the package preset to use for the root package
* `-n`, `--name <NAME>` — The name of the new package. If `dir` is set, it defaults to the last segment of it
* `-t`, `--ts-config <id=ID,output=PATH>` — One or many tsconfig files for this package. If unset, defaults are used
* `--package-json <ID>` — The id of the package.json preset to use for this package
* `--with-template <id=TEMPLATE_ID,output=PATH>` — The templates to generate when this package is created. Relative output paths will be joined to the package's root directory
* `--oxlint` — Generate a basic oxlint config at the root
* `-i`, `--install` — Installs the dependencies at the root after creation



## `sketch ts package`

Generates a new typescript package

**Usage:** `sketch ts package [OPTIONS] [DIR]`

###### **Arguments:**

* `<DIR>` — The new package's directory, starting from the `out_dir`. Defaults to the name of the package

###### **Options:**

* `-p`, `--preset <PRESET>` — The package preset to use
* `--update-root-tsconfig` — Whether the tsconfig file at the workspace root should receive a reference to the new package
* `--no-vitest` — Does not set up vitest for this package
* `--oxlint` — If an oxlint config is not defined or enabled, this will generate one with the default values
* `-i`, `--install` — Installs the dependencies with the chosen package manager
* `--app` — Marks the package as an application (only relevant for default tsconfigs)
* `--library` — Marks the package as a library (only relevant for default tsconfigs)
* `-n`, `--name <NAME>` — The name of the new package. If `dir` is set, it defaults to the last segment of it
* `-t`, `--ts-config <id=ID,output=PATH>` — One or many tsconfig files for this package. If unset, defaults are used
* `--package-json <ID>` — The id of the package.json preset to use for this package
* `--with-template <id=TEMPLATE_ID,output=PATH>` — The templates to generate when this package is created. Relative output paths will be joined to the package's root directory



## `sketch repo`

Creates a new git repo with a generated gitignore file and, optionally, it sets up the git remote and the pre-commit config

**Usage:** `sketch repo [OPTIONS]`

###### **Options:**

* `-p`, `--preset <PRESET>` — Selects a git preset from a configuration file
* `--no-pre-commit` — Does not generate a pre-commit config. It overrides the value in the git preset if one is being used
* `--pre-commit <PRE_COMMIT>` — Selects a pre-commit preset. It overrides the value in the git preset if one is being used
* `--gitignore <GITIGNORE>` — Selects a gitignore preset. It overrides the value in the git preset if one is being used
* `-t`, `--with-template <id=TEMPLATE_ID,output=PATH>` — One of many templates to render in the new repo's root. If a git preset with its own list of templates is being used, the lists are merged
* `--remote <REMOTE>` — The link to the git remote to use



## `sketch new`

Generates a new config file with some optional initial values defined via cli flags

**Usage:** `sketch new [OUTPUT]`

###### **Arguments:**

* `<OUTPUT>` — The output file. Must be an absolute path or a path relative from the cwd [default: sketch.yaml]



## `sketch render`

Renders a single template to a file or to stdout

**Usage:** `sketch render [OPTIONS] <OUTPUT_PATH|--stdout>`

###### **Arguments:**

* `<OUTPUT_PATH>` — The output file (relative from the cwd)

###### **Options:**

* `--stdout` — Prints the result to stdout
* `-f`, `--file <FILE>` — The path to the template file, as an absolute path or relative to the cwd
* `-i`, `--id <ID>` — The id of the template to use (a name for config-defined templates, or a relative path for a file inside `templates_dir`)
* `-c`, `--content <CONTENT>` — The literal definition for the template



## `sketch render-preset`

Renders a templating preset defined in the configuration file

**Usage:** `sketch render-preset <ID>`

###### **Arguments:**

* `<ID>` — The id of the preset



## `sketch exec`

Renders a template and executes it as a shell command

**Usage:** `sketch exec [OPTIONS] [CMD]`

###### **Arguments:**

* `<CMD>` — The literal definition for the template

###### **Options:**

* `--cwd <CWD>` — The cwd for the command to execute. [default: `.`]
* `-f`, `--file <FILE>` — The path to the command's template file, as an absolute path or relative to the cwd
* `-t`, `--template <TEMPLATE>` — The id of the template to use (a name for config-defined templates, or a relative path to a file inside `templates_dir`)



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
