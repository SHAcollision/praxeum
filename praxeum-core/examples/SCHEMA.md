# Praxeum Exercise Schema

Praxeum content is distributed as TOML (primary) or JSON. Files describe an array of `exercise` entries. JSON either uses a top-level array or `{ "exercises": [ ... ] }`; TOML uses `[[exercise]]` tables.

## Common rules
- `id`, `title`, and `prompt` must be non-empty strings.
- IDs must be unique within a file.
- Every exercise must include at least one meaningful prompt and the data needed for its type.

## Classification
```toml
[[exercise]]
kind = "classification"
id = "intro_action_event"
title = "Action vs Event"
prompt = "Classify each statement"
categories = ["action", "event"]

[[exercise.items]]
text = "Alice flips a coin"
correct_category = "action"
```
- Provide **two or more** `categories`; entries must be unique and non-empty.
- Each `items` entry needs `text` and a `correct_category` that appears in `categories`.
- Include at least one item.

## Multiple-choice
```toml
[[exercise]]
kind = "multiple_choice"
id = "opp_cost"
title = "Opportunity Cost"
prompt = "What is the cost of choosing to call a friend?"
options = ["Next-best forgone option", "All unchosen options"]
correct_indices = [0]
multi_select = false
```
- `options` must be non-empty strings; duplicate options are flagged.
- `correct_indices` uses zero-based positions into `options`.
- For `multi_select = false`, supply exactly one `correct_indices` entry. Multi-select allows several.

## Scenario
```toml
[[exercise]]
kind = "scenario"
id = "exchange_direct_indirect"
title = "Find the trade line"
description = "Three castaways with different holdings"
prompt = "Which move gets A coconuts fastest?"

[[exercise.choices]]
label = "A trades fish directly to B for coconuts"
is_correct = true
feedback = "Direct exchange works here"
```
- `description` and `prompt` must be present and non-empty.
- Provide at least one `choices` entry; each needs `label` and `feedback` text.
- Mark at least one choice with `is_correct = true`.

## Validation helpers
Use `ExerciseLoader::validate_path` or `ExerciseLoader::validate_str` to preflight files. The helpers return structured issues indicating the exercise `id`, field name, and reason for failure, e.g., duplicate IDs, missing prompts, or out-of-range indices.
