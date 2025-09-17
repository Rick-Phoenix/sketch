# Executing Templates

With the `sketch exec` command, you can render a template and then execute it as a shell command.

To do this, you can use a regular file, a template with an id (either defined in the config file or inside the `templates_dir`), or even one defined within the command itself.

### From Template

Let's say that we have a file named `cmd_template.j2` inside our `templates_dir`, and it looks like this:

```txt
{{#include ../../sketch/tests/commands_tests/cmd_template.j2}}
```

>ℹ️ You do not have to use `.j2` as an extension for the template files. Any extension can be used.

We can refer to it by its id:

>`{{#include ../../sketch/tests/output/commands_tests/commands/exec_from_template_cmd}}`

This will create a file containing

```txt
{{#include ../../sketch/tests/output/commands_tests/output_from_templates_dir.txt}}
```

### From File

We can also use a file that is not inside `templates_dir` by using the `-f` flag and providing the path to said file.  Relative paths will be resolved starting from the cwd.

So let's say that we have this template file:

```txt
{{#include ../../sketch/tests/commands_tests/cmd_from_file.j2}}
```

We launch the command like this:

>`{{#include ../../sketch/tests/output/commands_tests/commands/cmd_from_file}}`

To create a file containing:

```txt
{{#include ../../sketch/tests/output/commands_tests/output_from_file.txt}}
```

### From Literal Template

Templates can also be defined directly as part of the command.

>`{{#include ../../sketch/tests/output/commands_tests/commands/exec_literal_cmd}}`

This creates a file containing:

```txt
{{#include ../../sketch/tests/output/commands_tests/command_output.txt}}
```
