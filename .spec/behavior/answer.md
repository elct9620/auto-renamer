# Answer

The `vars` of a configuration are answers: what detection could not find, or found wrong, given by the user. Detecting stages and the values a pipeline fixes fill only the fields no answer gave, so one answer in a folder is enough for an exception, while stages that rework a value still apply to it.

## Includes

- `tests/answer.rs`

## `ANS-001` A number leaves a field an answer gave

| Step | Statement |
| --- | --- |
| Given | a file `Show 07.mkv` whose `vars` give `episode = 3` |
| When | a number stage writes into `episode` |
| Then | `episode` stays 3 |

## `ANS-002` A regex capture leaves a field an answer gave

| Step | Statement |
| --- | --- |
| Given | a file under `Series/Alpha` whose `vars` give `show = "Alpha Next"` |
| When | a regex captures `show` from the path |
| Then | `show` stays `Alpha Next` |

## `ANS-003` A fixed value leaves a field an answer gave

| Step | Statement |
| --- | --- |
| Given | a file whose `vars` give `show = "Alpha"` |
| When | a set stage fixes `show = "Beta"` |
| Then | `show` stays `Alpha` |

## `ANS-004` A stage that reworks a value still works on an answer

| Step | Statement |
| --- | --- |
| Given | a file whose `vars` give `show = "alpha"` |
| When | a case stage turns `show` upper |
| Then | `show` is `ALPHA` |
