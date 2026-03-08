# Pitfalls Research

**Domain:** Garden seed management app with web scraping (Botanical Interests / Shopify store)
**Researched:** 2026-03-08
**Confidence:** HIGH

## Critical Pitfalls

### Pitfall 1: Planting Data Is NOT in the Shopify Product JSON

**What goes wrong:**
Botanical Interests runs on Shopify (Dawn theme). The obvious scraping approach -- hitting `/products/{handle}.json` -- returns only commerce data (title, price, SKU, description HTML, images). The critical planting information (days to emerge, sow depth, spacing, when to start indoors relative to frost date, soil temperature ranges) is NOT in the product JSON. It is either rendered into the page HTML via Shopify Liquid templates pulling from metafields, or loaded dynamically via JavaScript after page render. The `/products.json` bulk endpoint also lacks metafields entirely.

**Why it happens:**
Shopify's public JSON endpoints intentionally exclude metafields unless they are exposed to the Storefront API with `PUBLIC_READ` access -- which is controlled by the store owner, not the scraper. Developers assume the JSON endpoint is the complete data source and build their scraper around it, then discover the planting data they actually need is missing.

**How to avoid:**
- Scrape the full rendered HTML page, not just the JSON endpoint. Use the HTML page as the primary data source.
- Inspect actual product pages in a browser to identify where planting data appears (likely in expandable tabs, "growing info" sections, or inside the `body_html` description field).
- As a fallback, check if planting data is embedded in `<script>` tags as Shopify metafield data or in JSON-LD structured data.
- Design the scraper to extract from multiple possible locations (description HTML, dedicated sections, script tags) since the template could change.
- Consider also checking the `.json` endpoint's `body_html` field -- Botanical Interests may embed some growing info in the product description itself, even if not in structured fields.

**Warning signs:**
- Scraper tests return product titles and prices but no planting dates or growing instructions.
- `body_html` field contains only marketing copy, not structured planting data.
- Pages look data-rich in the browser but the JSON is sparse.

**Phase to address:**
Phase 1 (Scraping) -- this must be validated before any other feature depends on scraped data. Build a proof-of-concept scraper for a single product page and verify all required fields are extractable before designing the data model.

---

### Pitfall 2: Planting Data Is Unstructured Text, Not Machine-Readable Fields

**What goes wrong:**
Even when you find the planting data on the page, it is typically free-form text like "Sow seeds indoors 6-8 weeks before last frost" or "Days to emerge: 7-14" embedded in paragraphs or semi-structured HTML. Different product pages may format this differently. Parsing "6-8 weeks before last frost" into a computable date offset requires natural language understanding that breaks when the wording varies ("Start indoors 6 to 8 weeks prior to transplanting outdoors after danger of frost has passed").

**Why it happens:**
Seed companies write planting instructions for humans reading physical seed packets, not for machines. Botanical Interests explicitly designs their packets with descriptive text (per their "How to Read a Seed Packet" guide). There is no standardized schema for seed planting data across the industry.

**How to avoid:**
- Catalog the actual text patterns used across 10-20 real Botanical Interests product pages before writing the parser. Build the parser to handle observed patterns, not assumed ones.
- Use regex patterns with fallback strategies, not a single rigid parser. Example patterns to match: "X-Y weeks before last frost", "Direct sow after last frost", "Days to emerge: X-Y".
- Store the raw scraped text alongside parsed values so you can re-parse later when you discover new patterns.
- Accept that some products may need manual data entry for fields the parser cannot handle. Design the UI to allow manual override/correction of scraped data.

**Warning signs:**
- Parser works for tomatoes but fails for herbs or flowers because the wording is different.
- Increasing number of "null" parsed fields as you scrape more products.
- Schedule generation produces nonsensical dates for certain crops.

**Phase to address:**
Phase 1 (Scraping) and Phase 2 (Data Model). The data model must accommodate both parsed and raw text. The scraper should be tested against diverse product types early.

---

### Pitfall 3: Hardcoded Frost Dates Produce Wrong Planting Schedules

**What goes wrong:**
The app hardcodes "last frost ~May 10" for Halifax, MA, but this is an average. In practice, last frost can vary by 2-3 weeks in either direction year to year. Microclimates (proximity to the coast, elevation, urban heat islands) add further variance. A schedule that says "start tomato seeds indoors March 1" based on a May 10 frost date could be two weeks early or late in any given year. Users who follow the schedule blindly may lose seedlings to late frost or miss optimal planting windows.

