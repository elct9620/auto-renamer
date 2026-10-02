# Library

The interfaces the library keeps for its own binary, its tests and the playground. None is published as a crate.

## Includes

- `src/**/*.rs`
- `crates/*/src/**/*.rs`

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

Create the record of a file from its path relative to the source and its modification time, refusing a path that is not valid UTF-8.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Record {
    pub fn new(path: &Path, mtime: DateTime<Utc>) -> Result<Record, NotUtf8> {}
}
```

## `Record::with_vars`

Add variables as fields, each an answer detection never overwrites, never replacing a built-in field.

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

## `Record::fields`

Every field of the record with its value, in the order of their names.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Record {
    pub fn fields(&self) -> impl Iterator<Item = (&str, &Value)> {}
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

## `Declared`

One stage as a pipeline declares it, with its parameters: the closed list of the stages there are.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum Declared {}
```

## `Declaration`

How one stage is declared, for a form to be made from: its name, whether a bare name declares it, the single value it may take, its named parameters, whether it takes fixed values named by the user, and an example it accepts.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct Declaration {}
```

## `Parameter`

One named parameter of a stage: its name, the kind of value it takes, whether the stage needs it, and the choices it is limited to.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct Parameter {}
```

## `ParameterKind`

The kind of value a parameter takes: text, a list of text, a whole number, or a yes or no.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum ParameterKind {}
```

## `Declared::declarations`

How each stage there is is declared, from the same list a declaration is read by.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Declared {
    pub fn declarations() -> &'static [Declaration] {}
}
```

## `Declared::read`

