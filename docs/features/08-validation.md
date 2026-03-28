# Validation

Aliasx is able to run a health check / validation on your current config(s).

The validator accepts all the parameters:
- `--index` : will only validate given task
- `--filter` : only validate within given scope
- `--verbose` : validate with verbose outputs

## Examples

All the examples below will use the following config:

```yaml
version: "1.0.0"
tasks:
  - label: "Cargo build package"
    command: "cargo build --package ${input:packages} --target-dir ${mapping:build-folder}"
    description: "Build a package"
  - label: "Clean build folder"
    command: "rm -rf ${mapping:build-folder} || true && echo 'cleaned ${mapping:build-folder}'"
  - label: "Invalid config"
    command: "echo '${input:missing-input}'"
  - label: "Valid config"
    command: "echo '${input:packages}'"

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
      "aliasx-cli": ".build-cli"
```

### Validate with default parameters

When no parameters are passed the default validation will look like this:

```bash
$ aliasx validate
════════════════════════════════════════════════════════════
  VALIDATION REPORT
════════════════════════════════════════════════════════════

✗ Cargo build package (1 issues)
    ✗ Mapping 'build-folder' doesn't define option for input 'aliasx-tui'
✗ Clean build folder (1 issues)
    ✗ Mapping 'build-folder' doesn't define option for input 'aliasx-tui'
✗ Invalid config (1 issues)
    ✗ Input 'missing-input' not defined
✓ Valid config

════════════════════════════════════════════════════════════
  SUMMARY
════════════════════════════════════════════════════════════
  ✓ 1 passed  ✗ 3 failed
  ⚠ 3 issues total
════════════════════════════════════════════════════════════

⚠ Some validations failed.
```

### Verbose validation

It can be helpful to add some more context to the validation. This can be done with the `-v`/`--verbose` flag:

```bash
$ aliasx -v validate
════════════════════════════════════════════════════════════
  VALIDATION REPORT
════════════════════════════════════════════════════════════

Cargo build package FAIL 1/3
  ✓ Input 'packages' defined
  ✓ Mapping 'build-folder' defined
  ✗ Mapping 'build-folder' doesn't define option for input 'aliasx-tui'
Clean build folder FAIL 1/2
  ✓ Mapping 'build-folder' defined
  ✗ Mapping 'build-folder' doesn't define option for input 'aliasx-tui'
Invalid config FAIL 1/1
  ✗ Input 'missing-input' not defined
Valid config PASS
  ✓ Input 'packages' defined

════════════════════════════════════════════════════════════
  SUMMARY
════════════════════════════════════════════════════════════
  ✓ 1 passed  ✗ 3 failed
  ⚠ 3 issues total
════════════════════════════════════════════════════════════

⚠ Some validations failed.
```

### Validate single task

If you want to focus on a single task you can also provide its index:

```bash
$ aliasx --verbose --index 0 validate
Cargo build package FAIL 1/3
  ✓ Input 'packages' defined
  ✓ Mapping 'build-folder' defined
  ✗ Mapping 'build-folder' doesn't define option for input 'aliasx-tui'
```

---

Navigation: ← [Previous: TUI](07-tui.md) | [Next: Conditions](09-conditions.md) →
