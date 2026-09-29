# Library

The interfaces the library keeps for its own binary and tests. None is published outside this project.

## Includes

- `src/**/*.rs`

## `Value`

What a field holds: text, a number, or a date.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum Value {}
```

## `Record`

One file as it moves through a pipeline.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct Record {}
```

## `Record::new`

Create the record of a file from its path relative to the source and its modification time.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Record {
    pub fn new(path: &Path, mtime: DateTime<Utc>) -> Record {}
}
```

## `Record::with_vars`

Add variables as fields, never replacing a built-in field.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Record {
    pub fn with_vars(self, vars: BTreeMap<String, Value>) -> Record {}
}
```

## `Record::plan`

The path the file is planned to be moved to, relative to the source.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Record {
    pub fn plan(&self) -> &Path {}
}
```

## `Record::set_plan`

Replace the plan.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Record {
    pub fn set_plan(&mut self, plan: PathBuf) {}
}
```

## `Record::field`

Read a field by name.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Record {
    pub fn field(&self, name: &str) -> Option<&Value> {}
}
```

## `Record::set_field`

Write a field, replacing any earlier value.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Record {
    pub fn set_field(&mut self, name: &str, value: Value) {}
}
```

## `Template`

A name written out of a record's fields.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct Template {}
```

## `Template::parse`

Read a template, refusing one that is malformed.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Template {
    pub fn parse(source: &str) -> Result<Template, TemplateError> {}
}
```

## `Template::render`

Write the template out for one record, refusing when a required field is missing or misused.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Template {
    pub fn render(&self, record: &Record) -> Result<String, RenderError> {}
}
```

## `TemplateError`

Why a template was refused when it was read.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum TemplateError {}
```

## `RenderError`

Why a template was refused for one record.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum RenderError {}
```

## `Stage`

One small function of a pipeline, declared with its parameters.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum Stage {}
```

## `Stage::declare`

Read one stage from its declaration, refusing a mistaken one.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Stage {
    pub fn declare(value: &toml::Value) -> Result<Stage, DeclareError> {}
}
```

## `DeclareError`

Why one stage declaration was refused.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum DeclareError {}
```

## `Pipeline`

The ordered stages a watch applies to the files it claims.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct Pipeline {}
```

## `Pipeline::declare`

Read a list of stage declarations and check their order.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Pipeline {
    pub fn declare(values: &[toml::Value]) -> Result<Pipeline, PipelineError> {}
}
```

## `Pipeline::from_toml`

Read the `stages` list of a TOML document.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Pipeline {
    pub fn from_toml(source: &str) -> Result<Pipeline, PipelineError> {}
}
```

## `Pipeline::stages`

The declared stages in the order they were written.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Pipeline {
    pub fn stages(&self) -> &[Stage] {}
}
```

## `Pipeline::has_effect`

Whether the pipeline ends in a stage that touches the filesystem.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Pipeline {
    pub fn has_effect(&self) -> bool {}
}
```

## `PipelineError`

Why a pipeline was refused.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum PipelineError {}
```

## `Outcome`

What a stage does with one record: pass it on, exclude it, or refuse it.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum Outcome {}
```

## `Rejection`

Why a stage refused a record, and which stage it was.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct Rejection {}
```

## `Stage::apply`

Run the stage on one record.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Stage {
    pub fn apply(&self, record: Record) -> Outcome {}
}
```

## `Pipeline::plan`

Run the stages that only rewrite the plan on one record, up to the first stage that touches the filesystem.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Pipeline {
    pub fn plan(&self, record: Record) -> Outcome {}
}
```
