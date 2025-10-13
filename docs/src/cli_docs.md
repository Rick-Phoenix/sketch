# Command-Line Help for `sketch`

This document contains the help content for the `sketch` command-line program.

**Command Overview:**

* [`sketch`↴](#sketch)
* [`sketch new`↴](#sketch-new)
* [`sketch repo`↴](#sketch-repo)
* [`sketch render`↴](#sketch-render)
* [`sketch render-preset`↴](#sketch-render-preset)
* [`sketch exec`↴](#sketch-exec)
* [`sketch gh-workflow`↴](#sketch-gh-workflow)
* [`sketch docker-compose`↴](#sketch-docker-compose)
* [`sketch pre-commit`↴](#sketch-pre-commit)
* [`sketch cargo-toml`↴](#sketch-cargo-toml)
* [`sketch ts`↴](#sketch-ts)
* [`sketch ts monorepo`↴](#sketch-ts-monorepo)
* [`sketch ts package`↴](#sketch-ts-package)
* [`sketch ts barrel`↴](#sketch-ts-barrel)
* [`sketch package-json`↴](#sketch-package-json)
* [`sketch ts-config`↴](#sketch-ts-config)
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
* `render-preset` — Renders a templating preset
* `exec` — Renders a template and executes it as a shell command
* `gh-workflow` — Generates a Github workflow
* `docker-compose` — Generates a Docker Compose file from a preset
* `pre-commit` — Generates a `pre-commit` config file from a preset
* `cargo-toml` — Generates a `Cargo.toml` file from a preset
* `ts` — Executes typescript-specific commands
* `package-json` — Generates a `package.json` file from a preset
* `ts-config` — Generates a `tsconfig.json` file from a preset
* `oxlint` — Generates a `.oxlintrc.json` file from a preset
* `pnpm-workspace` — Generates a `pnpm-workspace.yaml` file from a preset
* `license` — 

###### **Options:**

* `--print-config` — Prints the full parsed config
* `--templates-dir <DIR>` — The path to the templates directory
* `--no-overwrite` — Do not overwrite existing files
* `-c`, `--config <FILE>` — Sets a custom config file. Any file named `sketch.{yaml,json,toml}` in the cwd or in `XDG_CONFIG_HOME/sketch` will be detected automatically. If no file is found, the default settings are used
* `--ignore-config` — Ignores any automatically detected config files, uses cli instructions only
* `-s`, `--set <KEY=VALUE>` — Sets a variable (as key=value) to use in templates. Overrides global and local variables. Values must be in valid JSON
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
* `--no-pre-commit` — Do not generate a pre-commit config
* `--pre-commit <PRE_COMMIT>` — Selects a pre-commit preset
* `-g`, `--gitignore <GITIGNORE>` — Selects a gitignore preset
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

* `-w`, `--with-template <PRESET_ID|id=TEMPLATE_ID,output=PATH>` — One or many individual templates or templating presets to render in the new repo
* `--workflow <id=PRESET_ID,file=PATH>` — One or many workflow presets to use for the new repo. The file path will be joined to `.github/workflows`
* `-r`, `--remote <REMOTE>` — The link of the git remote to use for the new repo



## `sketch render`

Renders a single template to a file or to stdout

**Usage:** `sketch render [OPTIONS] <OUTPUT_PATH|--stdout>`

###### **Arguments:**

* `<OUTPUT_PATH>` — The output path for the generated file

###### **Options:**

* `--stdout` — Prints the result to stdout
* `-f`, `--file <FILE>` — The path to the template file
* `-i`, `--id <ID>` — The id of the template to use (a name for config-defined templates, or a relative path to a file from `templates_dir`)
* `-c`, `--content <CONTENT>` — The literal definition for the template



## `sketch render-preset`

Renders a templating preset

**Usage:** `sketch render-preset <ID> [OUT_DIR]`

###### **Arguments:**

* `<ID>` — The id of the preset
* `<OUT_DIR>` — The base path to join to relative output paths. [default: `.`]



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



## `sketch gh-workflow`

Generates a Github workflow

**Usage:** `sketch gh-workflow <PRESET> <OUTPUT>`

###### **Arguments:**

* `<PRESET>` — The preset id
* `<OUTPUT>` — The output path of the new file



## `sketch docker-compose`

Generates a Docker Compose file from a preset

**Usage:** `sketch docker-compose <PRESET> [OUTPUT]`

###### **Arguments:**

* `<PRESET>` — The preset id
* `<OUTPUT>` — The output path of the new file [default: `compose.yaml`]



## `sketch pre-commit`

Generates a `pre-commit` config file from a preset

**Usage:** `sketch pre-commit <PRESET> [OUTPUT]`

###### **Arguments:**

* `<PRESET>` — The preset id
* `<OUTPUT>` — The output path of the new file [default: `.pre-commit-config.yaml`]



## `sketch cargo-toml`

Generates a `Cargo.toml` file from a preset

**Usage:** `sketch cargo-toml <PRESET> [OUTPUT]`

###### **Arguments:**

* `<PRESET>` — The preset id
* `<OUTPUT>` — The output path of the new file [default: `Cargo.toml`]



## `sketch ts`

Executes typescript-specific commands

**Usage:** `sketch ts [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `monorepo` — Generates a new typescript monorepo
* `package` — Generates a new typescript package
* `barrel` — Creates a barrel file

###### **Options:**

* `--package-manager <NAME>` — The package manager being used. [default: pnpm]

  Possible values: `pnpm`, `npm`, `deno`, `bun`, `yarn`

* `--no-default-deps <NO_DEFAULT_DEPS>` — Do not add default dependencies to new `package.json` files (typescript and oxlint, plus vitest if enabled)

  Possible values: `true`, `false`

* `--version-range <KIND>` — The kind of version range to use for dependencies that are fetched automatically. [default: minor]

  Possible values: `patch`, `minor`, `exact`

* `--catalog <CATALOG>` — Uses the pnpm catalog for default dependencies, and automatically adds dependencies marked with `catalog:` to the catalog, if they are missing

  Possible values: `true`, `false`

* `--no-convert-latest <NO_CONVERT_LATEST_TO_RANGE>` — Do not convert dependencies marked as `latest` to a version range

  Possible values: `true`, `false`




## `sketch ts monorepo`

Generates a new typescript monorepo

**Usage:** `sketch ts monorepo [OPTIONS] [DIR]`

###### **Arguments:**

* `<DIR>` — The root directory for the new monorepo. [default: `ts_root`]

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

* `--hook-pre <ID>` — One or many IDs of templates to render and execute as commands before the package's creation
* `--hook-post <ID>` — One or many IDs of templates to render and execute as commands after the package's creation
* `--oxlint <ID>` — The oxlint preset to use. It can be set to `default` to use the default preset
* `-i`, `--install` — Installs the dependencies with the chosen package manager
* `-w`, `--with-template <PRESET_ID|id=TEMPLATE_ID,output=PATH>` — One or many templates or templating presets to generate in the new package's root



## `sketch ts package`

Generates a new typescript package

**Usage:** `sketch ts package [OPTIONS] [DIR]`

###### **Arguments:**

* `<DIR>` — The root directory for the new package. Defaults to the package name

###### **Options:**

* `-p`, `--preset <ID>` — The package preset to use. If unset, the default preset is used, along with the values set via cli flags
* `-u`, `--update-tsconfig <UPDATE_TSCONFIG>` — An optional list of tsconfig files where the new tsconfig file will be added as a reference
* `--oxlint <ID>` — The oxlint preset to use. It can be set to `default` to use the default preset
* `-i`, `--install` — Installs the dependencies with the chosen package manager
* `-w`, `--with-template <PRESET_ID|id=TEMPLATE_ID,output=PATH>` — One or many templates or templating presets to generate in the new package's root
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

* `--hook-pre <ID>` — One or many IDs of templates to render and execute as commands before the package's creation
* `--hook-post <ID>` — One or many IDs of templates to render and execute as commands after the package's creation



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



## `sketch package-json`

Generates a `package.json` file from a preset

**Usage:** `sketch package-json <PRESET> [OUTPUT]`

###### **Arguments:**

* `<PRESET>` — The preset id
* `<OUTPUT>` — The output path of the generated file [default: `package.json`]



## `sketch ts-config`

Generates a `tsconfig.json` file from a preset

**Usage:** `sketch ts-config <PRESET> [OUTPUT]`

###### **Arguments:**

* `<PRESET>` — The preset id
* `<OUTPUT>` — The output path of the generated file [default: `tsconfig.json`]



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


###### **Options:**

* `-o`, `--output <OUTPUT>` — The path of the output file [default: `LICENSE`]



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
