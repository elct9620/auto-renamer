# Runner

The runner is the watcher itself: it watches the sources, hands ready batches to be processed, and follows the configuration file as it changes. It runs until it is told to stop, and a broken configuration never stops it.

## Includes

- `tests/runner.rs`

## `RUN-001` A file dropped in the source ends up in the target

| Step | Statement |
| --- | --- |
| Given | a running watcher with a short batch window and a pipeline that moves |
| When | a file is written into the source |
| Then | the file appears in the target |

## `RUN-002` Files already there at start are processed

| Step | Statement |
| --- | --- |
| Given | a file already in the source |
| When | the watcher starts |
| Then | the file appears in the target |

## `RUN-003` A change to the configuration takes effect

| Step | Statement |
| --- | --- |
| Given | a running watcher whose pipeline keeps the name |
| When | the configuration is changed to rename the file to `renamed` and a file is written |
| Then | the file appears in the target as `renamed` |

## `RUN-004` A configuration that is not valid keeps the running one

| Step | Statement |
| --- | --- |
| Given | a running watcher |
| When | the configuration is changed to something that is not valid and a file is written |
| Then | the file is still processed by the earlier configuration |

## `RUN-005` A linked folder is not followed

| Step | Statement |
| --- | --- |
| Given | a running watcher and a symbolic link in the source to a folder outside it |
| When | a file is written into the folder outside the source |
| Then | the file stays where it was and the target stays empty |
