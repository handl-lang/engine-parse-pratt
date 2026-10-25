# Explanation

## Why this crate exists

Pratt parsing is compact and handles precedence without a rule per level;
driving it from `GrammarRow` data keeps precedence in the language definition,
where grammar readers can see it.

## Where it sits

This crate belongs to the `engine-*` family. Plugins, kits and hosts reach it
through the kit's operation slot, so the engine can be swapped without touching a
language definition.

See the [reference](/engine-parse-pratt/reference/) for the API and the
[how-to guides](/engine-parse-pratt/how-to/) for tasks.
