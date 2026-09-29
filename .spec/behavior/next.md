# Next

A next stage fills a field with the next number after the ones already in the target. Only the part that needs no target is here; reading the target comes with the batch.

## Includes

- `tests/next.rs`

## `NXT-001` A field that is already set is left alone

| Step | Statement |
| --- | --- |
| Given | a next stage filling `episode` and a record whose `episode` is the number 7 |
| When | the stage is applied |
| Then | the record goes on unchanged |
