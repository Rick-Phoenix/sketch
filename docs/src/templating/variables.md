# Special Variables

Sketch provides some special variables to get access to commonly used values such as the home directory. 

All of the following variables are available in templates, prefixed with `sketch_` (i.e. `sketch_os` and so on).

- `cwd`
- `tmp_dir` (from `env::temp_dir`)
- `home` (from `env::home_dir`)
- `os` (from `CARGO_CFG_TARGET_OS` or env `OS`)
- `os_family` (from `CARGO_CFG_TARGET_FAMILY`)
- `arch` (from `CARGO_CFG_TARGET_ARCH` or env `HOSTTYPE`)
- `user` (env `USER`)
- `hostname` (env `HOSTNAME`)
- `xdg_config` (env `XDG_CONFIG_HOME`)
- `xdg_data` (env `XDG_DATA_HOME`)
- `xdg_state` (env `XDG_STATE_HOME`)
- `xdg_cache` (env `XDG_CACHE_HOME`)
- `is_unix` (from `cfg!(unix)`)
- `is_linux` (from `cfg!(target_os = "linux")`)
- `is_macos` (from `cfg!(target_os = "macos")`)
- `is_wsl` (checks `/proc/sys/kernel/osrelease`)
- `is_windows` (from `cfg!(windows)`)