**Why it happens:**
Developers treat frost dates as precise values when they are statistical averages. The "last frost date" is defined as the date with <50% probability of frost, meaning frost occurs after that date roughly half the time. Zone 6b covers a wide temperature range (-5F to 0F minimum), and Halifax MA's coastal proximity adds additional complexity.

**How to avoid:**
- Display frost dates as ranges, not single dates. Show "Start indoors: Feb 25 - Mar 15" rather than "Start indoors: Mar 1".
- Make the frost date user-configurable even if defaulting to Halifax, MA. A simple settings field costs almost nothing and prevents the schedule from being silently wrong.
- Add a disclaimer that dates are estimates based on average frost dates.
- Consider pulling frost date data from the Old Farmer's Almanac API or NOAA data rather than hardcoding.

**Warning signs:**
- Users ask "why does the app say X date?" because it does not match their experience.
- Schedule shows single precise dates with no indication of uncertainty.
- No way to adjust the base frost date.

**Phase to address:**
Phase 3 (Schedule Generation). Display uncertainty from day one. Even with a hardcoded location, show date ranges.

---

### Pitfall 4: Seed Viability Lookup Table Oversimplifies a Complex Problem

**What goes wrong:**
Viability varies significantly by species, but also by storage conditions (temperature, humidity, light exposure). A simple "tomato seeds last 4 years" lookup table gives false confidence. Seeds stored in a hot garage degrade much faster than seeds stored in a cool, dark place below 50F. The app has no way to know storage conditions, so the viability estimate could be wildly inaccurate. Users may throw away perfectly good seeds or plant seeds with near-zero germination rates.

Published viability data itself varies between sources:
- High Mowing Seeds says tomatoes: 4 years
- Some extension services say tomatoes: 3-5 years
- Actual viability depends on initial seed quality, which varies by lot

**Why it happens:**
Developers want a clean lookup table but seed viability is inherently probabilistic and condition-dependent. There is no single authoritative source -- extension services, seed companies, and academic sources all give slightly different numbers.

**How to avoid:**
- Frame viability as "estimated viability under ideal storage" with a clear caveat about storage conditions.
- Use conservative estimates (shorter end of published ranges) as defaults.
- Show viability as a gradient (excellent / good / declining / poor) rather than a binary (viable / not viable).
- Use data from university extension services (Iowa State, Illinois Extension, University of Nebraska) as they are the most research-backed. Compile from multiple sources and use the conservative consensus.
- Include a "test germination" recommendation for seeds past their estimated viability window rather than a hard "expired" label.

**Warning signs:**
- Users report that seeds marked "low viability" germinated fine, or seeds marked "good" failed.
- The viability table has a single year cutoff with no nuance.
- No disclaimer about storage conditions affecting viability.

**Phase to address:**
Phase 2 (Data Model) for the lookup table structure. Phase 3 (Schedule/UI) for how viability is communicated to the user.

---

### Pitfall 5: Scraper Breaks Silently When Botanical Interests Updates Their Shopify Theme

**What goes wrong:**
Shopify stores update themes regularly. Botanical Interests currently uses the Dawn theme v13.0.1. When they update their theme, CSS classes, HTML structure, and content layout can change without warning. The scraper continues to run but returns empty or incorrect data because selectors no longer match. Since this is on-demand scraping (not bulk), the breakage may not be noticed until a user tries to add a new seed and gets garbage data.

**Why it happens:**
Web scraping is inherently fragile. Shopify themes use generated class names and semantic HTML that can change between versions. There is no contract between the website and the scraper.

**How to avoid:**
- Validate scraped data at parse time. If required fields (days to emerge, sow depth, plant spacing) come back empty, flag the scrape as potentially broken rather than silently storing incomplete data.
- Store raw HTML alongside parsed data so you can re-parse after fixing the scraper without re-fetching (which also reduces load on their server).
- Build the scraper with multiple extraction strategies (CSS selectors, regex on text content, JSON-LD) so if one breaks, others may still work.
- Add a "last successful full scrape" timestamp per product and surface warnings for products with incomplete data.
- Keep scraper selectors in a configuration file, not hardcoded in Rust, so they can be updated without recompiling (though for a single-user app, recompiling is acceptable).

**Warning signs:**
- Scraped products suddenly have missing fields that previously worked.
- New products scrape differently than older cached ones.
- HTML structure in stored raw data looks different from what the scraper expects.

**Phase to address:**
Phase 1 (Scraping). Build validation into the scraper from the start. Never silently accept incomplete data.

---

### Pitfall 6: Not Respecting Botanical Interests' Scraping Policies

