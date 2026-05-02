# aliasx-cli — Usage & Commands

You can always run `aliasx --help` for full self documentation.

```bash
$ aliasx --help
Alias e(x)tended CLI

Usage: aliasx [OPTIONS] [COMMAND]

Commands:
  ls        list all aliases (list)
  fzf       use fuzzy finder (f)
  validate  run validation on configs files
  history   use history instead of tasks
  run       run a task by id
  help      Print this message or the help of the given subcommand(s)

Options:
  -i, --index <INDEX>            the index of alias to handle
  -v, --verbose                  verbose output
  -f, --filter <FILTER>          filter which tasks to include [default: all]
  -n, --native                   only apply to native aliases
  -c, --conditions <CONDITIONS>  enable conditions [possible values: true, false]
  -h, --help                     Print help
  -V, --version                  Print version

```

Quick start

- Run `aliasx` to open the interactive fuzzy finder and pick a task or alias.
- Use `aliasx ls` to print a list of available tasks and aliases.
- Run `aliasx validate` to validate your current config.
- Run `aliasx --index N` to query only that index into the command.
- Run `aliasx run <task-id>` to run a task based on assigned id

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
- Show local tasks only: aliasx -f local ls
- Run item by index: aliasx --index 3

---

Navigation: ← [Previous: Scopes](05-scope.md) | [Next: TUI & fuzzy UI](07-tui.md) →
