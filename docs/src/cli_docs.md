# Command-Line Help for `sketch`

This document contains the help content for the `sketch` command-line program.

**Command Overview:**

* [`sketch`↴](#sketch)
* [`sketch pre-commit`↴](#sketch-pre-commit)
* [`sketch ts`↴](#sketch-ts)
* [`sketch ts pnpm-workspace`↴](#sketch-ts-pnpm-workspace)
* [`sketch ts package-json`↴](#sketch-ts-package-json)
* [`sketch ts ts-config`↴](#sketch-ts-ts-config)
* [`sketch ts oxlint`↴](#sketch-ts-oxlint)
* [`sketch ts monorepo`↴](#sketch-ts-monorepo)
* [`sketch ts package`↴](#sketch-ts-package)
* [`sketch ts barrel`↴](#sketch-ts-barrel)
* [`sketch repo`↴](#sketch-repo)
* [`sketch new`↴](#sketch-new)
* [`sketch render`↴](#sketch-render)
* [`sketch render-preset`↴](#sketch-render-preset)
* [`sketch exec`↴](#sketch-exec)

## `sketch`

🖌️ Templating made simple. Define and generate reusable structures for all sorts of projects

**Usage:** `sketch [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `pre-commit` — Generates a `pre-commit` config file from a preset
* `ts` — Executes typescript-specific commands
* `repo` — Creates a new git repo from a preset
* `new` — Generates a new config file
* `render` — Renders a single template to a file or to stdout
* `render-preset` — Renders a templating preset
* `exec` — Renders a template and executes it as a shell command

###### **Options:**

* `-c`, `--config <FILE>` — Sets a custom config file. Any file named `sketch.{yaml,json,toml}` in the cwd or in `XDG_CONFIG_HOME/sketch` will be detected automatically. If no file is found, the default settings are used
* `--ignore-config` — Ignores any automatically detected config files, uses cli instructions only
* `--shell <SHELL>` — The shell to use for commands [default: `cmd.exe` on windows and `sh` elsewhere]
* `--debug` — Activates debugging mode
* `--templates-dir <DIR>` — The path to the templates directory
* `--no-overwrite` — Do not overwrite existing files
* `-s`, `--set <KEY=VALUE>` — Sets a variable (as key=value) to use in templates. Overrides global and local variables. Values must be in valid JSON



## `sketch pre-commit`

Generates a `pre-commit` config file from a preset

**Usage:** `sketch pre-commit <PRESET> [OUTPUT]`

###### **Arguments:**

* `<PRESET>` — The preset id
* `<OUTPUT>` — The output path of the created file [default: `.pre-commit-config.yaml`]



## `sketch ts`

Executes typescript-specific commands

**Usage:** `sketch ts [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `pnpm-workspace` — Generates a `pnpm-workspace.yaml` file from a preset
* `package-json` — Generates a `package.json` file from a preset
* `ts-config` — Generates a `tsconfig.json` file from a preset
* `oxlint` — Generates a `.oxlintrc.json` file from a preset
* `monorepo` — Generates a new typescript monorepo
* `package` — Generates a new typescript package
* `barrel` — Creates a barrel file

###### **Options:**

* `--package-manager <NAME>` — The package manager being used. [default: pnpm]

  Possible values: `pnpm`, `npm`, `deno`, `bun`, `yarn`

* `--no-default-deps` — Do not add default dependencies to new `package.json` files (typescript and oxlint, plus vitest if enabled)
* `--version-range <KIND>` — The kind of version range to use for dependencies that are fetched automatically. [default: minor]

  Possible values: `patch`, `minor`, `exact`

* `--catalog` — Uses the pnpm catalog for default dependencies, and automatically adds dependencies marked with `catalog:` to the catalog, if they are missing
* `--no-convert-latest` — Do not convert dependencies marked as `latest` to a version range



## `sketch ts pnpm-workspace`

Generates a `pnpm-workspace.yaml` file from a preset

**Usage:** `sketch ts pnpm-workspace <PRESET> [OUTPUT]`

###### **Arguments:**

* `<PRESET>` — The preset id
* `<OUTPUT>` — The output path of the generated file [default: `pnpm-workspace.yaml`]



## `sketch ts package-json`

Generates a `package.json` file from a preset

**Usage:** `sketch ts package-json <PRESET> [OUTPUT]`

###### **Arguments:**

* `<PRESET>` — The preset id
* `<OUTPUT>` — The output path of the generated file [default: `package.json`]



## `sketch ts ts-config`

Generates a `tsconfig.json` file from a preset

**Usage:** `sketch ts ts-config <PRESET> [OUTPUT]`

###### **Arguments:**

* `<PRESET>` — The preset id
* `<OUTPUT>` — The output path of the generated file [default: `tsconfig.json`]



## `sketch ts oxlint`

Generates a `.oxlintrc.json` file from a preset

**Usage:** `sketch ts oxlint <PRESET> [OUTPUT]`

###### **Arguments:**

* `<PRESET>` — The preset id
* `<OUTPUT>` — The output path of the generated file [default: `.oxlintrc.json`]



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
* `--oxlint <ID>` — The oxlint preset to use. It can be set to `default` to use the default preset
* `-i`, `--install` — Installs the dependencies with the chosen package manager
* `-w`, `--with-template <id=TEMPLATE_ID,output=PATH>` — One or many individual templates to render in the new package's directory
* `-t`, `--with-templ-preset <ID>` — One or many templating presets to render in the new package's directory



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
* `-w`, `--with-template <id=TEMPLATE_ID,output=PATH>` — One or many individual templates to render in the new package's directory
* `-t`, `--with-templ-preset <ID>` — One or many templating presets to render in the new package's directory
* `--vitest <ID>` — The vitest preset to use. It can be set to `default` to use the default preset
* `-n`, `--name <NAME>` — The name of the new package. It defaults to the name of its directory
* `--ts-config <id=ID,output=PATH>` — One or many tsconfig presets (with their output path) to use for this package (uses defaults if not provided)
* `--package-json <ID>` — The package.json preset ID to use (uses defaults if not provided)



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



## `sketch repo`

Creates a new git repo from a preset

**Usage:** `sketch repo [OPTIONS] [DIR]`

###### **Arguments:**

* `<DIR>` — The directory where the new repo should be generated. [default: `.`]

###### **Options:**

* `-p`, `--preset <PRESET>` — Selects a git preset from a configuration file
* `--no-pre-commit` — Do not generate a pre-commit config
* `--pre-commit <PRE_COMMIT>` — Selects a pre-commit preset
* `--gitignore <GITIGNORE>` — Selects a gitignore preset
* `-w`, `--with-template <id=TEMPLATE_ID,output=PATH>` — One or many individual templates to render in the new repo
* `-t`, `--with-templ-preset <ID>` — One or many templating presets to render in the new repo
* `-r`, `--remote <REMOTE>` — The link of the git remote to use for the new repo



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

* `--cwd <CWD>` — The cwd for the command to execute [default: `.`]
* `-f`, `--file <FILE>` — The path to the command's template file, as an absolute path or relative to the cwd
* `-t`, `--template <TEMPLATE>` — The id of the template to use (a name for config-defined templates, or a relative path to a file from `templates_dir`)



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
