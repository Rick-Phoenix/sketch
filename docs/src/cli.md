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

🖌️ A tool for generating project setups, leveraging the power of templating engines for boilerplate generation

**Usage:** `sketch [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `ts` — Launches typescript-specific commands
* `init` — Creates a new git repo with a gitignore file. Optionally, it sets up the git remote and the pre-commit config
* `new` — Generates a new config file with some optional initial values defined via the cli flags
* `render` — Generates a single file from a template
* `render-preset` — Generates content from a templating preset, with predefined content, output and context
* `exec` — Renders a template (from text or file) and launches it as a command

###### **Options:**

* `-c`, `--config <FILE>` — Sets a custom config file
* `--ignore-config-file` — Ignores any config files, uses cli instructions only
* `--shell <SHELL>` — The shell to use for commands [default: `cmd.exe` on windows and `sh` elsewhere]
* `--debug` — Activates debugging mode
* `--root-dir <DIR>` — The root directory for the project [default: "."]
* `--templates-dir <DIR>` — The path to the directory with the template files
* `--no-overwrite` — Exits with error if a file being created already exists
* `--dry-run` — Aborts before writing any content to disk
* `-s`, `--set <KEY=VALUE>` — Set a variable (as key=value) to use in templates. Overrides global and local variables



## `sketch ts`

Launches typescript-specific commands

**Usage:** `sketch ts [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `monorepo` — Generates a new typescript monorepo
* `package` — Generates a new typescript package

###### **Options:**

* `--root-tsconfig <NAME>` — The name of the tsconfig file to use at the root [default: 'tsconfig.options.json']
* `--project-tsconfig <NAME>` — The name of the tsconfig file for individual packages [default: 'tsconfig.src.json']
* `--dev-tsconfig <NAME>` — The name of the development tsconfig file [default: 'tsconfig.dev.json']
* `--package-manager <NAME>` — The package manager being used. [default: pnpm]

  Possible values: `pnpm`, `npm`, `deno`, `bun`, `yarn`

* `--version-range <KIND>` — The kind of version range to use for dependencies added automatically [default: minor]

  Possible values: `patch`, `minor`, `exact`

* `--catalog` — Whether to use the pnpm catalog for default dependencies
* `--no-convert-latest` — Whether the dependencies with `latest` should be converted to a version range (configurable in [`TypescriptConfig::version_ranges`]) with the actual latest version for that package
* `--shared-out-dir <SHARED_OUT_DIR>` — The path to the shared out_dir for TS packages
* `--no-shared-out-dir` — Does not use a shared out_dir for TS packages

  Default value: `false`



## `sketch ts monorepo`

Generates a new typescript monorepo

**Usage:** `sketch ts monorepo [OPTIONS]`

###### **Options:**

* `-n`, `--name <NAME>` — The name of the root package [default: "root"]
* `-t`, `--ts-config <output=PATH,id=ID>` — One or many tsconfig files for the root package. If unset, defaults are used
* `-p`, `--package-json <ID>` — The id of the package.json preset to use for the root package
* `--no-oxlint` — Does not generate an oxlint config at the root
* `--moonrepo` — Generate setup for moonrepo



## `sketch ts package`

Generates a new typescript package

**Usage:** `sketch ts package [OPTIONS] [DIR]`

###### **Arguments:**

* `<DIR>` — The new package's directory, starting from the `root_dir`. Defaults to the name of the package

###### **Options:**

* `-p`, `--preset <PRESET>` — The package preset to use
* `--moonrepo` — Sets up a basic moon.yml file
* `--no-vitest` — Does not set up vitest for this package
* `--oxlint` — Sets up an oxlint config file for this package
* `-i`, `--install` — Installs the dependencies with the chosen package manager
* `--app` — Marks the package as an application
* `--library` — Marks the package as a library
* `-t`, `--ts-config <output=PATH,id=ID>` — One or many tsconfig files for this package. If unset, defaults are used
* `--ts-out-dir <DIR>` — The out_dir for this package's tsconfig. Ignored if the default tsconfigs are not used
* `--package-json <ID>` — The id of the package.json preset to use for this package
* `-u`, `--update-root-tsconfig` — Adds the new package to the references in the root tsconfig



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

Generates a single file from a template

**Usage:** `sketch render [OPTIONS] <OUTPUT_PATH|--stdout>`

###### **Arguments:**

* `<OUTPUT_PATH>` — The output file (relative from the cwd)

###### **Options:**

* `--stdout` — Output the result to stdout
* `-i`, `--id <ID>` — The id of the template to select (cannot be used with the --content flag)
* `-c`, `--content <CONTENT>` — The literal definition for the template (cannot be used with the --id flag)



## `sketch render-preset`

Generates content from a templating preset, with predefined content, output and context

**Usage:** `sketch render-preset <ID>`

###### **Arguments:**

* `<ID>` — The id of the preset



## `sketch exec`

Renders a template (from text or file) and launches it as a command

**Usage:** `sketch exec [OPTIONS] [CMD]`

###### **Arguments:**

* `<CMD>` — The literal definition for the command's template (cannot be used with the --file flag)

###### **Options:**

* `-f`, `--file <FILE>` — The path to the command's template file
* `-t`, `--template <TEMPLATE>` — The id (or path inside templates_dir) of the template to use



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
