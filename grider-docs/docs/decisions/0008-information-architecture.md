# 0008 — Information architecture and source documentation debt

- **Status:** accepted
- **Date:** 2026-11-07
- **Scope:** content architecture

## Diátaxis mapping

One type per page; mixing is forbidden.

| Type | Folder | Pages |
| :--- | :--- | :--- |
| Tutorial | `src/content/docs/tutorials/` | `extract-your-first-grid` |
| How-to | `src/content/docs/guides/` | `install`, `configure-extraction`, `choose-binarization`, `filter-lines`, `draw-overlays`, `serialize-grids`, `use-the-cli` |
| Reference | `src/content/docs/reference/` | `grid`, `lines-and-cells`, `grid-config`, `binarization`, `drawing`, `pipeline`, `macros`, `errors`, `cli`, `feature-flags` |
| Explanation | `src/content/docs/explanation/` | `extraction-pipeline`, `performance` |

The landing page (`index.mdx`) is a navigation hub, not a Diátaxis unit, so it is the only page
without the `type` frontmatter field (documented in `src/content.config.ts`).

## Feature inventory coverage

Every public feature of the crate has both a task page (tutorial or how-to) and a reference entry:
extraction (`try_from_image_with_config` / `try_from_gray_with_config` / `try_from_binarized`),
configuration (`GridConfig` + builder + validation), binarization choice (AdaptiveMean / Sauvola),
filtering (size filters, kinds, retain, coordinate lookups), drawing (`GridDrawingConfig`,
`Drawable`, `debug::save_image_with_grid`), serde serialization, the CLI, macros, feature flags and
errors.

## Source documentation debt (backlog for the repository, not this site)

1. **README version drift** — the README says `grider = "0.2"` while `Cargo.toml` is at `0.3.0`.
   This site documents 0.3 throughout.
2. **Broken README link** — the README links to `docs/BRAINSTORM.md`, which does not exist in the
   repository.
3. **0.3 breaking change under-communicated** — struct literals of `GridConfig` need the new
   `binarization` field or `..GridConfig::default()`; covered in this site's
   [serialize guide](/guides/serialize-grids/) and `GridConfig` reference.

## Truthfulness rules applied

- Every CLI transcript, JSON sample and figure on this site was produced by the real 0.3.0 binary
  on the repository's `tests/large.png` fixture.
- The 3 × 3 tutorial example reproduces a committed insta snapshot (`grid__construction__grid_processing_small_image.snap`).
- Benchmark numbers are quoted from the repository's `docs/BENCHMARKS.md` unchanged.
- No feature, flag or option was invented; omitted upstream plugins are listed in
  [0006](./0006-plugin-selection.md).
