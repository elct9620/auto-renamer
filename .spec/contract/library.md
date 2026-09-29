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
