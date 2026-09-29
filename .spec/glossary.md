# Glossary

The words this project keeps, and the ones it turns down in their place.

## Everywhere

### Includes

- `src/**/*.rs`

### Unit

The set of files that may relate to one another, such as a video and its subtitles. A watch says how a file is assigned to a unit; files of different units are never grouped and never share numbering.

#### Rejected

- `Bundle` - Unit is what the design calls it.

### Batch

The settled files of one unit, collected until the unit has been quiet long enough, or has waited long enough, to be processed together.

#### Rejected

- `Chunk` - Batch is what the design calls it.

### Group

The files of one batch whose fields agree, such as one episode's video and its subtitles. A group is found by fields, never by the batch or the folder.

#### Rejected

- `Cluster` - Group is what the design calls it.

### Stage

One small function over a batch. A pipeline is a stack of stages, and each stage does one thing.

#### Rejected

- `Step` - Stage is what the design calls it.
- `Processor` - Stage is what the design calls it.

### Pipeline

The ordered stages a watch applies to the files it claims.

#### Rejected

- `Workflow` - Pipeline is what the design calls it.

### Record

One file as it moves through a pipeline: a plan path together with named fields.

#### Rejected

- `Item` - Record is what the design calls it.

### Plan

The path, relative to the source, that a record will be moved to. Stages before the effect stages only rewrite the plan and never touch the filesystem.

#### Rejected

- `Destination` - Plan is what the design calls it.

### Settled

Said of a file whose writing has finished, so it may enter a batch.

#### Rejected

- `Stable` - Settled is what the design calls it.
