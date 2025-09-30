# 🖌️ Sketch

>`Templating made simple`

Templating is an awesome tool. While it is mostly used for rendering content in a web framework, it can also be used to do much more than that. 

For instance, it can be used to define structures for files or even entire projects. These structures can be standardized enough to provide structure, clarity, familiarity and reproducibility, but also flexible enough as to be able to accept customized or even dynamically generated parameters.

Taking advantage of these features comes with some serious benefits:

1. It drastically reduces the time and effort needed to perform the least rewarding aspects of development such as writing boilerplate code or setting up the file structure for a new project or tool

2. It makes project structures standardized and easily reproducible, which makes it much easier to navigate them or to introduce them to others

3. It reduces the time necessary to maintain these structures, as they can now be defined and modified only in one place and reused everywhere else

Sketch is a tool designed to draw on these strenghts, and to combine them with its own structure-oriented philosophy to provide a simple but powerful tool to simplify and automate many of the day-to-day development operations that have to do with creating files or setting up projects.

# Presets

The concept of `preset` is central to Sketch's design and configuration.

There are several preset categories, which come with a variety of features (the full updated list can be found in the documentation website):

- Templating
    - Templating presets (extensible)

- Docker
    - Docker Compose file (extensible)
    - Docker Compose service (extensible)

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

Some presets, such as those that belong to some of the most widely used configuration files such as Rust's `Cargo.toml`, or Typescript's `tsconfig.json`, are **fully typed** and documented (as part of the JSON schema for Sketch's configuration file), so that defining them comes with all of the benefits of IDE integration such as type safety and autocompletion.

## Extensible Presets

Some presets are extensible, which means that you can define a base preset containing some common data (such as a list of networks or volumes in a `compose.yaml` file or a list of dependencies in a `package.json` file), and then create a preset that extends that base preset, so that you do not need to repeat inputs for all presets that share common settings.

## Aggregator Presets

Some presets can aggregate or use other presets. For example, a git `repo` preset can contain multiple templating presets (which will be generated in the root of the new repo), and also select a specific `gitignore` or `pre-commit` preset.

# Enhanced Templating Toolset

Sketch uses [Tera](https://keats.github.io/tera/docs/) to render custom templates, which comes with its own rich feature set, and on top of that, it provides extra features, such as:

- Special variables that extract commonly used values such as the user directory or the host's operating system 
- Functions that perform actions such as:
    - Making a glob search in a directory
    - Generating a uuid 
    - Transforming a relative path into an absolute path 
    - Extracting capture groups from a regex 
    - (and many other more...)

All of these are available directly within templates, which greatly extends their flexibility and the variety of scenarios in which they can be applied.

# Installation

Sketch can be installed in two ways:

1. By downloading a pre-built binary from the [github repository](https://github.com/Rick-Phoenix/sketch)
2. Via `cargo` (`cargo install sketch-it`)

# Documentation

You can find out more about Sketch in the [dedicated website](https://rick-phoenix.github.io/sketch/).

The homepage will always show the docs for the latest version. Each minor version will have its own dedicated subroute (/v0.1, /v0.2 and so on...) 
