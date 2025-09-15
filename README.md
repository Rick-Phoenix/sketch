# 🖌️ Sketch

Sketch is a cli tool that can be used to quickly set up all sorts of files or project structures, as well as launching commands that are rendered with the Tera templating engine.

You can think of it as a more portable, simpler version of ansible.

## Generate typescript projects and packages

Sketch comes with some presets and special commands for generating typescript workspaces and packages. <br/>
What makes it particularly useful for that use case is the fact that you can define some package presets, which you can then generate with a simple command.

The package presets are also able to define their package.json and tsconfig.json files by using extensible presets and definitions.

Let's look at some examples to get an idea of how things

## Modular, extensible presets

# Customized templating

# Rendered commands

# Lsp integration

You can find the json schema in the github repo. A schema will be released for each version, and the latest.json file will track the schema of the latest version. 
You can use this to get autocompletion for your yaml, json or toml configuration files.

## Usage as a library

Sketch can also be used as a rust library. Refer to the [docs.rs](https://docs.rs/sketch-it/latest/sketch_it/) page to learn more about usage as a library.
