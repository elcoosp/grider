# 0006 — Plugin selection and deployment assumptions

- **Status:** accepted
- **Date:** 2026-11-07
- **Scope:** Starlight configuration

## Enabled plugins

| Plugin | Version | Purpose |
| :--- | :--- | :--- |
| `starlight-links-validator` | 0.27.0 | Fails the build on broken internal links — a release blocker |
| `starlight-image-zoom` | 0.16.2 | Click-to-zoom on the input/overlay figures |
| `starlight-heading-badges` | 0.8.0 | Inline badges in headings (used for feature gates and `non_exhaustive`) |
| `starlight-llms-txt` | 0.12.0 | Emits `llms.txt`, `llms-full.txt`, `llms-small.txt` and per-page text files for AI-agent discoverability |

## Omitted plugins

| Plugin | Reason |
| :--- | :--- |
| `starlight-typedoc` | Generates API reference from **TypeScript** source. Grider is a Rust crate with docs.rs as its rustdoc home; there is no TS surface to generate from. The Rust reference is hand-authored from the source in `reference/`. |
| `starlight-blog` | A blog/changelog section adds maintenance overhead for a single-version crate site; the changelog lives in `CHANGELOG.md` (git-cliff) in the repository. |
| `starlight-openapi` | No OpenAPI specification exists — grider is a library and CLI, not an HTTP API. |
| `starlight-versions` | Versioned docs are premature for an early-stage crate with a single released version. |
| DocSearch (`@astrojs/starlight-docsearch`) | No approved Algolia DocSearch application exists. Pagefind (Starlight's built-in zero-config search) is the search solution and is kept enabled. |

## Deployment assumptions

- `starlight-llms-txt` requires `site` in the Astro config. The URL is set to the canonical GitHub
  Pages location `https://elcoosp.github.io/grider/` — an assumption, not a verified deployment.
  Change `SITE` in `astro.config.mjs` when hosting elsewhere. No `base` is set (root-path deploy);
  add `base` only if the site moves under a subpath.
- `editLink.baseUrl` assumes the docs live in `docs-site/` on the `main` branch of
  `github.com/elcoosp/grider`.
