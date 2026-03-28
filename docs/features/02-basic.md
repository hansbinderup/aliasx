# Basics

This page will cover the basics of how to use aliasx

## Creating configuration files

Aliasx supports both json and yaml syntax. The json is only supported for the sake of being compatible with [vscode-tasks](https://code.visualstudio.com/docs/debugtest/tasks).

This repo already includes examples for [yaml](/.aliasx.yaml) and [json](/.vscode/tasks.json) syntax.

## The minimal configuration

Top-level file structure

.aliasx.yaml uses a `version` and `tasks` list. Example:

```yaml
version: "1.0.0"
tasks:
  - label: "Cargo build (release)"
    command: "cargo build --release"
    description: "Build the project"
  - label: "Cargo test"
    command: "cargo test"
    description: "Test the project"
```

- `version` (optional): is currently unused but is there to add support for vscode syntax and eventually to support (potential) breaking changes.
- `tasks` (required): the list of tasks you want to use with aliasx.

Minimal task fields

- `label` (required): short identifier shown in lists
- `command` (required): the shell command to run
- `description` (optional): friendly text shown in the UI

## Demo

<p align="center">
  <img src="/docs/gifs/basics-example.gif" width="700">
</p>

---

Navigation: ← [Previous: Installation](01-installation.md) | [Next: Inputs](03-inputs.md) →
