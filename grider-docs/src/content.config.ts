import { defineCollection } from 'astro:content';
import { docsLoader, i18nLoader } from '@astrojs/starlight/loaders';
import { docsSchema, i18nSchema } from '@astrojs/starlight/schema';
import { z } from 'astro/zod';

export const collections = {
        docs: defineCollection({
                loader: docsLoader(),
                schema: docsSchema({
                        extend: z.object({
                                // Diátaxis information type of the page. Omitted only on the
                                // landing page and the 404 page, which are navigation/utility
                                // pages, not Diátaxis units.
                                type: z
                                        .enum(['tutorial', 'how-to', 'reference', 'explanation'])
                                        .optional(),
                        }),
                }),
        }),
        i18n: defineCollection({
                loader: i18nLoader(),
                schema: i18nSchema(),
        }),
};
