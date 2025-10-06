## [0.2.0] - 2025-10-06

### ⛰️  Features

- Added node20 to tsconfig
- Create matching output dirs in structured templating presets, even if empty
- Added common license files generation
- Including variables from extra files
- Added git repos as a source for templating presets
- Added pre and post hooks for repo or ts package generation

### 🐛 Bug Fixes

- Removed dangling extends in default tsconfig
- Fixed wrong plural in tsconfig prop
- Removed invalid root_dir directive in default tsconfig
- Fixed hooks context not being loaded

### 🚜 Refactor

- *(docker)* Skip serialization for external field if undefined
- *(cli)* Order commands semantically in help
- Making name optional in cargo.toml presets

### 📚 Documentation

- Added documentation for hooks
- Fixed issue with homepage routing
- Added documentation for remote templating presets

### ⚡ Performance

- Removed slow iteration of config dir

### 🧪 Testing

- Added tests for hooks
- Added tests for vars files and remote presets

### 📦 CI/CD

- Fixed erroneous command for changelog generation on new releases
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
