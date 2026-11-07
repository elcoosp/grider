# 0003 — Syntax highlighter: Shiki via Expressive Code (no Rangi)

- **Status:** accepted
- **Date:** 2026-11-07
- **Scope:** code block rendering

## Context

Starlight renders code blocks through **Expressive Code**, whose supported highlighter is
**Shiki** — TextMate grammars (the same ones VS Code uses), 180+ languages, build-time execution
(zero client-side JavaScript), native integration with Starlight's light/dark themes.

Rangi is a legitimate lightweight highlighter, but the comparison published by Rangi itself places
it in a different niche: ~46 languages, regex tokenizing, tiny bundle (~13 kB) — best for chat
UIs, SSGs and small apps, while Shiki is designated for docs sites with rich code samples. Rangi is
not integrated with Expressive Code, so adopting it would mean writing and maintaining a custom
highlighter plugin with no upstream support, and losing transformers, Twoslash and the broad
language set (this site fences `rust`, `toml`, `bash`, `json`, `yaml` and `ansi`).

## Decision

Ship with Shiki, the Starlight default through Expressive Code. No `expressiveCode` overrides are
configured — the default dual-theme setup already matches the site's light/dark toggle, and the
default grammar set covers every language fenced in this documentation. No Rangi plugin is added.

## Consequences

- Every fenced code block declares a language (`rust`, `toml`, `bash`, `json`, `yaml`, `ansi`);
  terminal transcripts use the `ansi` tag.
- Built-in features (copy button, line highlighting where used) come from Expressive Code with zero
  additional configuration.
