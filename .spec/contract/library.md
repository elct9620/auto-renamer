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

Run the stage on one record, with what the batch and the target let it know.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Stage {
    pub fn apply(&self, record: Record, context: &mut Context) -> Outcome {}
}
```

## `Target`

What the target folder holds, as far as a stage needs to know.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub trait Target {}
```

## `Target::files_in`

The names of the files in a folder of the target, relative to its root.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub trait Target {
    fn files_in(&self, folder: &Path) -> Vec<String>;
}
```

## `Context`

What a run of stages shares: the target and the numbers already handed out in the batch.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct Context {}
```

## `Context::new`

Start a context over a target.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Context {
    pub fn new(target: &dyn Target) -> Context {}
}
```

## `Record::origin`

The path the record was made from, which no stage rewrites.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Record {
    pub fn origin(&self) -> &Path {}
}
```

## `Record::is_readable`

Whether the whole path is valid UTF-8, so that the name fields could be made from it.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Record {
    pub fn is_readable(&self) -> bool {}
}
```

## `Verdict`

What a batch made of one file: planned, excluded, unclaimed, or refused.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum Verdict {}
```

## `Judged`

One file of a batch with the verdict on it, and the place in the list of the pipeline that claimed it.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct Judged {}
```

## `plan_batch`

Plan a whole batch through the pipelines of a watch, in the order they are listed.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub fn plan_batch(pipelines: &[(String, Pipeline)], records: Vec<Record>, context: &mut Context) -> Vec<Judged> {}
```

## `Config`

What the operator configured: the watches, each with its pipelines and settings.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct Config {}
```

## `Config::parse`

Read the configuration, refusing everything that would leave a watch unable to run.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Config {
    pub fn parse(source: &str) -> Result<Config, ConfigError> {}
}
```

## `Config::watches`

The watches in the order of their names.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Config {
    pub fn watches(&self) -> &[Watch] {}
}
```

## `Watch`

One watched folder with where its files go and how they are grouped and named.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct Watch {}
```

## `Watch::pipelines`

The pipelines of the watch, in the order it lists them.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Watch {
    pub fn pipelines(&self) -> Vec<(String, Pipeline)> {}
}
```

## `Watch::under`

The watch as it is for the files of a folder that carries configurations of its own, from the farthest to the nearest.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Watch {
    pub fn under(&self, folders: &[FolderConfig]) -> Watch {}
}
```

## `FolderConfig`

What a folder says of the files in it: variables, pipelines and a batch limit.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct FolderConfig {}
```

## `FolderConfig::parse`

Read a folder configuration, refusing what a folder may not decide or what is too large.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl FolderConfig {
    pub fn parse(source: &str) -> Result<FolderConfig, ConfigError> {}
}
```

## `Unit`

How a file is assigned to a unit of the source.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum Unit {}
```

## `Unit::of`

The folder of the source, relative to it, that is the unit of a file.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Unit {
    pub fn of(&self, file: &Path) -> PathBuf {}
}
```

## `ConfigError`

Why a configuration was refused.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum ConfigError {}
```

## `Roots`

The source a file comes from and the target it goes to, which are one folder when files are renamed in place.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct Roots {}
```

## `Applied`

What a move did: moved the file, left it as it was, skipped it, or only said what it would do.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum Applied {}
```

## `SkipReason`

Why a file was left alone before it could be planned or moved: it is a link, not a regular file, gone, or has no modification time.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum SkipReason {}
```

## `EffectError`

Why a move was refused or failed, with the file as it was.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum EffectError {}
```

## `move_file`

Move the file of a planned record to its plan under the target, or say where it would go.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub fn move_file(stage: &Move, record: &Record, roots: &Roots, dry_run: bool) -> Result<Applied, EffectError> {}
```

## `cleanup_folders`

