# Declaration

A pipeline is declared as a list of stages, each a bare name or a table with one key. Declarations may come from configuration inside downloaded folders, so every mistake is refused when the pipeline is read, naming the stage and the parameter at fault.

## Includes

- `tests/declaration.rs`

## `DEC-001` A bare name declares a stage without parameters

| Step | Statement |
| --- | --- |
| Given | the stage list `["move"]` |
| When | the pipeline is read |
| Then | the pipeline holds one `move` stage |

## `DEC-002` A one-key table declares a stage with parameters

| Step | Statement |
| --- | --- |
| Given | the stage list `[{ format = "{name}" }]` |
| When | the pipeline is read |
| Then | the pipeline holds one `format` stage |

## `DEC-003` A table with two keys is refused

| Step | Statement |
| --- | --- |
| Given | the stage list `[{ format = "{name}", move = true }]` |
| When | the pipeline is read |
| Then | the pipeline is refused |

## `DEC-004` An unknown stage name is refused

| Step | Statement |
| --- | --- |
| Given | the stage list `["shred"]` |
| When | the pipeline is read |
| Then | the pipeline is refused, naming the stage `shred` |

## `DEC-005` An unknown parameter is refused

| Step | Statement |
| --- | --- |
| Given | the stage list `[{ filter = { ext = ["mkv"], colour = "red" } }]` |
| When | the pipeline is read |
| Then | the pipeline is refused, naming the parameter `colour` |

## `DEC-007` A filter needs an extension list or a name pattern

| Step | Statement |
| --- | --- |
| Given | the stage list `[{ filter = { invert = true } }]` |
| When | the pipeline is read |
| Then | the pipeline is refused |

## `DEC-009` Number extraction needs a field to write into

| Step | Statement |
| --- | --- |
| Given | the stage list `[{ number = { prefix = "Season" } }]` |
| When | the pipeline is read |
| Then | the pipeline is refused, naming the parameter `into` |

## `DEC-010` The nth candidate cannot be zero

| Step | Statement |
| --- | --- |
| Given | the stage list `[{ number = { into = "episode", nth = 0 } }]` |
| When | the pipeline is read |
| Then | the pipeline is refused |

## `DEC-011` A pattern that is not a regular expression is refused

| Step | Statement |
| --- | --- |
| Given | the stage list `[{ regex = { pattern = "(", into = "x" } }]` |
| When | the pipeline is read |
| Then | the pipeline is refused, naming the stage `regex` |

## `DEC-012` A regular expression cannot both extract and rewrite

| Step | Statement |
| --- | --- |
| Given | the stage list `[{ regex = { pattern = "a", into = "x", replace = "b" } }]` |
| When | the pipeline is read |
| Then | the pipeline is refused |

## `DEC-013` A pattern that compiles to something enormous is refused

| Step | Statement |
| --- | --- |
| Given | the stage list holding a regular expression that repeats a repeated repeat a thousand times over |
| When | the pipeline is read |
| Then | the pipeline is refused, naming the stage `regex` |

## `DEC-014` A fixed value is text or a whole number

| Step | Statement |
| --- | --- |
| Given | the stage list `[{ set = { season = true } }]` |
| When | the pipeline is read |
| Then | the pipeline is refused, naming the parameter `season` |

## `DEC-015` A case change is lower, upper or title

| Step | Statement |
| --- | --- |
| Given | the stage list `[{ case = { to = "shout" } }]` |
| When | the pipeline is read |
| Then | the pipeline is refused, naming the parameter `to` |

## `DEC-016` A bracket group is an opening and a closing character

| Step | Statement |
| --- | --- |
| Given | the stage list `[{ strip = { groups = ["[[]"] } }]` |
| When | the pipeline is read |
| Then | the pipeline is refused, naming the parameter `groups` |

## `DEC-017` A malformed template is refused when the pipeline is read

| Step | Statement |
| --- | --- |
| Given | the stage list `[{ format = "{name" }]` |
| When | the pipeline is read |
| Then | the pipeline is refused, naming the stage `format` |

## `DEC-018` A lift can be a number of levels

| Step | Statement |
| --- | --- |
| Given | the stage list `[{ lift = 1 }]` |
| When | the pipeline is read |
| Then | the pipeline holds one `lift` stage |

## `DEC-019` A lift target that is not a name pattern is refused

