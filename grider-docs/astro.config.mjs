// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import starlightLinksValidator from 'starlight-links-validator';
import starlightImageZoom from 'starlight-image-zoom';
import starlightHeadingBadges from 'starlight-heading-badges';
import starlightLlmsTxt from 'starlight-llms-txt';

// `site` is required by the starlight-llms-txt plugin, which emits llms.txt,
// llms-full.txt, llms-small.txt and per-page text files at build time.
// The URL below assumes the canonical GitHub Pages location for the grider
// repository; adjust it when deploying elsewhere (see docs/decisions/0006-*.md).
const SITE = 'https://elcoosp.github.io/grider/';

export default defineConfig({
        site: SITE,
        integrations: [
                starlight({
                        title: 'Grider',
                        description:
                                'High-performance row and column grid extraction from images, as a Rust library and a CLI.',
                        logo: {
                                light: './src/assets/logo-light.png',
                                dark: './src/assets/logo-dark.png',
                        },
                        customCss: ['./src/styles/custom.css'],
                        defaultLocale: 'en',
                        // The docs collection provides src/content/docs/404.md; the built-in
                        // dedicated /404 route is disabled so the two do not conflict.
                        disable404Route: true,
                        locales: {
				// `root` marks the unprefixed default locale in current Starlight;
				// a key named after the language would create a /en/ prefixed locale.
				root: { label: 'English', lang: 'en' },
			},
                        head: [
                                {
                                        tag: 'link',
                                        attrs: { rel: 'icon', type: 'image/png', href: '/favicon.png' },
                                },
                                {
                                        tag: 'link',
                                        attrs: { rel: 'apple-touch-icon', href: '/apple-touch-icon.png' },
                                },
                        ],
                        social: [
                                {
                                        icon: 'github',
                                        label: 'GitHub',
                                        href: 'https://github.com/elcoosp/grider',
                                },
                        ],
                        editLink: {
                                baseUrl: 'https://github.com/elcoosp/grider/edit/main/docs-site/',
                        },
                        plugins: [
                                // Fails the build on broken internal links (release gate).
                                starlightLinksValidator(),
                                // Click-to-zoom on figures (input vs. overlay comparisons).
                                starlightImageZoom(),
                                // Badges in headings, e.g. `:badge[serde]{variant="note"}`.
                                starlightHeadingBadges(),
                                // Emits llms.txt, llms-full.txt, llms-small.txt and per-page .txt files.
                                starlightLlmsTxt({
                                        projectName: 'Grider',
                                        description:
                                                'Documentation for Grider, a high-performance Rust library and CLI that extracts the row and column grid of an image.',
                                }),
                        ],
                        sidebar: [
                                {
                                        label: 'Start here',
                                        items: [
                                                { label: 'Overview', link: '/' },
                                                { label: 'Installation', link: '/guides/install/' },
                                                {
                                                        label: 'Extract your first grid',
                                                        link: '/tutorials/extract-your-first-grid/',
                                                },
                                        ],
                                },
                                {
                                        label: 'How-to guides',
                                        items: [
                                                { label: 'Configure extraction', link: '/guides/configure-extraction/' },
                                                {
                                                        label: 'Choose a binarization method',
                                                        link: '/guides/choose-binarization/',
                                                },
                                                { label: 'Filter grid lines', link: '/guides/filter-lines/' },
                                                { label: 'Draw grids and cells', link: '/guides/draw-overlays/' },
                                                { label: 'Serialize grids to JSON', link: '/guides/serialize-grids/' },
                                                { label: 'Use the CLI', link: '/guides/use-the-cli/' },
                                        ],
                                },
                                {
                                        label: 'Reference',
                                        items: [
                                                { label: 'Grid', link: '/reference/grid/' },
                                                { label: 'Lines and cells', link: '/reference/lines-and-cells/' },
                                                { label: 'GridConfig', link: '/reference/grid-config/' },
                                                { label: 'Binarization', link: '/reference/binarization/' },
                                                { label: 'Drawing API', link: '/reference/drawing/' },
                                                { label: 'Pipeline functions', link: '/reference/pipeline/' },
                                                { label: 'Macros', link: '/reference/macros/' },
                                                { label: 'Errors', link: '/reference/errors/' },
                                                { label: 'CLI', link: '/reference/cli/' },
                                                { label: 'Feature flags', link: '/reference/feature-flags/' },
                                        ],
                                },
                                {
                                        label: 'Concepts',
                                        items: [
                                                { label: 'The extraction pipeline', link: '/explanation/extraction-pipeline/' },
                                                { label: 'Performance', link: '/explanation/performance/' },
                                        ],
                                },
                        ],
                }),
        ],
});
