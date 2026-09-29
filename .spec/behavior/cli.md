# Cli

The command line says where the configuration is, and nothing more.

## Includes

- `tests/cli.rs`

## `CLI-001` The configuration is read from a default place

| Step | Statement |
| --- | --- |
| Given | no arguments |
| When | the arguments are read |
| Then | the configuration is `/etc/auto-renamer/config.toml` |

## `CLI-002` A path can be given

| Step | Statement |
| --- | --- |
| Given | the arguments `--config /tmp/a.toml` |
| When | the arguments are read |
| Then | the configuration is `/tmp/a.toml` |

## `CLI-003` A path can be given with an equals sign

| Step | Statement |
| --- | --- |
| Given | the arguments `--config=/tmp/a.toml` |
| When | the arguments are read |
| Then | the configuration is `/tmp/a.toml` |

## `CLI-004` A path is needed after the flag

| Step | Statement |
| --- | --- |
| Given | the arguments `--config` |
| When | the arguments are read |
| Then | they are refused |

## `CLI-005` An unknown argument is refused

| Step | Statement |
| --- | --- |
| Given | the arguments `--colour` |
| When | the arguments are read |
| Then | they are refused, naming `--colour` |

## `CLI-006` Help is asked for

| Step | Statement |
| --- | --- |
| Given | the arguments `--help` |
| When | the arguments are read |
| Then | the command is to show help |

## `CLI-007` The version is asked for

| Step | Statement |
| --- | --- |
| Given | the arguments `--version` |
| When | the arguments are read |
| Then | the command is to show the version |
