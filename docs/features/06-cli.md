# aliasx-cli — Usage & Commands

You can always run `aliasx --help` for full self documentation.

```bash
$ aliasx --help
Alias e(x)tended CLI

Usage: aliasx [COMMAND]

Commands:
  run               run a task
  ls                list all aliases (list)
  fzf               use fuzzy finder (f)
  validate          run validation on configs files
  history           use history instead of tasks
  config-generator  create or convert existing configs
  help              Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Quick start

- Run `aliasx` to open the interactive fuzzy finder and pick a task or alias.
- Use `aliasx ls` to print a list of available tasks and aliases.
- Run `aliasx validate` to validate your current config.
- Run `aliasx run <task-id>` to run a task based on assigned id
- Run `aliasx run --index N` to run a task based on assigned index

Each command has a dedicated helper page. Call it with `--help` or `help`:

```bash
$ aliasx run --help
run a task

Usage: aliasx run [OPTIONS] [ID]

Arguments:
  [ID]  id of task to run

Options:
  -i, --index <INDEX>
  -v, --verbose
  -f, --filter <FILTER>          [default: all] [possible values: all, local, global]
  -n, --native
  -c, --conditions <CONDITIONS>  [possible values: true, false]
  -h, --help                     Print help

```

### Common flags

- `-n, --native`  : show only native shell aliases
- `-f, --filter <local|global|all>` : restrict scope
- `-v, --verbose` : verbose output
- `-c, --conditions <true|false>` : enable or disable conditions (default: true; for example, use `--conditions false` to disable)

### fzf command flags:
- `-q, --query <text>` : initial search query for the `fzf` / `f` subcommand (use as `aliasx fzf --query <text>` or `aliasx f --query <text>`)

## Examples

- Interactive search: aliasx
- Search with query: aliasx f --query "test"
- Show local tasks only: aliasx ls -f local
- Run item by index: aliasx run --index 3

---

Navigation: ← [Previous: Scopes](05-scope.md) | [Next: TUI & fuzzy UI](07-tui.md) →
