# Install Summary Output Refactor Plan

## Background
The result summary in `src/commands/install.rs` is hard to scan because the
guarded `match` mixes conditions with output strings and repeats `println!`.

## Goal
- Improve readability of the summary logic.
- Keep external behavior unchanged.

## Non-Goals
- No CLI behavior changes.
- No new options or output formats.

## Approach
- Extract summary selection into a helper function.
- Return a small struct or tuple, e.g. `(prefix, message)` or `Summary { prefix, message }`.
- Print once at the call site.

## Steps
1. Add `Summary` and `format_summary(total_success, total_failure)` near `install` logic.
2. Replace the current `match` with a helper call and single `println!`.
3. (Optional) Add a small unit test for the helper if desired.

## Validation
- Confirm output for:
  - failure > 0
  - success > 0 and failure == 0
  - success == 0 and failure == 0
