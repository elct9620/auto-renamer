# Plan

A pipeline planned for one file runs its stages in the order written, up to the first stage that would touch the filesystem, and answers with the record as it would be moved.

## Includes

- `tests/plan.rs`

## `PLN-001` Stages run in order, each on the result of the last

| Step | Statement |
| --- | --- |
| Given | a pipeline that reads the episode and then formats the name, and a record for `Show - 12.mkv` |
| When | the pipeline is planned |
| Then | the plan is the formatted name |

## `PLN-002` An excluded record stops the run

| Step | Statement |
| --- | --- |
| Given | a pipeline that starts with a filter for `mkv`, and a record for `x.nfo` |
| When | the pipeline is planned |
| Then | the record is excluded |

## `PLN-003` A refusal stops the run and names the stage

| Step | Statement |
| --- | --- |
| Given | a pipeline whose second stage refuses the record |
| When | the pipeline is planned |
| Then | the record is refused, naming that stage |

## `PLN-004` The run stops before the first effect stage

| Step | Statement |
| --- | --- |
| Given | a pipeline that formats and then moves, and a record for `x.mkv` |
| When | the pipeline is planned |
| Then | the plan is the formatted name, and nothing has been moved |
