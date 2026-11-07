// Search modal probe: open Pagefind search, type a domain term, report results.
import { chromium } from "playwright";

const BASE = process.env.QA_BASE_URL ?? "http://localhost:4321";
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
await page.goto(BASE + "/guides/choose-binarization/", { waitUntil: "networkidle" });

const openBtn = page.locator("button[aria-label='Search'], button[aria-keyshortcuts='Control+k'], starlight-search-button button").first();
await openBtn.click();
await page.keyboard.type("Sauvola", { delay: 80 });
await page.waitForTimeout(1200);

const results = page.locator("[class*='search-results'] a, [role='option'], dialog a[href]");
const count = await results.count();
const titles = [];
for (let i = 0; i < Math.min(count, 5); i++) {
	const text = (await results.nth(i).innerText()).replace(/\s+/g, " ").trim();
	titles.push(text.slice(0, 90));
}
console.log(JSON.stringify({ query: "Sauvola", resultCount: count, sample: titles }, null, 2));

// second probe
await page.keyboard.press("Control+a");
await page.keyboard.type("filter_biggest_rows", { delay: 80 });
await page.waitForTimeout(1200);
const count2 = await page.locator("[class*='search-results'] a, [role='option'], dialog a[href]").count();
console.log(JSON.stringify({ query: "filter_biggest_rows", resultCount: count2 }));

await browser.close();
if (count === 0 || count2 === 0) {
	console.error("SEARCH PROBE FAILED — zero results");
	process.exit(1);
}
console.log("SEARCH PROBE PASSED");
