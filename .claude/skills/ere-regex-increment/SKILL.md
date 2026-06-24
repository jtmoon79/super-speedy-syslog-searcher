---
name: ere-regex-increment
description: "Use when: incrementing, renumbering, shifting, or inserting ERE_REGEX_DATETIME regex id declarations in ere_datetimes_impl.rs. Updates cfg regex ids, macro first arguments, and DATETIME_PARSE_DATAS_LEN_MAX, then verifies pair continuity."
argument-hint: "<start regex id, line, or nearby comment> [increment]"
---

# ERE Regex Increment

Use this skill to renumber `ERE_REGEX_DATETIME!` declarations in `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs` when a new datetime regex is inserted or an existing run needs to shift.

The workflow updates the declaration identifier, not datetime test expectations or match offsets.

## Target Pattern

Each declaration has two paired identifiers that must stay equal:

```rust
#[cfg(any(regex = "N", regex = "ALL"))]
ERE_REGEX_DATETIME!(
    N,
    counter!(DP_KEY),
    // ...
)
```

Some declarations include extra cfg selectors, such as `regex = "TEST"`. Preserve all non-numeric selectors and only change the numeric regex id for the declaration being renumbered.

## Preflight

1. Read the target area in `subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs`.
2. Identify the exact starting declaration from the user's line number, regex id, nearby comment, or example text.
3. If the same numeric id appears more than once near the start point, use the line number or surrounding comment to choose the intended declaration. Ask a short clarification only if the start declaration is genuinely ambiguous.
4. Determine the increment amount. Default to `+1` when the user says increment, shift, bump, or make room.
5. Find the last declaration in the affected run and the current value of `DATETIME_PARSE_DATAS_LEN_MAX`.

## Edit Rules

For each affected `ERE_REGEX_DATETIME!` declaration, update exactly these values in lockstep:

1. The numeric `regex = "N"` selector inside the immediately preceding `#[cfg(any(...))]` attribute.
2. The first numeric argument inside the immediately following `ERE_REGEX_DATETIME!` invocation.

Do not change:

- Test-vector tuple values inside the `&[...]` examples.
- Date, time, offset, or expected parse tuple values.
- Comments, examples, declaration order, or regex pattern text.
- Other cfg selectors such as `regex = "ALL"` or `regex = "TEST"`.
- Unrelated numeric literals in the macro body.

If the final declaration id changes, update `DATETIME_PARSE_DATAS_LEN_MAX` to the new highest declaration id.

## Verification

After editing, verify the file before reporting success.

1. Confirm every numeric cfg id matches the first argument of the immediately following `ERE_REGEX_DATETIME!` declaration.
2. Confirm the renumbered range is continuous for the intended affected declarations.
3. Confirm the final declaration id matches `DATETIME_PARSE_DATAS_LEN_MAX` when the tail was shifted.
4. Run:

```sh
git diff --check -- subprojects/ere/ere_datetimes_impl/src/ere_datetimes_impl.rs
```

5. If cargo validation is requested or the edit is risky, use this repo's regex build convention:

```sh
S4_BUILD_REGEX=1 cargo check
S4_BUILD_REGEX=ALL cargo check
```

## Reporting

Summarize the starting declaration, final id range, whether `DATETIME_PARSE_DATAS_LEN_MAX` changed, and which verification checks passed.
