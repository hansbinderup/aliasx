# aliasx-cli — Usage & Commands

You can always run `aliasx --help` for full self documentation.

```bash
$ aliasx --help
Alias e(x)tended CLI

Examples:
  aliasx                    (default to fzf)
  aliasx ls                 (list aliases)
  aliasx fzf -q query       (fzf with query as search)
  aliasx --index 0          (execute alias 0)
  aliasx -n                 (fzf native aliases (.bashrc, .zshrc etc))
  aliasx -n -v -i 0 ls      (list first native aliases verbosely)
  aliasx -f local ls        (filter local aliases only)
  aliasx -v validate        (validates all configs verbosely)


Usage: aliasx [OPTIONS] [COMMAND]

Commands:
  ls        list all aliases (list)
  fzf       use fuzzy finder (f)
  validate  run validation on configs files
  help      Print this message or the help of the given subcommand(s)

Options:
  -i, --index <INDEX>
          the index of alias to handle

  -n, --native
          only apply to native aliases

  -v, --verbose
          verbose output

  -f, --filter <FILTER>
          filter which tasks to include
          
          [default: all]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

Quick start

- Run `aliasx` to open the interactive fuzzy finder and pick a task or alias.
- Use `aliasx ls` to print a list of available tasks and aliases.
- Run `aliasx validate` to validate your current config.
- Run `aliasx --index N` to query only that index into the command.

### Common flags

- `-n, --native`  : show only native shell aliases
- `-f, --filter <local|global|all>` : restrict scope
- `-v, --verbose` : verbose output

### fzf command flags:
- `-q, --query <text>` : initial search query for the `fzf` / `f` subcommand (use as `aliasx fzf --query <text>` or `aliasx f --query <text>`)

## Examples

- Interactive search: aliasx
- Search with query: aliasx f --query "test"
- Show local tasks only: aliasx -f local ls
- Run item by index: aliasx --index 3

---

Navigation: ← [Previous: Scopes](05-scope.md) | [Next: TUI & fuzzy UI](07-tui.md) →
