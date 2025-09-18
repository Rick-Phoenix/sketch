# Command-Line Help for `sketch`

This document contains the help content for the `sketch` command-line program.

**Command Overview:**

* [`sketch`↴](#sketch)
* [`sketch ts`↴](#sketch-ts)
* [`sketch ts monorepo`↴](#sketch-ts-monorepo)
* [`sketch ts package`↴](#sketch-ts-package)
* [`sketch init`↴](#sketch-init)
* [`sketch new`↴](#sketch-new)
* [`sketch render`↴](#sketch-render)
* [`sketch render-preset`↴](#sketch-render-preset)
* [`sketch exec`↴](#sketch-exec)

## `sketch`

🖌️ Templating made portable. A tool to generate files, project structures or shell commands via custom templates

**Usage:** `sketch [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `ts` — Launches typescript-specific commands
* `init` — Creates a new git repo with a gitignore file. Optionally, it sets up the git remote and the pre-commit config
* `new` — Generates a new config file with some optional initial values defined via the cli flags
* `render` — Renders a single template to a file or to stdout
* `render-preset` — Renders a templating preset defined in the configuration file
* `exec` — Renders a template and launches it as a command

###### **Options:**

* `-c`, `--config <FILE>` — Sets a custom config file
* `--ignore-config-file` — Ignores any config files, uses cli instructions only
* `--shell <SHELL>` — The shell to use for commands [default: `cmd.exe` on windows and `sh` elsewhere]
* `--debug` — Activates debugging mode
* `--root-dir <DIR>` — The base path for the generated files [default: "."]
* `--templates-dir <DIR>` — The path to the templates directory, starting from the cwd (when set via cli) or from the config file (when defined in one of them)
* `--no-overwrite` — Does not overwrite existing files
* `--dry-run` — Aborts before writing any content to disk
* `-s`, `--set <KEY=VALUE>` — Set a variable (as key=value) to use in templates. Overrides global and local variables



## `sketch ts`

Launches typescript-specific commands

**Usage:** `sketch ts [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `monorepo` — Generates a new typescript monorepo
* `package` — Generates a new typescript package

###### **Options:**

* `--package-manager <NAME>` — The package manager being used. [default: pnpm]

  Possible values: `pnpm`, `npm`, `deno`, `bun`, `yarn`

* `--no-default-deps` — Does not add default dependencies to new `package.json` files (typescript and oxlint, plus vitest if enabled)
* `--version-range <KIND>` — The kind of version ranges to use for dependencies that are fetched automatically. [default: minor]

  Possible values: `patch`, `minor`, `exact`

* `--catalog` — Uses the pnpm catalog for default dependencies
* `--no-convert-latest` — Does not convert dependencies marked as `latest` to a version range



## `sketch ts monorepo`

Generates a new typescript monorepo

**Usage:** `sketch ts monorepo [OPTIONS]`

###### **Options:**

* `-n`, `--name <NAME>` — The name of the root package [default: "root"]
* `-t`, `--ts-config <output=PATH,id=ID>` — One or many tsconfig files for the root package. If unset, defaults are used
* `-p`, `--package-json <ID>` — The id of the package.json preset to use for the root package
* `--no-oxlint` — Does not generate an oxlint config at the root
* `-i`, `--install` — Install the dependencies at the root after creation



## `sketch ts package`

Generates a new typescript package

**Usage:** `sketch ts package [OPTIONS] [DIR]`

###### **Arguments:**

* `<DIR>` — The new package's directory, starting from the `root_dir`. Defaults to the name of the package

###### **Options:**

* `-p`, `--preset <PRESET>` — The package preset to use
* `--update-root-tsconfig` — Whether the tsconfig file at the workspace root should receive a reference to the new package
* `--no-vitest` — Does not set up vitest for this package
* `--oxlint` — Sets up an oxlint config file for this package
* `-i`, `--install` — Installs the dependencies with the chosen package manager
* `--app` — Marks the package as an application (only relevant for default tsconfigs)
* `--library` — Marks the package as a library (only relevant for default tsconfigs)
* `-n`, `--name <NAME>` — The name of the new package. If `dir` is set, it defaults to the last segment of it
* `-t`, `--ts-config <output=PATH,id=ID>` — One or many tsconfig files for this package. If unset, defaults are used
* `--package-json <ID>` — The id of the package.json preset to use for this package



## `sketch init`

Creates a new git repo with a gitignore file. Optionally, it sets up the git remote and the pre-commit config

**Usage:** `sketch init [OPTIONS]`

###### **Options:**

* `--no-pre-commit` — Does not generate a pre-commit config
* `--remote <REMOTE>` — The link to the git remote to use



## `sketch new`

Generates a new config file with some optional initial values defined via the cli flags

**Usage:** `sketch new [OUTPUT]`

###### **Arguments:**

* `<OUTPUT>` — The output file [default: sketch.yaml]



## `sketch render`

Renders a single template to a file or to stdout

**Usage:** `sketch render [OPTIONS] <OUTPUT_PATH|--stdout>`

###### **Arguments:**

* `<OUTPUT_PATH>` — The output file (relative from the cwd)

###### **Options:**

* `--stdout` — Output the result to stdout
* `-f`, `--file <FILE>` — The path to the template file, from the cwd
* `-i`, `--id <ID>` — The id of the template to use
* `-c`, `--content <CONTENT>` — The literal definition for the template



## `sketch render-preset`

Renders a templating preset defined in the configuration file

**Usage:** `sketch render-preset <ID>`

###### **Arguments:**

* `<ID>` — The id of the preset



## `sketch exec`

Renders a template and launches it as a command

**Usage:** `sketch exec [OPTIONS] [CMD]`

###### **Arguments:**

* `<CMD>` — The literal definition for the command's template

###### **Options:**

* `-f`, `--file <FILE>` — The path to the command's template file
* `-t`, `--template <TEMPLATE>` — The id of the template to use



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
