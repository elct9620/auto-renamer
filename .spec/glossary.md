# Glossary

The words this project keeps, and the ones it turns down in their place.

## Everywhere

### Includes

- `src/**/*.rs`
- `crates/*/src/**/*.rs`

### Unit

The set of files that may relate to one another, such as a video and its subtitles. A watch says how a file is assigned to a unit; files of different units are never grouped and never share numbering.

#### Rejected

- `Bundle` - Unit is what the design calls it.

### Batch

The settled files of one unit, collected until the unit has been quiet long enough, or has waited long enough, to be processed together. The part of a batch a pipeline claims is a batch of its own, which each of its stages takes and hands on.

#### Rejected

- `Chunk` - Batch is what the design calls it.

### Group

The files of one batch whose fields agree, such as one episode's video and its subtitles. A group is found by fields, never by the batch or the folder.

#### Rejected

- `Cluster` - Group is what the design calls it.

### Stage

One small function from a batch to a batch. A pipeline is a stack of stages, and each stage does one thing.

#### Rejected

- `Step` - Stage is what the design calls it.
- `Processor` - Stage is what the design calls it.

### Parameter

What a stage is declared with: a single value, or named values in its table. The core describes each stage's parameters so the playground's forms offer what the parser reads.

### Pipeline

The ordered stages a route applies to the files it claims.

#### Rejected

- `Workflow` - Pipeline is what the design calls it.

### Record

One file as it moves through a pipeline: a plan path together with named fields.

#### Rejected

- `Item` - Record is what the design calls it.

### Plan

The path, relative to the source, that a record will be moved to under its route's target. Stages only rewrite the plan and never touch the filesystem.

#### Rejected

- `Destination` - Plan is what the design calls it.

### Settled

Said of a file whose writing has finished, so it may enter a batch.

#### Rejected

- `Stable` - Settled is what the design calls it.

### Watch

One watched folder with how its files are grouped into units and the routes that name and move them.

#### Rejected

- `Job` - Watch is what the design calls it.

### Route

One entry of a watch's routes: the pipeline that claims and names files, joined to the effects carried out on what it planned, and the route a rejected file takes instead. Routes claim in the order they are listed.

#### Rejected

- `Rule` - Route is what the design calls it.

### Target

A named root files are moved to, declared only in the global configuration and referred to by name from a route.

### Folder configuration

The `auto-renamer.toml` a folder carries for the exceptions of what lies in it. It comes from downloaded content, so it may change how files are named and never where they come from, go, or how they are grouped.

#### Rejected

- `Local` - Folder configuration is what the design calls it.

### Verdict

What a batch made of one file: planned, unclaimed or refused.

#### Rejected

- `Disposition` - Verdict is what the design calls it.

### Flow

What a stage answers for one record: the record to go on with, or the stop that ends its way through the pipeline. A stop is a refusal, naming the stage and its reason; a filter that turns a file away refuses it too.

#### Rejected

- `Outcome` - Flow is a result, so a stage passes a stop on instead of matching on it.

### Answer

A field the `vars` of a configuration gave a file, the nearest folder configuration first. Detecting stages and the values a pipeline fixes never overwrite an answer; stages that rework a value still apply to it.

#### Rejected

- `Override` - Answer is what the design calls it; overriding is what folder configurations do to whole pipelines.

### Step

What one file holds at one point of its planning: once a pipeline claims it, and after each stage it reaches, until a stage stops it. Only the playground asks for steps; the CLI plans without keeping any.

#### Rejected

- `Snapshot` - Step is what the playground shows one at a time.

### Effect

What a route does to a planned file: a move to its target, or in place, and a cleanup. It is carried out once the whole batch is planned.

#### Rejected

- `Action` - Effect is what the design calls it.

### Queue

The notifications that say something changed, waiting for the runner. It holds a bounded number; when it has no room, or the kernel lost notifications of its own, notifications are lost and the watcher starts over from a scan.

#### Rejected

- `Inbox` - Queue is what the design calls it.
- `Backlog` - Queue is what the design calls it.

### Tree

The files of the source and the target as processing a batch reads them and its effects change them. The filesystem stands behind it for the CLI and a virtual tree for the playground, so both run the same rules.

### Playground

The page on GitHub Pages where a configuration is edited, simulated, and downloaded as a global or folder configuration. Only the core runs there for real.

### Simulation

A run of the playground: the user's virtual source is triggered, each unit becomes a batch, the core plans it, and the virtual files are moved by the verdicts. A preview is what a dry run reports instead, and a simulation honours a dry run as the CLI does.

## Traditional Chinese

### Includes

- `README.md`
- `docs/*.md`
- `playground/src/locales/zh-TW.json`

### 監控

A watch, as the Traditional Chinese text calls it; the key `watch` stays as the CLI writes it.

#### Rejected

- `監看` - 監控 is what the Traditional Chinese text calls a watch.
