# 🖌️ Sketch

Sketch is a tool that allows you to define and organize various kinds of templates and presets in order to reduce the time spent creating boilerplate files or code, while making your workspace more organized and easily reproducible.

**Table of contents**
- [Improving Organization And Productivity](#improving-organization-and-productivity)
- [Enhanced Templating Toolset](#enhanced-templating-toolset)
- [Installation](#installation)
- [Documentation](#documentation)

# Improving Organization And Productivity

Templating is an awesome tool. While it is mostly used for rendering content in a web framework, it can also be used to do much more than that. 

For instance, it can be used to define structures for files or even entire projects. These structures can be standardized enough to provide structure, clarity, familiarity and reproducibility, but also flexible enough as to be able to accept customized or even dynamically generated parameters.

Taking advantage of these features helps in dealing with some common pain points in software development:

## 1. **Boilerplate Generation**

#### The Problem

Creating boilerplate for files or entire projects is an unrewarding and time consuming process. It causes unwanted cognitive load and kills productive momentum.

#### The Solution

When using presets, a single command can be used to generate a file or an entire project structure. It's orders of magnitude faster than doing it manually, while still being less complex than creating scripts and more flexible than copy-pasting.

## 2. Structure And Reproducibility

#### The Problem

There are many cases in which you need to have a specific configuration file in multiple places: a Docker Compose service, a script, a configuration file for a linter, or anything else.

When you need to update this configuration file, you then need to update it in every other place where you are using it.
If this is done manually, each copy may look slightly different than the other (even if the content is the same), maybe for something as simple as ordering fields in a different way from one file to another.

While this may seem trivial, it causes mental friction and it can compound to outright confusion because each time your brain has to process a slightly different structure.

#### The Solution

If you use a preset, then every file generated with that preset will look the exact same. It removes the cognitive penalty caused by ever-slightly-different configuration files, and lets you focus on more important areas of your work.

## 3. Maintainability

#### The Problem

Like in the example above, if you need to use a specific configuration in many projects, that means that every change must be applied to every instance of the same configuration. This is a tedious and error-prone process, that eventually leads to projects being misconfigured and requiring manual review to bring them up to date with the new settings.

#### The Solution

With a preset, you can modify the preset itself and then update each instance programmatically. Even if the new changes are very specific and need manual review rather than being immediately applicable everywhere, it's still easier to maintain 1 preset (or even 2 or 10, for that matter) than maintaining 100s of individual files.

# Presets

The concept of `preset` is central to Sketch's design and configuration. The basic concept about presets is that you should be able to define specific configurations that you find yourself using very often, and then use them to generate the files or even as a base for a more detailed preset.

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

Some presets also have extra features that make it easier to construct and use them. You can consult the [documentation](https://rick-phoenix.github.io/sketch/presets/summary.html) for more in-depth examples of various use cases.

## Type-safe Presets

Some presets, such as those that belong to some of the most widely used configuration files such as Rust's `Cargo.toml`, or Typescript's `tsconfig.json`, are **fully typed** and documented (as part of the JSON schema for Sketch's configuration file), so that defining them comes with all of the benefits of IDE integration such as type safety and autocompletion.

## Extensible Presets

Some presets are extensible, which means that you can define a base preset containing some common data (such as a list of networks or volumes in a `compose.yaml` file or a list of dependencies in a `package.json` file), and then create a preset that extends that base preset, so that you do not need to repeat inputs for all presets that share common settings.

## Aggregator Presets

Some presets can aggregate or use other presets. For example, a git `repo` preset can contain multiple templating presets (which will be generated in the root of the new repo), select a specific `gitignore` or `pre-commit` preset, along with some templates to render and execute as commands in the form of pre or post `hooks`.

# Enhanced Templating Toolset

Sketch uses [Tera](https://keats.github.io/tera/docs/) to render custom templates, which comes with its own rich feature set, and on top of that, it provides extra features, such as:

- Special variables that extract commonly used values such as the user directory or the host's operating system 
- Functions that perform actions such as:
    - Making a glob search in a directory
    - Generating a uuid 
    - Transforming a relative path into an absolute path 
    - Extracting capture groups from a regex 

...and many other more, which you can find in more detail in the documentation website.

All of these are available directly within templates, which greatly extends their flexibility and the variety of scenarios in which they can be applied.

# Installation

Sketch can be installed in two ways:

1. By downloading a pre-built binary from the [github repository](https://github.com/Rick-Phoenix/sketch)
2. Via `cargo` (`cargo install sketch-it`)

# Documentation

You can find out more about Sketch in the [dedicated website](https://rick-phoenix.github.io/sketch/).

The homepage will always show the docs for the latest version. Each minor version will have its own dedicated subroute under `/versions`.