**What goes wrong:**
Botanical Interests' robots.txt (standard Shopify) blocks scraping of `/search`, sorted collections, and several other paths. While individual product pages are not blocked, aggressive scraping (fast request rates, no user-agent identification) could get the IP blocked. For a single-user app doing on-demand scraping, this is low risk but not zero risk. Shopify implements rate limiting and bot detection at the platform level.

**Why it happens:**
Developers focus on getting the scraper working and ignore rate limiting and robots.txt compliance. Shopify's anti-bot measures have become more sophisticated over time.

**How to avoid:**
- Respect robots.txt -- only scrape allowed paths (individual product pages are allowed).
- Set a reasonable User-Agent string identifying the app (not pretending to be a browser).
- Add delays between requests (at least 1-2 seconds) even though on-demand scraping is inherently slow.
- Cache aggressively -- seed packet data does not change frequently. Once scraped, a product should not need re-scraping for months.
- Never scrape checkout, cart, or search endpoints.

**Warning signs:**
- HTTP 429 (Too Many Requests) responses.
- HTTP 403 (Forbidden) responses.
- CAPTCHAs appearing in responses.

**Phase to address:**
Phase 1 (Scraping). Build rate limiting and caching into the scraper architecture from the start.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Hardcode CSS selectors in Rust code | Faster initial development | Recompile required when selectors change | MVP only -- extract to config before adding more products |
| Store only parsed data, not raw HTML | Simpler data model | Cannot re-parse when parser improves; must re-scrape | Never -- always store raw HTML |
| Single frost date instead of range | Simpler schedule math | Misleading precision, users make bad planting decisions | Never -- show ranges from day one |
| Skip viability estimates initially | Ship faster | Users have no guidance on whether old seeds are worth planting | Acceptable for MVP if clearly marked as "coming soon" |
| Use blocking HTTP client | Simpler code, no async runtime | Freezes UI during scraping | Acceptable for single-user app, but add loading state |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Botanical Interests (Shopify) | Relying on `/products.json` for planting data | Scrape full rendered HTML page; JSON lacks metafield data |
| Botanical Interests (Shopify) | Assuming all products have identical HTML structure | Test scraper against vegetables, herbs, and flowers -- each category may have different page layouts |
| Shopify platform | Not handling redirects (product URL changes, slug updates) | Follow redirects and update stored URLs; Shopify sometimes changes product handles |
| SQLite | Using TEXT for dates instead of proper date types | Use ISO 8601 strings or Julian day numbers; SQLite has no native date type but has date functions that work with ISO strings |
| Image downloads | Downloading full-resolution images from Shopify CDN | Use Shopify's image transform URLs (append `_300x300` etc.) to get appropriately sized images |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Scraping on every page load | Slow page loads (2-5s per product) | Cache scraped data in SQLite; only re-scrape on explicit user action | First product add |
| Loading all seed images at once | Slow inventory page, high memory | Lazy load images; store thumbnails locally | 20+ seeds in inventory |
| Full DOM parse for simple data | Scraper uses 10x more memory than needed | Use targeted CSS selectors, not full document traversal | Not a real concern at this scale |
| SQLite write locks during scraping | UI freezes while writing scraped data | Use WAL mode for SQLite; scrape in background | Concurrent reads during writes |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Storing scraped HTML without sanitization | XSS if raw HTML is rendered in Maud templates | Sanitize or escape all scraped content before display; Maud escapes by default, but be careful with `PreEscaped` |
| Following arbitrary redirects during scraping | SSRF if scraper follows redirects to internal network | Validate redirect URLs stay on botanicalinterests.com domain |
| Exposing SQLite database file via web server | Data leak of seed inventory | Ensure static file serving does not serve the `.db` file |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Showing viability as binary "viable/expired" | User throws away good seeds or plants dead ones | Show gradient: Excellent (1yr) / Good (2yr) / Declining (3yr) / Test First (4yr+) |
| Planting schedule as a flat list of dates | Overwhelming; user does not know what to do today | Group by time period: "This Week", "Next Week", "This Month"; highlight overdue items |
| No feedback during scraping | User clicks "Add Seed" and nothing happens for 3-5 seconds | Show loading indicator; display partial data as it arrives |
| Requiring exact product URL to add seeds | User has to navigate Botanical Interests site to copy URLs | Allow search by seed name; provide autocomplete from previously scraped data |
| Showing all seeds in inventory equally | User wastes time scrolling past seeds they are not growing this year | Separate "Inventory" (everything owned) from "This Season" (what they are actively growing) |
| Calendar view without actionable context | User sees a date but not what to do | Each calendar entry should say what action to take: "Start Cherokee Purple Tomato seeds indoors in seed trays" |

