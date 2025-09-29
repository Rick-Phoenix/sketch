# 🖌️ Sketch

>`Templating made simple`

Templating is an awesome tool. While it is mostly used for rendering content in a web framework, it can also be used to define reusable structures for files or entire projects which are standardized enough to provide structure, clarity, familiarity and reproducibility, but also flexible enough as to be able to accept customized or even dynamically generated parameters.

Taking advantage of these features comes with some serious benefits:

1. It drastically reduces the time and effort needed to perform the least rewarding aspects of development such as writing boilerplate code or setting up the file structure for a new project or tool

2. It makes project structures standardized and easily reproducible, which makes it much easier to navigate them or to introduce them to others

3. It reduces the time necessary to maintain these structures, as they can now be defined and modified only in one place and reused everywhere else

Sketch is a tool designed to draw on these strenghts, and to combine them with its own structure-oriented philosophy to provide a simple but powerful tool to simplify and automate many of the day-to-day development operations that have to do with file creation.

# Presets

The concept of `preset` is central to Sketch's design and configuration.

`Templating` presets are the general-purpose vehicle that can be used to aggregate individual templates to create a structural file tree layout to generate in the output directory. But there are many other preset categories, which come with a variety of features.

As of now, these presets are available (the full updated list can be found in the documentation website):

- Templating
    - Templating presets (extensible)

- Docker
    - Docker Compose file (extensible)

- Git
    - Git repo
    - `.gitignore` (extensible)
    - `.pre-commit-config.yaml` (extensible)

- Rust
    - `Cargo.toml` (extensible)

- Typescript
    - Typescript package
    - `pnpm-workspace.yaml` (extensible)
    - `package.json` (extensible, with extra features)
    - `tsconfig.json` (extensible, with merging of values for the `references`, `include`, `exclude` and `files` fields)
    - `.oxlintrc.json` (extensible)
    - `vitest` (not a full configuration for `vitest.config.ts`, but a basic testing setup)

## Type-safe Presets

Some presets, such as those that belong to some of the most widely used configuration files such as `.pre-commit-config.yaml`, or Typescript's `tsconfig.json` file, are **fully typed** and documented (as part of the JSON schema for Sketch's configuration file), so that defining them comes with all of the benefits of IDE integration such as type safety and autocompletion.

## Extensible Presets

Some presets, such as those for `compose.yaml`, `Cargo.toml` or `package.json` files, are extensible, which means that you can define a base preset containing some common data (such as a list of networks or volumes in a compose file or a list of dependencies in a package.json file), and then create a preset that extends that base preset, so that you do not need to repeat inputs for all presets who share common settings.

## Aggregator Presets

Some presets can aggregate or use other presets. For example, a git `repo` preset can contain multiple templating presets (which will be generated in the root of the new repo), and also select a specific `gitignore` or `pre-commit` preset.

# Enhanced Templating Toolset

Sketch uses the [Tera](https://keats.github.io/tera/docs/) to render custom templates, which comes with its own rich feature set, and on top of that, it provides extra features, such as special variables that extract commonly used values such as the user directory or the host's operating system, or functions that perform actions such as making a glob search in a directory, generating a uuid, transforming a relative path into an absolute path or extracting capture groups from a regex (and many other more...).

All of these are available directly within templates, which greatly extends their flexibility and the variety of scenarios in which they can be applied.

# Documentation

You can find out more about Sketch in the [dedicated website](https://rick-phoenix.github.io/sketch/).
