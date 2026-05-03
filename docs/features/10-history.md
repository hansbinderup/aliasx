# History

Sometimes tasks can have multiple inputs requiring multiple selections from the user. Maybe you just want to re-run the same command over and over again.
This is where history comes into play.

When you call `aliasx history` the fuzzy finder will open and display a list with your previous runs. The details panel is open by default containing some meta-data.

Examples:
```
aliasx history              -> open fuzzy finder with last history (with your resolved runs)
aliasx history -i 0         -> call the last history (history with index=0)
aliasx history -c/--clear   -> clear the entire history
```
---

Navigation: ← [Previous: Conditions](09-conditions.md) | [Next: Config Generator](11-config-generator.md) →