## "Looks Done But Isn't" Checklist

- [ ] **Scraper:** Tested against at least 3 product categories (vegetable, herb, flower) -- different categories may have different page structures
- [ ] **Scraper:** Handles products with no planting data (bundles, supplies, gift cards) gracefully -- does not crash or store garbage
- [ ] **Viability table:** Covers all common vegetable families, not just the 5 most popular crops
- [ ] **Schedule generation:** Handles crops with "direct sow only" instructions (no indoor start date) -- not everything starts indoors
- [ ] **Schedule generation:** Handles crops with multiple planting windows (succession planting, fall planting)
- [ ] **Schedule generation:** Distinguishes "weeks before last frost" from "weeks after last frost" -- sign error produces completely wrong dates
- [ ] **Schedule generation:** Handles cool-season crops that should be planted BEFORE last frost (lettuce, peas, spinach)
- [ ] **Data model:** Stores both parsed and raw scraped data for every product
- [ ] **Data model:** Tracks purchase year per seed lot, not just per species (user may buy same seed in 2024 and 2026)
- [ ] **UI:** Displays uncertainty in dates (ranges, not single dates)

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Scraper breaks due to site redesign | LOW | Fix selectors; re-parse stored raw HTML; no re-scraping needed if raw HTML was saved |
| Scraper breaks, no raw HTML stored | MEDIUM | Must re-scrape all products; rate-limit to avoid blocking; 1-2 days of work |
| Wrong frost date used in schedule | LOW | Update frost date constant; recalculate all schedules (automatic if dates are derived, not stored) |
| Viability table has wrong data | LOW | Update lookup table; viability is display-only, no downstream data corruption |
| Planting data parser misses a pattern | LOW | Add new regex pattern; re-parse stored raw text; update affected schedules |
| Shopify blocks IP | MEDIUM | Wait 24 hours; add proper User-Agent and rate limiting; use different network if needed |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Planting data not in JSON endpoint | Phase 1 (Scraping POC) | Verify all required fields extracted from at least 5 real product pages |
| Unstructured planting text | Phase 1 (Scraping) + Phase 2 (Data Model) | Parser handles 90%+ of products in user's actual seed inventory |
| Hardcoded frost dates | Phase 3 (Schedule Generation) | Schedule shows date ranges, not single dates; frost date is configurable |
| Viability oversimplification | Phase 2 (Data Model) + Phase 3 (UI) | Viability shown as gradient with storage caveat; data from 2+ extension sources |
| Scraper breaks silently | Phase 1 (Scraping) | Scraper validates completeness; incomplete scrapes are flagged in UI |
| Scraping policy violations | Phase 1 (Scraping) | Rate limiting, caching, proper User-Agent all implemented from first scrape |
| Sign error in frost date math | Phase 3 (Schedule Generation) | Unit tests for "before frost" vs "after frost" crops; test cool-season and warm-season crops |

## Sources

- [Botanical Interests - How to Read a Seed Packet](https://www.botanicalinterests.com/pages/how-to-read-a-seed-packet)
- [High Mowing Seeds - Seed Viability Chart](https://www.highmowingseeds.com/blog/seed-viability-chart/)
- [Iowa State Extension - Seed Storage and Germination Rates](https://yardandgarden.extension.iastate.edu/how-to/how-store-seeds-and-test-germination-rates)
- [University of Nebraska Extension - Seed Storage](https://go.unl.edu/seedstorage)
- [USDA Plant Hardiness Zone Map (2023 update)](https://planthardiness.ars.usda.gov/)
- [Old Farmer's Almanac - Frost Dates](https://www.almanac.com/gardening/frostdates)
- [Shopify - Product Metafields Documentation](https://shopify.dev/docs/apps/build/custom-data)
- [Shopify - products.json scraping](https://dev.to/dentedlogic/the-shopify-productsjson-trick-scrape-any-store-25x-faster-with-python-4p95)
- [WebScraping.AI - Reqwest Pitfalls](https://webscraping.ai/faq/reqwest/what-are-the-common-pitfalls-when-using-reqwest-for-web-scraping)
- [ScrapingBee - Web Scraping Best Practices](https://www.scrapingbee.com/blog/web-scraping-best-practices/)
- [USDA Climate Hubs - Shifts in Growing Zones](https://www.climatehubs.usda.gov/hubs/topic/shifts-growing-degree-days-plant-hardiness-zones-and-heat-zones)

---
*Pitfalls research for: Garden seed management app (Botanical Interests scraper, SQLite, zone 6b scheduling)*
*Researched: 2026-03-08*
