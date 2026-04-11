# Configuration - Conditions

`conditions` is an optional parameter to each task.
`conditions` allows you to enable/disable tasks based on given criteria:

- `paths` (optional) : if you're currently in a path matching any of the provided paths.
- `files` (optional) : if any of the given files are matched with files in the current directory.

## Examples

```yaml
version: "1.0.0"
tasks:
  - label: "Git status"
    command: "git status"
    description: "Get git status when .git dir exists or when in any 'repos/github' sub-folder"
    conditions:
      files:
        - ".git"
      paths:
        - "**/repos/**"
        - "/home/user/github/**"

  - label: "Cargo build"
    command: "cargo build"
    description: "Build when Cargo.toml file is present"
    conditions:
      files:
        - "Cargo.toml"

  - label: "Should not be shown!"
    command: "echo  'conditions should not be true!'"
    conditions:
      paths:
        - "something-invalid"
      files:
        - "FILE_THAT_DOES_NOT_EXIST"
```

Key points

- Both `paths` and `files` support globs/wildcards
- `files` can both be directories or regular files
- You can provide as many `paths` and `files` as you like
- If no `conditions` are provided then the task is always enabled
- Allows for general rules/tasks that are auto enabled/disabled for eg. repos, git projects and so on.

## Enabling and disabling

Conditions can be enabled/disabled from the cli with the following option:

```bash
Options:
  -c, --conditions <CONDITIONS>
          enable conditions

          [possible values: true, false]
```

Conditions are default enabled and will always be disabled for validation (see below why).

## Validation

The [validator](08-validation.md) will never mark a task as failed even if conditions are not met. The status will always be `passed`.
But it's still useful for debugging your conditions when running the validator in verbose mode (for example, with `aliasx -v validate` or `aliasx --verbose validate`), as it will explicitly state if a task was skipped or not, for example:

```bash
Should not be shown! PASS
  ⏭ Conditions are not met (skipped)
Git status PASS
  ✓ Conditions are met
```

Conditions will only be marked as failed if the provided options can not be treated as a glob.

---

Navigation: ← [Previous: Validation](08-validation.md) | [Next: History](10-history.md) →
