# Configuration — Mappings

Mappings are yet another powerful feature of aliasx. This feature provides flexibility and creativity to be added to your tasks.

Mapping can be anything from build types, folder names, ssh host names (based on eg. device types) and much more!

Mappings live under the top-level `mappings` list. 
A mapping links a concrete input value to a human-friendly string (or other output) and is referenced from task commands using `${mapping:<id>}`.

## YAML example

This example is written in yaml but the same config can be used for json (with json syntax).

```yaml
version: "1.0.0"
tasks:
  - label: "Cargo build package"
    command: "cargo build --package ${input:packages} --target-dir ${mapping:build-folder}"
    description: "Build a package"
  - label: "Clean build folder"
    command: "rm -rf ${mapping:build-folder} || true && echo 'cleaned ${mapping:build-folder}'"

inputs:
  - id: packages
    description: "Packages in aliasx"
    options:
      - "aliasx-core"
      - "aliasx-tui"
      - "aliasx-cli"

mappings:
  - id: build-folder
    input: packages
    description: "Build folder for packages"
    options:
      "aliasx-core": ".build-core"
      "aliasx-tui": ".build-tui"
      "aliasx-cli": ".build-cli"
```

- `id` (required): the id that will be used to reference the mapping using: `${mapping:<id>}`
- `input` (required): the input that the mapping is implementing
- `description` (optional): friendly text shown in the UI
- `options` (required): the mapped values

Key points

- Each mapping item has `id`, `input` (the input id to map from), and `options` (a key -> value map).
- Mapping keys should match possible values from the referenced input (types: string/number/boolean are supported as YAML keys).
- Use `${mapping:<id>}` in `command` strings to substitute the mapped value.
- You don't need to manually provide the input - a mapping will also prompt for the input if not already provided.

Behavior notes

- If the user selects or enters a value that has no mapping, the substitution may be empty or reported as missing based on runtime behavior.
- Prefer keeping mapping keys aligned with the input `options` to avoid surprises.
- Mappings can be debugged using the [validator](08-validation.md).

## Demo

<p align="center">
  <img src="/docs/gifs/mappings-example.gif" width="700">
</p>

---

Navigation: ← [Previous: Inputs](03-inputs.md) | [Next: Scope](05-scope.md) →
