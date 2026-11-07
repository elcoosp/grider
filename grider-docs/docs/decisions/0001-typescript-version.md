# 0001 — Pin TypeScript to 6.x

- **Status:** accepted
- **Date:** 2026-11-07
- **Scope:** docs-site toolchain

## Context

`astro check` (Astro 7.4.x) does not support TypeScript 7. Running type checking with TypeScript
7.0.2 (the current `latest` on the npm registry) fails with the error:

> astro check does not currently support TypeScript 7.0

Astro's guidance is to keep using the latest TypeScript 6.x release until `astro check` gains
TypeScript 7 support. The experimental workaround (`tsc` plus `@astrojs/ts-content-mapper`) exists
but is not the stable default and is not adopted here.

## Decision

`typescript` is pinned to `6.0.3` (the latest 6.x release at build time) in `package.json` — an
exact pin, not a caret range, so a `pnpm up` cannot silently pull 7.x into `pnpm check`.

## Consequences

- `pnpm check` runs against TypeScript 6.x and reports clean results.
- Revisit this decision when Astro announces TypeScript 7 support for `astro check`; then bump the
  pin deliberately, re-run `pnpm check`, and delete this note.
