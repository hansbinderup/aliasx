# Configuration - Task options

`options` is an optional parameter to each task.
`options` allows you to manipulate the environment from which you execute the task.

- `env` (optional) : map of environment variables
- `cwd` (optional) : "current work directory" -> where to execute your task

## Examples

```yaml
version: "1.0.0"
tasks:
  - label: "Task with options (env and cwd)"
    command: "echo \"task with env: $ALIASX_ENV from $(pwd)\""
    options:
      env:
        ALIASX_ENV: "THIS_IS_A_TEST"
      cwd: "/var/log"
```

Key points

- `cwd` can be both relative or absolute paths
- `env` is only active during the execution - it won't leave your shell polluted
- You can provide as many `env`s as you like
- If no `cwd` is provided then the task will be executed in current work directory

---

Navigation: ← [Previous: Config Generator](11-config-generator.md)
