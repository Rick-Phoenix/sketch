# Executing Templates

With the `sketch exec` command, you can render a template and then execute it as a shell command.

To do this, you can use a regular file, a template with an id (either defined in the config file or inside the `templates_dir`), or even one defined within the command itself.

### From Template

File:

```txt
{{#include ../../sketch/tests/commands_tests/cmd_template.j2}}
```

Command:

>`{{#include ../../sketch/tests/output/commands_tests/commands/exec_from_template_cmd}}`

>ℹ️ You do not have to use `.j2` as an extension for the template files. Any extension can be used.

Output:

```txt
{{#include ../../sketch/tests/output/commands_tests/output_from_templates_dir.txt}}
```

### From File

File:

```txt
{{#include ../../sketch/tests/commands_tests/cmd_from_file.j2}}
```

Command:

>`{{#include ../../sketch/tests/output/commands_tests/commands/cmd_from_file}}`

Output:

```txt
{{#include ../../sketch/tests/output/commands_tests/output_from_file.txt}}
```

### From Literal Template

>`{{#include ../../sketch/tests/output/commands_tests/commands/exec_literal_cmd}}`

Output:

```txt
{{#include ../../sketch/tests/output/commands_tests/command_output.txt}}
```
