# TsConfig Presets

The configuration file can contain a variety of presets that can be used for generating Tsconfig files. 

These presets contain all of the fields of a normal `tsconfig.json` file, with the addition of an extra field called `extend-presets`, which allows you to extend a basic preset from another one.

## Example

Let's start with some basic starting point here:

Here we define two typical tsconfig settings to use for the packages in our workspace.

The `app` preset will not emit any .d.ts or .js files, whereas the `library` preset, which can be used for internal packages, will have `emitDeclarationOnly` set to `true`.
```yaml
{{#include ../../examples/typescript/tsconfig_presets.yaml:basic}}
```

Now let's say that we want to create another preset which starts from the `app` preset but adds the `tests` directory to `include`:

```yaml
{{#include ../../examples/typescript/tsconfig_presets.yaml:extended}}
```

Now we will set up a package that will generate 2 tsconfig files.
- A basic tsconfig.json file, which we will define literally, and it will have just the `files` field set to an empty array.
- One which will use the lib preset and extend it with an extra feature of our choice, and that will apply only to the files inside src. We will generate this in `tsconfig.src.json`.
- A separate config that will take care of typechecking the files inside the `tests` directory, without emitting files. We will create this inside `tsconfig.dev.json`.

```yaml
{{#include ../../examples/typescript/tsconfig_presets.yaml:package}}
```

After running the command

`{{#include ../../sketch/tests/output/ts_examples/commands/tsconfig_cmd}}`

tsconfig.json

```json
{{#include ../../sketch/tests/output/ts_examples/packages/tsconfig-example/tsconfig.json}}
```

tsconfig.src.json

```json
{{#include ../../sketch/tests/output/ts_examples/packages/tsconfig-example/tsconfig.src.json}}
```

tsconfig.dev.json

```json
{{#include ../../sketch/tests/output/ts_examples/packages/tsconfig-example/tsconfig.dev.json}}

