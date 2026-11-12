import { defineDocsConfig } from '@elcoosp/docs-toolkit/config';
import starlightImageZoom from 'starlight-image-zoom';
import starlightHeadingBadges from 'starlight-heading-badges';
import starlightLlmsTxt from 'starlight-llms-txt';
export default await defineDocsConfig({
  llmsTxt: {},
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
                        // The docs collection provides src/content/docs/404.md; the built-in
                        // dedicated /404 route is disabled so the two do not conflict.
                        disable404Route: true,
				// `root` marks the unprefixed default locale in current Starlight;
				// a key named after the language would create a /en/ prefixed locale.
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
                                        label: 'GitHub', translations: { fr: "GitHub" },
                                        href: 'https://github.com/elcoosp/grider'
                                },
                        ],
                        editLink: {
                                baseUrl: 'https://github.com/elcoosp/grider/edit/main/docs-site/'
                        },
                        plugins: [
                                // Fails the build on broken internal links (release gate).
                                
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
                                        label: 'Start here', translations: { fr: "Commencer ici" },
                                        items: [
                                                { label: 'Overview', translations: { fr: 'Aperçu' }, link: '/' },
                                                { label: 'Installation', translations: { fr: "Installation" }, link: '/guides/install/' },
                                                {
                                                        label: 'Extract your first grid', translations: { fr: "Extraire votre première grille" },
                                                        link: '/tutorials/extract-your-first-grid/'
                                                },
                                        ]
                                },
                                {
                                        label: 'How-to guides', translations: { fr: 'Guides pratiques' },
                                        items: [
                                                { label: 'Configure extraction', translations: { fr: "Configurer l'extraction" }, link: '/guides/configure-extraction/' },
                                                {
                                                        label: 'Choose a binarization method', translations: { fr: "Choisir une méthode de binarisation" },
                                                        link: '/guides/choose-binarization/'
                                                },
                                                { label: 'Filter grid lines', translations: { fr: "Filtrer les lignes de la grille" }, link: '/guides/filter-lines/' },
                                                { label: 'Draw grids and cells', translations: { fr: "Dessiner des grilles et des cellules" }, link: '/guides/draw-overlays/' },
                                                { label: 'Serialize grids to JSON', translations: { fr: "Sérialiser les grilles en JSON" }, link: '/guides/serialize-grids/' },
                                                { label: 'Use the CLI', translations: { fr: "Utiliser la CLI" }, link: '/guides/use-the-cli/' },
                                        ]
                                },
                                {
                                        label: 'Reference', translations: { fr: 'Référence' },
                                        items: [
                                                { label: 'Grid', translations: { fr: "Grille" }, link: '/reference/grid/' },
                                                { label: 'Lines and cells', translations: { fr: "Lignes et cellules" }, link: '/reference/lines-and-cells/' },
                                                { label: 'GridConfig', translations: { fr: "GridConfig" }, link: '/reference/grid-config/' },
                                                { label: 'Binarization', translations: { fr: "Binarisation" }, link: '/reference/binarization/' },
                                                { label: 'Drawing API', translations: { fr: "API de dessin" }, link: '/reference/drawing/' },
                                                { label: 'Pipeline functions', translations: { fr: "Fonctions du pipeline" }, link: '/reference/pipeline/' },
                                                { label: 'Macros', translations: { fr: "Macros" }, link: '/reference/macros/' },
                                                { label: 'Errors', translations: { fr: "Erreurs" }, link: '/reference/errors/' },
                                                { label: 'CLI', translations: { fr: "CLI" }, link: '/reference/cli/' },
                                                { label: 'Feature flags', translations: { fr: "Drapeaux de fonctionnalités" }, link: '/reference/feature-flags/' },
                                        ]
                                },
                                {
                                        label: 'Concepts', translations: { fr: "Concepts" },
                                        items: [
                                                { label: 'The extraction pipeline', translations: { fr: "Le pipeline d'extraction" }, link: '/explanation/extraction-pipeline/' },
                                                { label: 'Performance', translations: { fr: "Performances" }, link: '/explanation/performance/' },
                                        ]
                                },
                        ]
                },
});
