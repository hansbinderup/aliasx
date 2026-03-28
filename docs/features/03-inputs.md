# Configuration — Inputs & variables

This page shows the exact .aliasx.yaml syntax for inputs and how to reference them from task commands.

Inputs are one of the key features that make aliasx such a powerful tool. Instead of writing eg. a bunch of aliases for all your build targets, ssh hosts eg. you can reference such with an input.

An input is declared in aliasx as `${input:<id>}` where:
`input -> this is an input` and `<id> -> the id of the declared input`.

## YAML example

This example is written in yaml but the same config can be used for json (with json syntax).


```yaml
version: "1.0.0"
tasks:
  - label: "Cargo build package"
    command: "cargo build --package ${input:packages} --${input:targets}"
    description: "Build a package"
  - label: "Cargo build"
    command: "cargo build --${input:targets}"
    description: "Build root"

inputs:
  - id: packages
    description: "Packages in aliasx"
    options:
      - "aliasx-core"
      - "aliasx-tui"
      - "aliasx-cli"
  - id: targets
    description: "Build targets"
    default: "all-targets"
    options:
      - "all-targets"
      - lib
      - bins
      - examples
      - tests
      - benches
```

- `id` (required): the `id` that will be used to reference with: `${input:<id>}`
- `description` (optional): friendly text shown in the UI
- `default` (optional): an optional default value - will default to first entry if not provided
- `options` (required): the available inputs that the user will be prompted to select

Key points

- Inputs live under the top-level `inputs` list.
- Each input item must have an `id`. Optional fields: `description`, `default`, `options`.
- In command strings reference inputs using `${input:<id>}` (e.g. `${input:targets}`).
- Aliasx will prompt for values at runtime
- You can specify as many inputs as you like
- Options are parsed as string so you can be creative here as well
- Inputs can be debugged using the [validator](08-validation.md).

Limitations:

- Inputs must be referenced from the same file - you can not use an input in a local file from a global file.

## Demo

<p align="center">
  <img src="/docs/gifs/inputs-example.gif" width="700">
</p>

---

Navigation: ← [Previous: Basics](02-basic.md) | [Next: Mappings](04-mappings.md) →
