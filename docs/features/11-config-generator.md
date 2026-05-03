# Config Generator

You can use the config generator to create minimal example config, convert configs written in `json<->yaml` (both directions).

## Basic usage

The config generator is documented in the cli: 

```bash
create or convert existing configs

Usage: aliasx config-generator <COMMAND>

Commands:
  example-config  print a minimal example config
  json-to-yaml    convert existing json config to yaml
  yaml-to-json    convert existing yaml config to json
  help            Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

- `example-config` : will print an example config into the shell.
- `json-to-yaml`   : will print the converted format to the shell.
- `yaml-to-json`   : will print the converted format to the shell.

With all the commands you can eg:

```bash
# create a local .aliasx.yaml file
aliasx config-generator example-config > .aliasx.yaml
aliasx config-generator example-config -f json > .aliasx.json

# copy the config from yaml -> json and store in .aliasx.json
aliasx config-generator yaml-to-json .aliasx.yaml > .aliasx.json
```

## The minimal config

The minimal config created by `example-config` is a great starting point showing how a simple mapping can be made.

---

Navigation: ← [Previous: History](10-history.md) | [Next: Installation & Getting Started](01-installation.md) →
