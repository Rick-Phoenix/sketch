# 🖌️ Sketch

>`Templating, made portable`

Templating is awesome. While it is mostly used for websites, it can be a powerful tool for developers as it can allow us to automate some repetitive tasks like setting up projects which share the same basic structure. 

There are very powerful templating engines, which however require a library to work with them, which makes them not exactly portable. 

And there are pieces of software like `ansible`, which provide better portability and great templating capabilities but still require a significant amount of setup, which makes them slightly overkill for simpler tasks like generating a few files and directories.

This is why I made `sketch`. 

`sketch` is portable enough so that the most basic setups require zero configuration whatsoever, but can also sustain more complex setups like extensible, multi-format configuration files with overrides available via cli flags in most cases.

It makes it very easy to define a template for a file or for an entire project, and set it all up with just a single command.

And there's more: by leveraging the [Tera](https://keats.github.io/tera/docs/) rendering engine, you can take advantage of its built-in filters and functions which can allow you to populate your templates not only with static values but with dynamically calculated ones, such as the current time of the day, a manipulated string, an env variable, or the result of an arithmetic calculation. 

Every setup that used to require several repetitive steps can now be executed in just a single command.

And speaking of commands, you can use `sketch` to render the output to stdout (so that it can be piped to other commands), or even to execute it as a shell command directly, which expands its utility for even more creative and flexible use cases.

# Typescript Project Generation

`sketch` was initially conceived as a typescript-centric tool, because I needed a way to manage the chaotic nature of typescript projects in a structured, orderly and easily reproducible way, that would also give me enough flexibility to be able to customize how each project piece is defined and generated.

It contains special commands and tools to generate new typescript projects and packages, such as the ability to define reusable, extensible `package.json` and `tsconfig` presets, as well as commonly used configuration files such as those belonging to `oxlint` or `vitest`.

# Documentation

You can find the full documentation in the [dedicated website](https://rick-phoenix.github.io/sketch/).
