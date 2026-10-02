# Playground

The interfaces the playground page reaches through WebAssembly. The page holds all state; each call is answered from what it is handed.

## Includes

- `crates/wasm/src/**/*.rs`

## `Configuration`

Which configuration a text is: the global one or a folder configuration.

```rust
pub enum Configuration {}
```

## `check`

Read a configuration as the CLI reads it, answering why it is refused when it is.

```rust
pub fn check(kind: Configuration, text: &str) -> Result<(), String> {}
```

## `read`

The table a configuration text declares, for the page to edit.

```rust
pub fn read(text: &str) -> Result<Table, String> {}
```

## `render`

The configuration text of a table, without comments or the layout it was read with.

```rust
pub fn render(table: &Table) -> String {}
```

## `Entry`

One file or folder of the virtual tree, with its modification time and, for a file, its text.

```rust
pub struct Entry {}
```

## `Outcome`

What became of one file of a simulated batch.

```rust
pub struct Outcome {}
```

## `SimulatedStep`

What one file of a simulated batch held at one step of its planning: the pipeline, the stage by its place and name or none for the claim, its plan and fields while it went on, or why it stopped.

```rust
pub struct SimulatedStep {}
```

## `Simulation`

What a simulation reported and the virtual tree it left.

```rust
pub struct Simulation {}
```

## `simulate`

Run a watch of a configuration over a virtual tree: each unit of its source is processed as one batch, and the tree changes as the effects say.

```rust
pub fn simulate(config: &str, watch: &str, entries: Vec<Entry>) -> Result<Simulation, String> {}
```

## `stages`

How each stage is declared, for the page to offer the stages and make their forms.

```rust
pub fn stages() -> &'static [Declaration] {}
```