| Step | Statement |
| --- | --- |
| Given | the stage list `[{ lift = { to = "[" } }]` |
| When | the pipeline is read |
| Then | the pipeline is refused, naming the parameter `to` |

## `DEC-020` Ranking needs the fields that make a group

| Step | Statement |
| --- | --- |
| Given | the stage list `[{ rank = { into = "index", by = [] } }]` |
| When | the pipeline is read |
| Then | the pipeline is refused, naming the parameter `by` |

## `DEC-021` A conflict policy is reject or suffix

| Step | Statement |
| --- | --- |
| Given | the stage list `[{ move = { on_conflict = "overwrite" } }]` |
| When | the pipeline is read |
| Then | the pipeline is refused, naming the parameter `on_conflict` |

## `DEC-022` A stage that only rewrites the plan cannot follow an effect

| Step | Statement |
| --- | --- |
| Given | the stage list `["move", { format = "{name}" }]` |
| When | the pipeline is read |
| Then | the pipeline is refused, naming the stage `format` |

## `DEC-023` A path stage cannot follow next

| Step | Statement |
| --- | --- |
| Given | the stage list holding `next` and then `lift` |
| When | the pipeline is read |
| Then | the pipeline is refused, naming the stage `lift` |

## `DEC-024` A pipeline without an effect stage is read but reported

| Step | Statement |
| --- | --- |
| Given | the stage list `[{ format = "{name}" }]` |
| When | the pipeline is read |
| Then | the pipeline says it has no effect stage |

## `DEC-025` Stages keep the order they were written in

| Step | Statement |
| --- | --- |
| Given | the stage list `[{ filter = { ext = ["mkv"] } }, { format = "{name}" }, "move"]` |
| When | the pipeline is read |
| Then | the pipeline holds `filter`, `format` and `move` in that order |

## `DEC-026` The stages must be a list

| Step | Statement |
| --- | --- |
| Given | the document `stages = "move"` |
| When | the pipeline is read |
| Then | the pipeline is refused |

## `DEC-027` A conflict suffix cannot reach outside the file name

| Step | Statement |
| --- | --- |
| Given | the stage list `[{ move = { on_conflict = "suffix", suffix = "/../x" } }]` |
| When | the pipeline is read |
| Then | the pipeline is refused, naming the parameter `suffix` |

## `DEC-028` A move rejects on conflict unless told otherwise

| Step | Statement |
| --- | --- |
| Given | the stage list `["move"]` |
| When | the pipeline is read |
| Then | the move stage rejects a file whose target exists |

## `DEC-030` A literal replace needs something to look for

| Step | Statement |
| --- | --- |
| Given | the stage list `[{ replace = { find = "", with = "x" } }]` |
| When | the pipeline is read |
| Then | the pipeline is refused, naming the parameter `find` |

## `DEC-031` A next template must mention the field it fills

| Step | Statement |
| --- | --- |
| Given | the stage list `[{ next = { into = "episode", like = "{name}" } }]` |
| When | the pipeline is read |
| Then | the pipeline is refused, naming the parameter `like` |

## `DEC-032` A pipeline of too many stages is refused

| Step | Statement |
| --- | --- |
| Given | a stage list of 65 stages |
| When | the pipeline is read |
| Then | the pipeline is refused |

## `DEC-033` Every stage is described with an example it accepts

| Step | Statement |
| --- | --- |
| Given | the description of each stage there is |
| When | each stage is declared with its example |
| Then | every declaration is accepted |

## `DEC-034` A described parameter is one the stage accepts

| Step | Statement |
| --- | --- |
| Given | each named parameter a stage's description lists |
| When | the stage is declared with its example and that parameter |
| Then | none is refused as not a parameter of the stage |

## `DEC-035` A parameter a stage reads is described

| Step | Statement |
| --- | --- |
| Given | declarations reaching every parameter each stage reads |
| When | they are read |
| Then | every parameter read is one the stage's description lists |

## `DEC-036` A required parameter left out is refused

| Step | Statement |
| --- | --- |
| Given | each parameter a stage's description marks as required |
| When | the stage is declared with its example without that parameter |
| Then | the declaration is refused |

## `DEC-037` A parameter limited to choices accepts each of them

| Step | Statement |
| --- | --- |
| Given | each choice a stage's description lists for a parameter |
| When | the stage is declared with its example and that choice |
| Then | the declaration is accepted |
