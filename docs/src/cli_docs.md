# Command-Line Help for `sketch`

This document contains the help content for the `sketch` command-line program.

**Command Overview:**

* [`sketch`↴](#sketch)
* [`sketch pre-commit`↴](#sketch-pre-commit)
* [`sketch ts`↴](#sketch-ts)
* [`sketch ts package-json`↴](#sketch-ts-package-json)
* [`sketch ts ts-config`↴](#sketch-ts-ts-config)
* [`sketch ts oxlint-config`↴](#sketch-ts-oxlint-config)
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

* `pre-commit` — Generates a `pre-commit` config file from a preset
* `ts` — Launches typescript-specific commands
* `repo` — Creates a new git repo
* `new` — Generates a new config file
* `render` — Renders a single template to a file or to stdout
* `render-preset` — Renders a templating preset defined in a configuration file
* `exec` — Renders a template and executes it as a shell command

###### **Options:**

* `-c`, `--config <FILE>` — Sets a custom config file. Any file named `sketch.{yaml,json,toml}` in the cwd or in `XDG_CONFIG_HOME/sketch` will be detected automatically. If no file is found, the default settings are used
* `--ignore-config` — Ignores any automatically detected config files, uses cli instructions only
* `--shell <SHELL>` — The shell to use for commands [default: `cmd.exe` on windows and `sh` elsewhere]
* `--debug` — Activates debugging mode
* `--templates-dir <DIR>` — The path to the templates directory, starting from the cwd (when set via cli) or from the config file (when defined in one of them)
* `--no-overwrite` — Does not overwrite existing files
* `--dry-run` — Aborts before writing any content to disk
* `-s`, `--set <KEY=VALUE>` — Sets a variable (as key=value) to use in templates. Overrides global and local variables



## `sketch pre-commit`

Generates a `pre-commit` config file from a preset

**Usage:** `sketch pre-commit --preset <ID> [OUTPUT]`

###### **Arguments:**

* `<OUTPUT>` — The output path of the created file [default: `.pre-commit-config.yaml`]

###### **Options:**

* `-p`, `--preset <ID>` — The preset id



## `sketch ts`

Launches typescript-specific commands

**Usage:** `sketch ts [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `package-json` — Generates a `package.json` file from a preset
* `ts-config` — Generates a `tsconfig.json` file from a preset
* `oxlint-config` — Generates a `.oxlintrc.json` file from a preset
* `monorepo` — Generates a new typescript monorepo
* `package` — Generates a new typescript package

###### **Options:**

* `--package-manager <NAME>` — The package manager being used. [default: pnpm]

  Possible values: `pnpm`, `npm`, `deno`, `bun`, `yarn`

* `--no-default-deps` — Does not add default dependencies to new `package.json` files (typescript and oxlint, plus vitest if enabled)
* `--version-range <KIND>` — The kind of version range to use for dependencies that are fetched automatically. [default: minor]

  Possible values: `patch`, `minor`, `exact`

* `--catalog` — Uses the pnpm catalog for default dependencies, and automatically adds dependencies marked with `catalog:` to the catalog, if they are missing
* `--no-convert-latest` — Does not convert dependencies marked as `latest` to a version range



## `sketch ts package-json`

Generates a `package.json` file from a preset

**Usage:** `sketch ts package-json --preset <ID> [OUTPUT]`

###### **Arguments:**

* `<OUTPUT>` — The output path of the generated file [default: `package.json`]

###### **Options:**

* `-p`, `--preset <ID>` — The preset id



## `sketch ts ts-config`

Generates a `tsconfig.json` file from a preset

**Usage:** `sketch ts ts-config --preset <ID> [OUTPUT]`

###### **Arguments:**

* `<OUTPUT>` — The output path of the generated file [default: `tsconfig.json`]

###### **Options:**

* `-p`, `--preset <ID>` — The preset id



## `sketch ts oxlint-config`

Generates a `.oxlintrc.json` file from a preset

**Usage:** `sketch ts oxlint-config --preset <ID> [OUTPUT]`

###### **Arguments:**

* `<OUTPUT>` — The output path of the generated file [default: `.oxlintrc.json`]

###### **Options:**

* `-p`, `--preset <ID>` — The preset id



## `sketch ts monorepo`

Generates a new typescript monorepo

**Usage:** `sketch ts monorepo [OPTIONS] [DIR]`

###### **Arguments:**

* `<DIR>` — The root directory for the new package. [default: `ts_root`]

###### **Options:**

* `-r`, `--root-package <ID>` — The id of the package preset to use for the root package. If unset, the default preset is used, along with the values set via cli flags
* `-n`, `--name <NAME>` — The name of the new package. It defaults to the name of its parent directory
* `-t`, `--ts-config <id=ID,output=PATH>` — One or many tsconfig presets (with their output path) to use for this package. If unset, defaults are used
* `--package-json <ID>` — The id of the package.json preset to use for this package
* `--with-template <id=TEMPLATE_ID,output=PATH>` — The templates to generate when this package is created. Relative output paths will be joined to the package's root directory
* `--oxlint <ID>` — The oxlint preset to use. It can be set to `default` to use the default preset
* `-i`, `--install` — Installs the dependencies at the root after creation



## `sketch ts package`

Generates a new typescript package

**Usage:** `sketch ts package [OPTIONS] [DIR]`

###### **Arguments:**

* `<DIR>` — The root directory for the new package. Defaults to the package name, if that is set

###### **Options:**

* `-p`, `--preset <ID>` — The package preset to use. If unset, the default preset is used, along with the values set via cli flags
* `-u`, `--update-tsconfig <UPDATE_TSCONFIG>` — An optional list of tsconfig paths where the new tsconfig file will be added as a reference
* `--no-vitest` — Does not set up vitest for this package
* `--oxlint <ID>` — The oxlint preset to use. It can be set to `default` to use the default preset
* `-i`, `--install` — Installs the dependencies with the chosen package manager
* `-n`, `--name <NAME>` — The name of the new package. It defaults to the name of its parent directory
* `-t`, `--ts-config <id=ID,output=PATH>` — One or many tsconfig presets (with their output path) to use for this package. If unset, defaults are used
* `--package-json <ID>` — The id of the package.json preset to use for this package
* `--with-template <id=TEMPLATE_ID,output=PATH>` — The templates to generate when this package is created. Relative output paths will be joined to the package's root directory



## `sketch repo`

Creates a new git repo

**Usage:** `sketch repo [OPTIONS] [DIR]`

###### **Arguments:**

* `<DIR>` — The directory where the new repo should be generated. [default: `.`]

###### **Options:**

* `-p`, `--preset <PRESET>` — Selects a git preset from a configuration file
* `--no-pre-commit` — Does not generate a pre-commit config. It overrides the value in the git preset if one is being used
* `--pre-commit <PRE_COMMIT>` — Selects a pre-commit preset. It overrides the value in the git preset if one is being used
* `--gitignore <GITIGNORE>` — Selects a gitignore preset. It overrides the value in the git preset if one is being used
* `-t`, `--with-template <id=TEMPLATE_ID,output=PATH>` — One or many templates to render in the new repo's root. If a preset is being used, the list is extended and not replaced
* `--remote <REMOTE>` — The link of the git remote to use for the new repo



## `sketch new`

Generates a new config file

**Usage:** `sketch new [OUTPUT]`

###### **Arguments:**

* `<OUTPUT>` — The output file [default: sketch.yaml]



## `sketch render`

Renders a single template to a file or to stdout

**Usage:** `sketch render [OPTIONS] <OUTPUT_PATH|--stdout>`

###### **Arguments:**

* `<OUTPUT_PATH>` — The output path for the generated file

###### **Options:**

* `--stdout` — Prints the result to stdout
* `-f`, `--file <FILE>` — The path to the template file, as an absolute path or relative to the cwd
* `-i`, `--id <ID>` — The id of the template to use (a name for config-defined templates, or a relative path to a file from `templates_dir`)
* `-c`, `--content <CONTENT>` — The literal definition for the template



## `sketch render-preset`

Renders a templating preset defined in a configuration file

**Usage:** `sketch render-preset <ID> [OUT_DIR]`

###### **Arguments:**

* `<ID>` — The id of the preset
* `<OUT_DIR>` — The base path to join to relative output paths. [default: `.`]



## `sketch exec`

Renders a template and executes it as a shell command

**Usage:** `sketch exec [OPTIONS] [CMD]`

###### **Arguments:**

* `<CMD>` — The literal definition for the template

###### **Options:**

* `--cwd <CWD>` — The cwd for the command to execute [default: `.`]
* `-f`, `--file <FILE>` — The path to the command's template file, as an absolute path or relative to the cwd
* `-t`, `--template <TEMPLATE>` — The id of the template to use (a name for config-defined templates, or a relative path to a file from `templates_dir`)



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
