import { defineDocsConfig } from '@elcoosp/docs-toolkit/config';
import starlightLinksValidator from 'starlight-links-validator';
import starlightImageZoom from 'starlight-image-zoom';
import starlightHeadingBadges from 'starlight-heading-badges';
import starlightLlmsTxt from 'starlight-llms-txt';


export default await defineDocsConfig({
  changelog: { repo: 'elcoosp/grider' },
  site: 'https://grider-docs.vercel.app',
  domain: 'grider-docs.vercel.app',
  title: "Grider",
  description: "High-performance row and column grid extraction from images, as a Rust library and a CLI.",
  sidebar: [],
  extraStarlight: {
                        expressiveCode: {
                        	themes: ['github-light', 'github-dark']
                        },
                        title: 'Grider',
                        description:
                                'High-performance row and column grid extraction from images, as a Rust library and a CLI.',
                        favicon: '/favicon.png',
                        defaultLocale: 'en',
                        // The docs collection provides src/content/docs/404.md; the built-in
                        // dedicated /404 route is disabled so the two do not conflict.
                        disable404Route: true,
                        locales: {
				// `root` marks the unprefixed default locale in current Starlight;
				// a key named after the language would create a /en/ prefixed locale.
				root: { label: 'English', lang: 'en' }
			},
                        head: [
                                {
                                        tag: 'link',
                                        attrs: { rel: 'icon', type: 'image/png', href: '/favicon.png' }
                                },
                                {
                                        tag: 'link',
                                        attrs: { rel: 'apple-touch-icon', href: '/apple-touch-icon.png' }
                                },
                        ],
                        social: [
                                {
                                        icon: 'github',
                                        label: 'GitHub',
                                        href: 'https://github.com/elcoosp/grider'
                                },
                        ],
                        editLink: {
                                baseUrl: 'https://github.com/elcoosp/grider/edit/main/docs-site/'
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
                                                'Documentation for Grider, a high-performance Rust library and CLI that extracts the row and column grid of an image.'
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
                                                        link: '/tutorials/extract-your-first-grid/'
                                                },
                                        ]
                                },
                                {
                                        label: 'How-to guides',
                                        items: [
                                                { label: 'Configure extraction', link: '/guides/configure-extraction/' },
                                                {
                                                        label: 'Choose a binarization method',
                                                        link: '/guides/choose-binarization/'
                                                },
                                                { label: 'Filter grid lines', link: '/guides/filter-lines/' },
                                                { label: 'Draw grids and cells', link: '/guides/draw-overlays/' },
                                                { label: 'Serialize grids to JSON', link: '/guides/serialize-grids/' },
                                                { label: 'Use the CLI', link: '/guides/use-the-cli/' },
                                        ]
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
                                        ]
                                },
                                {
                                        label: 'Concepts',
                                        items: [
                                                { label: 'The extraction pipeline', link: '/explanation/extraction-pipeline/' },
                                                { label: 'Performance', link: '/explanation/performance/' },
                                        ]
                                },
                        ]
                },
});
