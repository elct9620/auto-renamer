# Queue

The queue holds the notifications that wait for the runner. Only what says something changed is queued, and only so many: when there is no room, or the kernel lost notifications of its own, the queue says that notifications were lost, and the runner starts over from a scan.

## Includes

- `tests/queue.rs`

## `QUE-002` A notification that nothing changed is dropped

| Step | Statement |
| --- | --- |
| Given | the handler a queue gives the filesystem watcher |
| When | it is handed notifications that a file was opened, closed after reading, and had its attributes changed |
| Then | none of them comes out of the queue |

## `QUE-003` An error is queued

| Step | Statement |
| --- | --- |
| Given | the handler a queue gives the filesystem watcher |
| When | it is handed an error of the filesystem watcher |
| Then | the error comes out of the queue |

## `QUE-004` Whatever is translated or rewrites the configuration is queued

| Step | Statement |
| --- | --- |
| Given | the handler a queue gives the filesystem watcher and a notification of every kind a translation or a rewrite answers to |
| When | it is handed each of them |
| Then | each comes out of the queue |

## `QUE-006` A queue with no room loses what comes and says so

| Step | Statement |
| --- | --- |
| Given | a queue with room for two notifications, holding two |
| When | it is handed a third |
| Then | it says notifications were lost |

## `QUE-007` Notifications the kernel lost count as lost

| Step | Statement |
| --- | --- |
| Given | a queue with room |
| When | it is handed the notification that the kernel's own queue overflowed |
| Then | it says notifications were lost |

## `QUE-009` A loss is told once

| Step | Statement |
| --- | --- |
| Given | a queue that lost notifications and has said so |
| When | it is asked again |
| Then | it says none were lost |

## `QUE-010` A turn takes no more notifications than its limit

| Step | Statement |
| --- | --- |
| Given | a queue holding five notifications |
| When | a turn takes up to three and the next takes up to three |
| Then | three come and then two |

