# 0005 — Known warnings

- **Status:** accepted
- **Date:** 2026-11-07
- **Scope:** release gate

## Policy

Any `warn`, `warning` or `error` line in `pnpm build`, `pnpm check` or `pnpm lint` is a release
blocker. The only permitted exception is a warning originating from a third-party dependency that
cannot be fixed in this repository; such a warning must be recorded here with its exact output line
and a narrow `grep -v` filter matching exactly that string (blanket suppression is forbidden).

## Current state

**None.** At the time of the final QA pass, `pnpm build`, `pnpm check` and `pnpm lint` all run
clean: zero warnings, zero errors. No allowlist entries exist, so the zero-warning assertion is a
plain negative grep across the captured logs (`.qa/build.log`, `.qa/check.log`, `.qa/lint.log`).

If a future dependency starts emitting warnings, add the exact line and reason as a new entry in
this file, with the narrow filter, and re-run the full gate.
