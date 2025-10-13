# Templating

Sketch uses the [Tera](https://keats.github.io/tera/docs/) templating engine to render custom templates. Every template, whether it's defined in a file, in the config file, or inlined, can take advantage of all of Tera's functionalities, like using filters, functions, defining and using macros, performing operations and importing other templates.

<div class="warning">
Since all autoescaping is disabled in templates, you should always only use templates that you made or trust.
</div>

## Context And Variables

Each template can be populated with `context` variables. These variables can be set at many different stages, with different degrees of priority (from lowest to highest):

- Global variables
- Cli-set variable files
- Preset context
- Single template context
- Cli-set variables


<div class="warning">
Variables defined with the <code>--set</code> flag must be formatted in valid json. This means that, for example, strings must be wrapped in escaped quotes.
</div>
