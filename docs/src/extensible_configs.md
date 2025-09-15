# Extensible Configurations

Configuration files can be extended like this:

```yaml
extends: ["other_config.yaml"]
```

The path used here is a relative path starting from the parent directory of the originating config file.

Conflicting values will be overridden by the extended config, except for default values. 

Non-conflicting values that are lists of elements such as the templating variables will instead be merged.
