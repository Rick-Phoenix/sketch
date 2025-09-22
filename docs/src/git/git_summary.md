# Repo Setup

You can also use the `init` command to create a new git repo. This command will:

1. Create a new git repo in the specified `out_dir`.
2. If a `--remote` is provided, it will also add that remote as the origin/main for the repo.
3. Unless `pre-commit` is disabled, it will generate a new .pre-commit-config.yaml file in the root, with the repos specified in the config file (if there are any, otherwise it will just add the gitleaks repo). It will then run `pre-commit install` to install the given hooks.
