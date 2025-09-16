# Extensible Configurations

Configuration files can extend one another by using the `extends` field:

```yaml
extends: ["other_config.yaml"]
```

Where the path being used is a relative path starting from the parent directory of the original config file.

The merging strategy works as follows:
- For conflicting values, such as opposite booleans, the previous value will be overridden.
- For non-conflicting values such as maps (for example, the global template vars map), the values will be merged.
