# 🖌️ Sketch

Sketch is a tool designed to **reduce boilerplate**, **increase productivity**, and facilitate the creation and management of **structured** and **easily reproducible** workspaces.

**Table of contents**
- [Goals And Design Philosophy](#goals-and-design-philosophy)
    - [Presets](#presets)
    - [Why Use Presets](#why-use-presets)
    - [Supported Presets](#supported-presets)
    - [Enhanced Templating Toolset](#enhanced-templating-toolset)
- [Installation](#installation)
- [Documentation](#documentation)

## Goals And Design Philosophy

Templating is mostly used to render webpages, but it can also be used to generate files used in day-to-day development, and even more, it can be used to define structures for entire projects in a way that can be **standardized** enough to provide structure and clarity, but also **flexible** enough as to make use of customized or even dynamically generated parameters.

While templating is a powerful technology on its own, it needs a frontend, a *framework* of sort, to make use of its flexible toolset, enhance it with with extra features, and to wrap it with an **ergonomic** API explicitely designed with the intent of simplifying most if not all activities related to creating and managing files and projects.

This is that Sketch aims to be, and it all starts with the concept of `preset`. 

### Presets

The basic concept about presets is that you should be able to define specific configurations that you find yourself using very often, and then use them to generate the files that you need or as the base for a more detailed preset.

Presets come in different forms and shapes and with different features.

Some presets are used as a way to **aggregate** other presets or templates into a single structure. For example, a git `repo` preset can use a `gitignore` or a `pre-commit` preset, it can aggregate customized templates templates to render inside the new repo, and it can also contain a list of commands (which also benefit from templating features) to execute before or after generation.

Other presets, such as those that belong to some of the most widely used configuration files such as Rust's `Cargo.toml`, or Typescript's `tsconfig.json`, are **fully typed** and documented (as part of the JSON schema for Sketch's configuration file), so that defining them comes with all of the benefits of IDE integration such as **type safety** and **autocompletion**.

Other presets are **extensible**, which means that you can define a *base* preset containing some common data (such as a list of networks or volumes in a `compose.yaml` file or a list of dependencies in a `package.json` file), and then create a preset that **extends** that base preset, so that you do not need to repeat inputs for all presets that share common settings.

### Why Use Presets

Using presets helps in dealing with some of the most common pain points in software development:

#### 1. **Boilerplate Generation**

##### The Problem

Writing boilerplate is an unrewarding and time consuming process. It causes unwanted cognitive load and kills productive momentum.

##### The Solution

When using presets, a single command can be used to generate a file or an entire project structure. It's orders of magnitude faster than doing it manually, while still being less complex than creating an ad-hoc script and more flexible than mere copy-pasting.

#### 2. Structure And Reproducibility

##### The Problem

There are many cases in which you need to use a configuration file in multiple projects, such as a `compose.yaml` file, a github workflow, a setup file for a formatter or action runner, and so on. And chances are, you want to use this exact configuration in many if not all of your projects.

When you need to update this configuration file, you then need to update it in every other place where you are using it.
If this is done manually, each copy may look slightly different than the other even for something as simple as ordering fields in a different way from one file to another.

While this may seem trivial, it causes mental friction and it can compound to outright confusion because each time your brain has to process a slightly different structure.

##### The Solution

If you use a preset, then every file generated with that preset will look the exact same. It removes the cognitive penalty caused by ever-slightly-different configuration files, and lets you focus on more important areas of your work.

And with extensible presets, you can keep a core preset where you define all of your shared configurations, and then extend it with more customized settings based on your project's needs.

#### 3. Maintainability

##### The Problem

When you need to use a specific configuration in many different projects, that means that every change must be applied to every instance of the same configuration. This is a tedious and error-prone process, that eventually leads to projects being misconfigured and requiring manual review to bring them up to date with the new settings.

##### The Solution

With a preset, you can modify the preset itself and then update each instance programmatically (either by reusing or extending another preset). Even if the new changes are very specific and need manual review rather than being immediately applicable everywhere, it's still easier to maintain 1 preset (or even 2 or 10, for that matter) than maintaining 100s of individual files.

### Supported Presets

The list of supported presets includes:

- Git
    - `.gitignore` (extensible)
    - `.pre-commit-config.yaml` (extensible)
    - [Github workflow](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax) (extensible)
    - [Github workflow job](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobs) (extensible)

- Docker
    - Docker Compose file (extensible)
    - Docker Compose service (extensible)

- Rust
    - `Cargo.toml` (extensible)

- Typescript
    - `pnpm-workspace.yaml` (extensible)
    - `package.json` (extensible, with extra features)
    - `tsconfig.json` (extensible, with merging of values for the `references`, `include`, `exclude` and `files` fields)
    - `.oxlintrc.json` (extensible)
    - `vitest` (not a full configuration for `vitest.config.ts`, but a basic testing setup)

### Enhanced Templating Toolset

While presets are designed to cover some of the most common use cases, `custom templates` can also be used to cover all sorts of scenarios. Custom templates can then be aggregated and used by other presets.

Sketch uses [Tera](https://keats.github.io/tera/docs/) as the templating engine to render custom templates, enhanced with a variety of extra features, such as:

- **Special variables** that extract commonly used values such as the user directory or the host's operating system 
- **Functions** that perform actions such as:
    - Making a glob search in a directory
    - Generating a uuid 
    - Transforming a relative path into an absolute path 
    - Extracting capture groups from a regex 

...and many other more, which you can find out about in more detail in the [dedicated website](https://rick-phoenix.github.io/sketch/).

## Installation

Sketch can be installed in two ways:

1. By downloading a pre-built binary from the [github repository](https://github.com/Rick-Phoenix/sketch)
2. Via `cargo` (`cargo install sketch-it`)

## Documentation

You can find out more about Sketch in the [dedicated website](https://rick-phoenix.github.io/sketch/).

The homepage will always show the docs for the latest version. Each minor version will have its own dedicated subroute under `/versions`.
