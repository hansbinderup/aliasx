# Configuration — Scopes: local, global, native

How aliasx loads tasks from different sources and how to control which source is used.

## Sources

- Local: 
    - `.aliasx.yaml` in the repository root — project-specific tasks
    - `.vscode/tasks.json` in the repository root — project-specific tasks
- Global: `~/.aliasx.yaml` — personal tasks across projects
- Native shell aliases: discovered by running `alias` in your shell

Precedence & filters

- tui: you can use `tab/⇧tab` to cycle through the scopes
- aliasx includes tasks from all sources by default. Use `-f` / `--filter` to restrict results:
  - local — only project-local tasks
  - global — only `~/.aliasx.yaml`
  - all — include local, global, and native aliases

Tips

- Prefer local tasks for repo-specific workflows
- Keep personal helpers in `~/.aliasx.yaml`

---

Navigation: ← [Previous: Mappings](04-mappings.md) | [Next: CLI](06-cli.md) →
