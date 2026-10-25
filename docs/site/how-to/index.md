# How-to guides

Task recipes. Each assumes sibling checkouts of this repo and of the platform
repos it patches.

## Add an operator

Add grammar rows for the token with its left/right binding power; a precedence change is a data edit, not a code edit.

## Consume the parse result

Read the root as the arena's **last** node — this engine appends children before their parents.

```
let ast = engine.parse(&request)?;
let root = ast.last().expect("parser always emits a root");
```
