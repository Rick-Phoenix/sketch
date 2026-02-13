# Command-Line Help for `sketch`

This document contains the help content for the `sketch` command-line program.

**Command Overview:**

* [`sketch`↴](#sketch)
* [`sketch new`↴](#sketch-new)
* [`sketch repo`↴](#sketch-repo)
* [`sketch render`↴](#sketch-render)
* [`sketch exec`↴](#sketch-exec)
* [`sketch gitignore`↴](#sketch-gitignore)
* [`sketch gh-workflow`↴](#sketch-gh-workflow)
* [`sketch docker-compose`↴](#sketch-docker-compose)
* [`sketch pre-commit`↴](#sketch-pre-commit)
* [`sketch rust`↴](#sketch-rust)
* [`sketch rust crate`↴](#sketch-rust-crate)
* [`sketch rust manifest`↴](#sketch-rust-manifest)
* [`sketch ts`↴](#sketch-ts)
* [`sketch ts monorepo`↴](#sketch-ts-monorepo)
* [`sketch ts package`↴](#sketch-ts-package)
* [`sketch ts barrel`↴](#sketch-ts-barrel)
* [`sketch ts config`↴](#sketch-ts-config)
* [`sketch package-json`↴](#sketch-package-json)
* [`sketch oxlint`↴](#sketch-oxlint)
* [`sketch pnpm-workspace`↴](#sketch-pnpm-workspace)
* [`sketch license`↴](#sketch-license)

## `sketch`

🖌️ Templating made simple. A tool to define and generate files and reusable project structures

**Usage:** `sketch [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `new` — Generates a new config file
* `repo` — Creates a new git repo from a preset
* `render` — Renders a single template to a file or to stdout
* `exec` — Renders a template and executes it as a shell command
* `gitignore` — Generates a `.gitignore` file from a preset
* `gh-workflow` — Generates a Github workflow
* `docker-compose` — Generates a Docker Compose file from a preset
* `pre-commit` — Generates a `pre-commit` config file from a preset
* `rust` — The subcommands to generate files used in Rust workspaces
* `ts` — Executes typescript-specific commands
* `package-json` — Generates a `package.json` file from a preset
* `oxlint` — Generates a `.oxlintrc.json` file from a preset
* `pnpm-workspace` — Generates a `pnpm-workspace.yaml` file from a preset
* `license` — Generates a license file

###### **Options:**

* `--print-config` — Prints the full parsed config
* `--templates-dir <DIR>` — The path to the templates directory
* `--no-overwrite` — Do not overwrite existing files
* `-c`, `--config <FILE>` — Sets a custom config file. Any file named `sketch.{yaml,json,toml}` in the cwd or in `XDG_CONFIG_HOME/sketch` will be detected automatically. If no file is found, the default settings are used
* `--ignore-config` — Ignores any automatically detected config files, uses cli instructions and config file defined with --config
* `-S`, `--set <KEY=VALUE>` — Sets a variable (as key=value) to use in templates. Overrides global and local variables. Values must be in valid JSON
* `--vars-file <VARS_FILES>` — One or more paths to json, yaml or toml files to extract template variables from, in the given order



## `sketch new`

Generates a new config file

**Usage:** `sketch new [OUTPUT]`

###### **Arguments:**

* `<OUTPUT>` — The output file [default: sketch.yaml]



## `sketch repo`

Creates a new git repo from a preset

**Usage:** `sketch repo [OPTIONS] [DIR]`

###### **Arguments:**

* `<DIR>` — The directory where the new repo should be generated. [default: `.`]

###### **Options:**

* `-p`, `--preset <PRESET>` — Selects a git preset from a configuration file
* `-g`, `--gitignore <GITIGNORE>` — Settings for the gitignore file
* `--pre-commit <PRE_COMMIT>` — Configuration settings for [`pre-commit`](https://pre-commit.com/)
* `-t`, `--template <WITH_TEMPLATES>` — A set of templates to generate when this preset is used
* `-l`, `--license <LICENSE>` — A license file to generate for the new repo

  Possible values:
  - `apache2`:
    Apache 2.0 license
  - `gpl3`:
    GNU GPL 3.0 license
  - `agpl3`:
    GNU AGPL 3.0 license
  - `mit`:
    MIT license
  - `mpl2`

* `--workflow <id=PRESET_ID,file=PATH>` — One or many workflows to generate in the new repo
* `-r`, `--remote <REMOTE>` — The link of the git remote to use for the new repo



## `sketch render`

Renders a single template to a file or to stdout

**Usage:** `sketch render [OPTIONS] [OUTPUT]`

###### **Arguments:**

* `<OUTPUT>` — The output path for the template/preset. Implies `stdout` if absent for single templates. Required when a preset is selected

###### **Options:**

* `-p`, `--preset <PRESET>` — The id of a templating preset
* `-f`, `--file <FILE>` — The path to the template file
* `-t`, `--template <TEMPLATE>` — The id of the template to use (a name for config-defined templates, or a relative path to a file from `templates_dir`)
* `-c`, `--content <CONTENT>` — The literal definition for the template



## `sketch exec`

Renders a template and executes it as a shell command

**Usage:** `sketch exec [OPTIONS] [CMD]`

###### **Arguments:**

* `<CMD>` — The literal definition for the template (incompatible with `--file` or `--template`)

###### **Options:**

* `--print-cmd` — Prints the rendered command to stdout before executing it
* `-s`, `--shell <SHELL>` — The shell to use for commands [default: `cmd.exe` on windows and `sh` elsewhere]
* `--cwd <CWD>` — The cwd for the command to execute [default: `.`]
* `-f`, `--file <FILE>` — The path to the command's template file, as an absolute path or relative to the cwd
* `-t`, `--template <TEMPLATE>` — The id of the template to use (a name for config-defined templates, or a relative path to a file from `templates_dir`)



## `sketch gitignore`

Generates a `.gitignore` file from a preset

**Usage:** `sketch gitignore <PRESET> [OUTPUT]`

###### **Arguments:**

* `<PRESET>` — The preset id
* `<OUTPUT>` — The output path of the new file [default: `.gitignore`]



## `sketch gh-workflow`

Generates a Github workflow

**Usage:** `sketch gh-workflow <PRESET> <OUTPUT>`

###### **Arguments:**

* `<PRESET>` — The preset id
* `<OUTPUT>` — The output path of the new file



## `sketch docker-compose`

Generates a Docker Compose file from a preset

**Usage:** `sketch docker-compose [OPTIONS] [PRESET] [OUTPUT]`

###### **Arguments:**

* `<PRESET>` — The preset id. Not required if services are added manually with the `--service` flag
* `<OUTPUT>` — The output path of the new file [default: `compose.yaml`]

###### **Options:**

* `-s`, `--service <SERVICES>` — PRESET_ID|id=PRESET,name=NAME



## `sketch pre-commit`

Generates a `pre-commit` config file from a preset

**Usage:** `sketch pre-commit <PRESET> [OUTPUT]`

###### **Arguments:**

* `<PRESET>` — The preset id
* `<OUTPUT>` — The output path of the new file [default: `.pre-commit-config.yaml`]



## `sketch rust`

The subcommands to generate files used in Rust workspaces

**Usage:** `sketch rust <COMMAND>`

###### **Subcommands:**

* `crate` — 
* `manifest` — Generates a new `Cargo.toml` file from a preset



## `sketch rust crate`

**Usage:** `sketch rust crate [OPTIONS] <DIR>`

###### **Arguments:**

* `<DIR>` — The output directory for the new crate. Also the name of the generated crate by default

###### **Options:**

* `-p`, `--preset <PRESET>` — The crate preset to use
* `-m`, `--manifest <MANIFEST>` — The `Cargo.toml` manifest preset to use (overrides the one in the preset if one was selected)
* `-n`, `--name <NAME>` — The name of the generated crate (by default, it uses the name of the output dir)
* `--gitignore <GITIGNORE>` — Settings for the gitignore file
* `--license <LICENSE>` — A license file to generate for the new repo

  Possible values:
  - `apache2`:
    Apache 2.0 license
  - `gpl3`:
    GNU GPL 3.0 license
  - `agpl3`:
    GNU AGPL 3.0 license
  - `mit`:
    MIT license
  - `mpl2`

* `-t`, `--template <PRESET_ID>`



## `sketch rust manifest`

Generates a new `Cargo.toml` file from a preset

**Usage:** `sketch rust manifest <PRESET> [OUTPUT]`

###### **Arguments:**

* `<PRESET>` — The id of the preset
* `<OUTPUT>` — The output path [default: `Cargo.toml`]



## `sketch ts`

Executes typescript-specific commands

**Usage:** `sketch ts [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `monorepo` — Generates a new typescript monorepo
* `package` — Generates a new typescript package
* `barrel` — Creates a barrel file
* `config` — Generates a `tsconfig.json` file from a preset

###### **Options:**

* `--package-manager <NAME>` — The package manager being used. [default: pnpm]

  Possible values: `pnpm`, `npm`, `deno`, `bun`, `yarn`

* `--no-default-deps <NO_DEFAULT_DEPS>` — Do not add default dependencies to new `package.json` files (typescript and oxlint, plus vitest if enabled)

  Possible values: `true`, `false`

* `--version-range <KIND>` — The kind of version range to use for dependencies that are fetched automatically. [default: major]

  Possible values: `major`, `minor`, `exact`

* `--catalog <CATALOG>` — Uses the default catalog (supported by pnpm and bun) for default dependencies, and automatically adds dependencies marked with `catalog:` to the catalog, if they are missing

  Possible values: `true`, `false`

* `--no-convert-latest <NO_CONVERT_LATEST_TO_RANGE>` — Do not convert dependencies marked as `latest` to a version range

  Possible values: `true`, `false`




## `sketch ts monorepo`

Generates a new typescript monorepo

**Usage:** `sketch ts monorepo [OPTIONS] <DIR>`

###### **Arguments:**

* `<DIR>` — The root directory for the new monorepo

###### **Options:**

* `-p`, `--pnpm <PRESET_ID>` — The `pnpm-workspace.yaml` preset to use for the new monorepo. If it's unset and `pnpm` is the chosen package manager, the default preset will be used
* `-r`, `--root-package <PRESET_ID>` — The id of the package preset to use for the root package. If unset, the default preset is used, along with the values set via cli flags
* `-n`, `--name <NAME>` — The name of the new package. It defaults to the name of its directory
* `--ts-config <id=ID,output=PATH>` — One or many tsconfig presets (with their output path) to use for this package (uses defaults if not provided)
* `--package-json <ID>` — The package.json preset ID to use (uses defaults if not provided)
* `--license <LICENSE>` — A license file to generate for the new package

  Possible values:
  - `apache2`:
    Apache 2.0 license
  - `gpl3`:
    GNU GPL 3.0 license
  - `agpl3`:
    GNU AGPL 3.0 license
  - `mit`:
    MIT license
  - `mpl2`

* `-t`, `--template <PRESET_ID>` — One or many templates to generate along with this package. Relative output paths will resolve from the root of the package
* `--hook-pre <ID>` — One or many IDs of templates to render and execute as commands before the package's creation
* `--hook-post <ID>` — One or many IDs of templates to render and execute as commands after the package's creation
* `--oxlint <ID>` — The configuration for this package's oxlint setup. It can be set to `true` (to use defaults), to a preset id, or to a literal configuration
* `-i`, `--install` — Installs the dependencies with the chosen package manager



## `sketch ts package`

Generates a new typescript package

**Usage:** `sketch ts package [OPTIONS] [DIR]`

###### **Arguments:**

* `<DIR>` — The root directory for the new package. Defaults to the package name

###### **Options:**

* `-p`, `--preset <ID>` — The package preset to use. If unset, the default preset is used, along with the values set via cli flags
* `-u`, `--update-tsconfig <UPDATE_TSCONFIG>` — An optional list of tsconfig files where the new tsconfig file will be added as a reference
* `-i`, `--install` — Installs the dependencies with the chosen package manager
* `--vitest <ID>` — The vitest preset to use. It can be set to `default` to use the default preset
* `-n`, `--name <NAME>` — The name of the new package. It defaults to the name of its directory
* `--ts-config <id=ID,output=PATH>` — One or many tsconfig presets (with their output path) to use for this package (uses defaults if not provided)
* `--package-json <ID>` — The package.json preset ID to use (uses defaults if not provided)
* `--license <LICENSE>` — A license file to generate for the new package

  Possible values:
  - `apache2`:
    Apache 2.0 license
  - `gpl3`:
    GNU GPL 3.0 license
  - `agpl3`:
    GNU AGPL 3.0 license
  - `mit`:
    MIT license
  - `mpl2`

* `-t`, `--template <PRESET_ID>` — One or many templates to generate along with this package. Relative output paths will resolve from the root of the package
* `--hook-pre <ID>` — One or many IDs of templates to render and execute as commands before the package's creation
* `--hook-post <ID>` — One or many IDs of templates to render and execute as commands after the package's creation
* `--oxlint <ID>` — The configuration for this package's oxlint setup. It can be set to `true` (to use defaults), to a preset id, or to a literal configuration



## `sketch ts barrel`

Creates a barrel file

**Usage:** `sketch ts barrel [OPTIONS] [DIR]`

###### **Arguments:**

* `<DIR>` — The directory where to search recursively for the files and generate the barrel file [default: `.`]

###### **Options:**

* `-o`, `--output <OUTPUT>` — The output path for the barrel file. It defaults to `{dir}/index.ts`
* `--keep-ext <EXT>` — The file extensions that should be kept in export statements
* `--js-ext` — Exports `.ts` files as `.js`. It assumes that `js` is among the file extensions to keep
* `--exclude <EXCLUDE>` — One or more glob patterns to exclude from the imported modules



## `sketch ts config`

Generates a `tsconfig.json` file from a preset

**Usage:** `sketch ts config <PRESET> [OUTPUT]`

###### **Arguments:**

* `<PRESET>` — The preset id
* `<OUTPUT>` — The output path of the generated file [default: `tsconfig.json`]



## `sketch package-json`

Generates a `package.json` file from a preset

**Usage:** `sketch package-json <PRESET> [OUTPUT]`

###### **Arguments:**

* `<PRESET>` — The preset id
* `<OUTPUT>` — The output path of the generated file [default: `package.json`]



## `sketch oxlint`

Generates a `.oxlintrc.json` file from a preset

**Usage:** `sketch oxlint <PRESET> [OUTPUT]`

###### **Arguments:**

* `<PRESET>` — The preset id
* `<OUTPUT>` — The output path of the generated file [default: `.oxlintrc.json`]



## `sketch pnpm-workspace`

Generates a `pnpm-workspace.yaml` file from a preset

**Usage:** `sketch pnpm-workspace <PRESET> [OUTPUT]`

###### **Arguments:**

* `<PRESET>` — The preset id
* `<OUTPUT>` — The output path of the generated file [default: `pnpm-workspace.yaml`]



## `sketch license`

Generates a license file

**Usage:** `sketch license [OPTIONS] <LICENSE>`

###### **Arguments:**

* `<LICENSE>`

  Possible values:
  - `apache2`:
    Apache 2.0 license
  - `gpl3`:
    GNU GPL 3.0 license
  - `agpl3`:
    GNU AGPL 3.0 license
  - `mit`:
    MIT license
  - `mpl2`


###### **Options:**

* `-o`, `--output <OUTPUT>` — The path of the output file [default: `LICENSE`]



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