Remove the folders a moved file left empty, from its folder upward and only inside its unit.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub fn cleanup_folders(stage: &Cleanup, origin: &Path, unit: &Path, roots: &Roots) -> Result<Vec<PathBuf>, EffectError> {}
```

## `Done`

What one effect stage did.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum Done {}
```

## `EffectsRun`

What the effect stages of a pipeline did to one file, and the failure that stopped them if there was one.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct EffectsRun {}
```

## `apply_effects`

Run the effect stages of a pipeline in order on a planned file, stopping at the first failure.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub fn apply_effects(pipeline: &Pipeline, record: &Record, unit: &Path, roots: &Roots, dry_run: bool) -> EffectsRun {}
```

## `Event`

What happened to a file of the source, as far as the machine needs to know.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum Event {}
```

## `Ready`

A batch the machine hands over: the settled files of one unit.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct Ready {}
```

## `Machine`

Decides when the settled files of a unit are handed over as a batch, from what it is told and what time it is.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct Machine {}
```

## `Machine::new`

Start a machine for a watch, with its unit, window and maximum wait.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Machine {
    pub fn new(watch: &Watch) -> Machine {}
}
```

## `Machine::observe`

Tell the machine what happened to a file and when.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Machine {
    pub fn observe(&mut self, event: Event, now: SystemTime) {}
}
```

## `Machine::ready`

The batches that are ready at a time, each handed over once.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Machine {
    pub fn ready(&mut self, now: SystemTime) -> Vec<Ready> {}
}
```

## `Machine::next_deadline`

The earliest time at which asking again could change the answer, or none when nothing is waiting.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Machine {
    pub fn next_deadline(&self) -> Option<SystemTime> {}
}
```

## `Translated`

What a filesystem notification means for the watcher: an event for a file, or a folder to be scanned.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum Translated {}
```

## `translate`

Translate a filesystem notification into what happened to files of the source.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub fn translate(notification: &notify::Event, source: &Path, is_dir: impl Fn(&Path) -> bool) -> Vec<Translated> {}
```

## `scan_folder`

List the regular files below a folder as found-at-start events, without following links.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub fn scan_folder(source: &Path, folder: &Path) -> Vec<Event> {}
```

## `FsTarget`

The target folder as the filesystem holds it.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct FsTarget {}
```

## `FsTarget::new`

A target over the folder at the root.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl FsTarget {
    pub fn new(root: PathBuf) -> FsTarget {}
}
```

## `What`

What became of one file of a batch.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum What {}
```

## `Processed`

One file of a processed batch and what became of it.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct Processed {}
```

## `process_batch`

Plan a ready batch through the pipelines of its watch, with the folder configurations that apply, and run the effects on what was planned, refusing a file renamed in place too many times in a row.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub fn process_batch(watch: &Watch, unit: &Path, files: &[PathBuf], renames: &mut Renames) -> Vec<Processed> {}
```

## `Command`

What the command line asks for.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum Command {}
```

## `Options`

The settings of a run that come from the command line.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct Options {}
```

## `parse_args`

Read the command line, refusing what it does not know.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Command, String> {}
```

## `run`

Watch the sources of a configuration until told to stop, reading the configuration file again when it is rewritten or when told to.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub fn run(options: &Options, stop: &AtomicBool, reload: &AtomicBool) -> Result<(), RunError> {}
```

## `RunError`

Why the watcher could not start or stopped.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum RunError {}
```

## `Config::warnings`

What is legal but probably not what was meant, such as a pipeline that never moves anything.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Config {
    pub fn warnings(&self) -> Vec<String> {}
}
```

## `rewrites`

Whether a filesystem notification says a file was written, created or replaced, as opposed to only read.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub fn rewrites(notification: &notify::Event, file: &Path) -> bool {}
```

## `Renames`

How many times each file was renamed in place in a row, kept while the program runs.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct Renames {}
```

## `Renames::new`

A record with no renames in it.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Renames {
    pub fn new() -> Renames {}
}
```
