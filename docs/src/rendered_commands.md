# Rendered Commands

Sketch makes it possible to create shell commands starting from templates, which can be defined in a file or provided via command.

### From literal template

`{{#include ../../sketch/tests/output/commands_tests/commands/exec_literal_cmd}}`

Output:

```txt
{{#include ../../sketch/tests/output/commands_tests/command_output.txt}}
```

### From file

File:

```txt
{{#include ../../sketch/tests/commands_tests/cmd_from_file.j2}}
```

Command:

`{{#include ../../sketch/tests/output/commands_tests/commands/cmd_from_file}}`

Output:

```txt
{{#include ../../sketch/tests/output/commands_tests/output_from_file.txt}}
```

### From template

You can also use a named template, which can be either a file inside `templates_dir` or a literal template defined in the config file.

File:

```txt
{{#include ../../sketch/tests/commands_tests/cmd_template.j2}}
```

Command:

`{{#include ../../sketch/tests/output/commands_tests/commands/exec_from_template_cmd}}`

Output:

```txt
{{#include ../../sketch/tests/output/commands_tests/output_from_templates_dir.txt}}
```
