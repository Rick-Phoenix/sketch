## [0.1.0] - 2025-09-30

### ⛰️  Features

- Added config and docs for pnpm
- Added fully typed oxlint config
- Add oxlint presets
- Added fully types pre-commit config
- Added gitignore presets
- Added ability to add templates via command
- Added command for generating single config files from presets
- Added extra tera functions, filters and tests
- Added semver filter
- Added pnpm-workspace presets
- Added vitest presets
- Added case conversion filters
- Added structured templating presets
- Added path filters
- Added exclude patterns for structured template preset
- Added barrel command
- Added full docker compose configuration
- Added extensibility to templating presets
- Added cargo.toml presets
- Added service presets for docker compose

### 🚜 Refactor

- *(config)* Changed root_dir to out_dir for more clarity
- Changed all extension fields to `extends_presets` to avoid confusion

### 📚 Documentation

- Adjusted visibility for potential use as a lib
- Added example for filters usage
- Documented docker compose presets
- Added docker compose example
- Added example for barrel command

### 🧪 Testing

- Removed some npm api calls in tests to avoid getting rate limited

### 📦 CI/CD

- Added workflow for release