Read one stage from its declaration, refusing a mistaken one.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Declared {
    pub fn read(value: &toml::Value) -> Result<Declared, DeclareError> {}
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

## `Pipeline::stages`

The declared stages in the order they were written.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Pipeline {
    pub fn stages(&self) -> &[Declared] {}
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

## `Stop`

Why a stage refused a record, and which stage it was. A filter that turns a record away refuses it too, so every stop is a refusal. A stage answers with the record to go on with, or with a stop.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct Stop {}
```

## `Declared::apply`

Run the stage on one record, which is a batch of one, with what the target lets it know.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Declared {
    pub fn apply(&self, record: Record, context: &mut Context) -> Flow {}
}
```

## `Stage`

One small function from a batch to a batch, which every stage there is implements.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub trait Stage {}
```

## `Stage::name`

The name the stage is declared by.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub trait Stage {
    fn name(&self) -> &'static str;
}
```

## `Stage::run`

Run the stage over the batch its pipeline claimed, with what the target and the pipelines before let it know, and hand on the batch it made.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub trait Stage {
    fn run(&self, batch: Batch, context: &mut Context) -> Batch;
}
```

## `Batch`

The files a pipeline claimed in the order of their paths, each going on with its record or stopped by one of the stages.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct Batch {}
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

What a run of stages shares: the target, what it was already asked, the numbers already handed out in the batch, and what the pipelines before made of their files.

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

## `Verdict`

What a batch made of one file: planned, unclaimed, or refused.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum Verdict {}
```

## `Judged`

One file of a batch with the verdict on it, the place in the list of the pipeline that claimed it, and the effects its pipeline asks for.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct Judged {}
```

## `plan_batch`

Plan a whole batch, each path at most once, through the pipelines of a watch, in the order they are listed, against what the target holds. What the stages share lives only for this one batch.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub fn plan_batch(pipelines: &[(String, Pipeline)], records: Vec<Record>, target: &dyn Target) -> Vec<Judged> {}
```

## `Step`

What one file holds at one point of planning: the pipeline that claimed it, the stage just run by its place in that pipeline and its name or none for the claim, and the flow it goes on with.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct Step<'a> {}
```

## `plan_batch_observed`

Plan a batch as `plan_batch` does, telling `observe` every step of every file as it happens.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub fn plan_batch_observed(pipelines: &[(String, Pipeline)], records: Vec<Record>, target: &dyn Target, observe: &mut dyn FnMut(Step)) -> Vec<Judged> {}
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

## `Config::check_paths`

Check where the configuration file, the sources and the targets lie against one another, with each path taken to where `resolve` says it really is.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Config {
    pub fn check_paths(&self, config_file: &Path, resolve: impl Fn(&Path) -> PathBuf) -> Result<(), ConfigError> {}
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
pub fn move_file(tree: &dyn Tree, stage: &Move, record: &Record, roots: &Roots, dry_run: bool) -> Result<Applied, EffectError> {}
```

## `cleanup_folders`

Remove the folders a moved file left empty, from its folder upward and only inside its unit.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub fn cleanup_folders(tree: &dyn Tree, stage: &Cleanup, origin: &Path, unit: &Path, roots: &Roots) -> Result<Vec<PathBuf>, EffectError> {}
```

## `Done`

What one effect stage did.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum Done {}
```

## `Effect`

What an effect stage asks to be done to a planned file: a move or a cleanup, with the settings the stage was declared with.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum Effect {}
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

Carry out the effects a planned file carries, in their order, stopping at the first failure.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub fn apply_effects(tree: &dyn Tree, effects: &[Effect], record: &Record, unit: &Path, roots: &Roots, dry_run: bool) -> EffectsRun {}
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

## `Scan`

A walk over the regular files below a folder, as found-at-start events, that is taken a number of entries at a time. It follows no link.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct Scan {}
```

## `Scan::new`

Start a scan of a folder, whose files are named relative to the source.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Scan {
    pub fn new(source: &Path, folder: &Path) -> Scan {}
}
```

## `Scan::look_at`

Look at up to a number of entries, folders counted as well as files, and answer the files found among them. Each folder is told to `entering` before it is read, which is where a watch on it is asked for.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Scan {
    pub fn look_at(&mut self, entries: usize, entering: impl FnMut(&Path)) -> Vec<Event> {}
}
```

## `Scan::is_finished`

Whether the scan has looked at everything below its folder.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Scan {
    pub fn is_finished(&self) -> bool {}
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
pub fn process_batch(tree: &dyn Tree, watch: &Watch, unit: &Path, files: &[PathBuf], renames: &mut Renames) -> Vec<Processed> {}
```

## `process_batch_observed`

Process a ready batch as `process_batch` does, telling `observe` every step of its planning.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub fn process_batch_observed(tree: &dyn Tree, watch: &Watch, unit: &Path, files: &[PathBuf], renames: &mut Renames, observe: &mut dyn FnMut(Step)) -> Vec<Processed> {}
```

## `replaced_pipelines`

The pipelines of a watch that the folder configurations applying to a unit replace.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub fn replaced_pipelines(tree: &dyn Tree, watch: &Watch, unit: &Path) -> Vec<String> {}
```

## `Tree`

The files of the source and the target as processing a batch reads them and its effects change them, so the filesystem or a virtual tree can stand behind it.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub trait Tree {}
```

## `Tree::kind`

What is at a path, without following a link.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub trait Tree {
    fn kind(&self, path: &Path) -> io::Result<Kind>;
}
```

## `Tree::modified`

When the file at a path was last modified.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub trait Tree {
    fn modified(&self, path: &Path) -> io::Result<DateTime<Utc>>;
}
```

## `Tree::read`

The text of the file at a path, never more than a limit of its bytes.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub trait Tree {
    fn read(&self, path: &Path, limit: u64) -> io::Result<String>;
}
```

## `Tree::files`

The names of the regular files in a folder, none when it cannot be listed.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub trait Tree {
    fn files(&self, folder: &Path) -> Vec<String>;
}
```

## `Tree::is_empty`

Whether a folder holds nothing at all.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub trait Tree {
    fn is_empty(&self, folder: &Path) -> io::Result<bool>;
}
```

## `Tree::place`

Put a file where it is wanted, making the folders on the way, without ever replacing what is there.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub trait Tree {
    fn place(&self, from: &Path, to: &Path) -> Result<(), EffectError>;
}
```

## `Tree::remove_folder`

Remove a folder that holds nothing.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub trait Tree {
    fn remove_folder(&self, folder: &Path) -> io::Result<()>;
}
```

## `Kind`

What a path holds: a file, a folder, a link, or something else.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub enum Kind {}
```

## `FsTree`

The tree the filesystem holds.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct FsTree {}
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

## `Queue`

The notifications that say something changed, waiting for the runner, of which it holds a bounded number.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
pub struct Queue {}
```

## `Queue::new`

Start a queue with room for a number of notifications.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Queue {
    pub fn new(capacity: usize) -> Queue {}
}
```

## `Queue::handler`

The handler to give the filesystem watcher: it queues what says something changed and every error, drops what only says a file was opened, read or had its attributes changed, and notes a loss when there is no room or the kernel lost notifications.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Queue {
    pub fn handler(&self) -> impl FnMut(notify::Result<notify::Event>) + Send + 'static {}
}
```

## `Queue::take`

The notifications of one turn: the first is waited for, and those already there follow it, up to a limit.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Queue {
    pub fn take(&self, timeout: Duration, limit: usize) -> Vec<notify::Result<notify::Event>> {}
}
```

## `Queue::lost`

Whether notifications were lost since this was last asked.

| Attribute | Value |
| --- | --- |
| internal | yes |

```rust
impl Queue {
    pub fn lost(&self) -> bool {}
}
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
