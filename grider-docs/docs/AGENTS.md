# AGENTS.md — Authoring guide for this documentation site

This site documents [elcoosp/grider](https://github.com/elcoosp/grider) 0.3.x. It is built with
Astro 7.4 + Starlight 0.42, uses pnpm as the package manager, and follows the Diátaxis framework.

## Commands

```bash
pnpm install   # install pinned dependencies
pnpm dev       # local dev server on http://localhost:4321
pnpm build     # production build (fails on broken internal links)
pnpm check     # astro check against TypeScript 6.0.3 (7.x is unsupported)
pnpm lint      # markdownlint over src/content/docs
pnpm qa        # playwright screenshots of every route × 2 viewports × 2 themes
node scripts/search-probe.mjs  # Pagefind search modal probe (needs preview server)
```

## Information architecture

| Type | Folder | Frontmatter |
| :--- | :--- | :--- |
| Tutorial | `src/content/docs/tutorials/` | `type: tutorial` |
| How-to | `src/content/docs/guides/` | `type: how-to` |
| Reference | `src/content/docs/reference/` | `type: reference` |
| Explanation | `src/content/docs/explanation/` | `type: explanation` |

One type per page. The landing page `src/content/docs/index.mdx` is the only page without `type`
(it is a hub, not a Diátaxis unit — see `docs/decisions/0008-information-architecture.md`).

## Writing rules

- Second person, present tense, active voice; imperative for instructions.
- Never write "simply", "just", "easy" or "obviously".
- Every page: `title` + `description` frontmatter, a one-sentence summary up top, prerequisites
  where relevant, and Next steps via `<LinkCard>`.
- Every fenced code block declares a language (`rust`, `toml`, `bash`, `json`, `yaml`, `ansi` for
  terminal transcripts).
- Reference pages use tables: signature, type, default, description, example.

## Truthfulness rules

- Never document a feature, flag or option that does not exist in the source. Check `src/` of the
  grider repository before adding or changing API claims.
- CLI transcripts, JSON samples and figures must come from real runs of the `grider` binary on the
  repository fixtures (`tests/large.png`). Do not hand-craft output.
- Benchmark numbers are quoted from `docs/BENCHMARKS.md` of the repository, unchanged.
- The crate version is 0.3; the upstream README still says 0.2 (tracked in
  `docs/decisions/0008-information-architecture.md`).

## Components

Import from `@astrojs/starlight/components`: `Card`, `CardGrid`, `LinkCard`, `Tabs`, `TabItem`,
`Steps`, `Aside`, `FileTree`, `Badge`. `:badge[label]{variant="…"}` (variants: `caution`, `danger`,
`default`, `note`, `success`, `tip`) adds a badge to a heading via `starlight-heading-badges`.

## Theming

- Accent: steel-slate derived from the logo — see `src/styles/custom.css` and
  `docs/decisions/0007-accent-color.md`. Never revert to Starlight's default blue.
- Logo: `src/assets/logo-light.png` / `logo-dark.png` (raster pipeline, see
  `docs/decisions/0002-logo-format.md` — never upscale, replace with the SVG original when one
  exists upstream).
- Both themes must stay readable; the dark/light toggle must not be removed.
- The unprefixed default locale uses the reserved `root` key (`locales: { root: … }`); a key named
  after the language creates a /en/-prefixed locale and breaks sidebar links, active states and
  pagination (see `docs/decisions/0009-locale-sidebar-pagination.md`).

## Release gate

`pnpm build` (zero warnings) → `pnpm check` (0 errors, 0 warnings) → `pnpm lint` (clean) →
`pnpm preview` → `pnpm qa` (exit 0) → update `docs/qa-report.md`. Any change after the QA pass
invalidates it. Ship only `docs-site.zip` excluding `node_modules/`, `dist/`, `.astro/`, `.qa/`.
