# Feature Research

**Domain:** Garden seed inventory management and planting scheduler
**Researched:** 2026-03-08
**Confidence:** MEDIUM-HIGH

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Seed inventory list | Core purpose of the app -- users need to see what seeds they own | LOW | Basic CRUD on seed records with species, variety, source, purchase year |
| Add seed by Botanical Interests product | The app's primary data entry method per PROJECT.md | MEDIUM | Requires scraping BI product pages; must handle page structure changes |
| Seed detail view with packet info | Users expect to see the data from their seed packet (days to maturity, sowing depth, spacing, light) without digging out the physical packet | LOW | Display scraped data; images from BI product page |
| Purchase year tracking | Seeds degrade over time; users need to know when they bought each packet | LOW | Simple date field per inventory record |
| Germination viability estimate | Core differentiator in PROJECT.md but also table stakes for a seed *management* app -- without this it's just a list | MEDIUM | Lookup table by species family; viability curves from extension service data (e.g., onions 1-2 yr, tomatoes 4-5 yr, squash 5+ yr) |
| Season planting schedule | Core value prop: tell me when to start seeds indoors and transplant outdoors | MEDIUM | Based on Halifax MA last frost ~May 10, first frost ~Oct 15; calculate from BI packet data (weeks before last frost, days to germination) |
| Action list / task view | Users need a "what do I do this week" view, not just a calendar | LOW | Filter/sort schedule by date range; show upcoming actions |
| Calendar / timeline view | Visual representation of planting schedule across the season | MEDIUM | Timeline showing seed-start, transplant, and harvest windows per crop |
| Select seeds to grow this season | Users have inventory across years but only grow a subset each season | LOW | Boolean flag or season association on inventory items |
| Offline access / local data | Single-user local app; data must persist without internet after initial scrape | LOW | SQLite storage per PROJECT.md; scrape results cached locally |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valuable.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Automatic viability degradation warnings | No competitor does this well. Show "use this year or toss" alerts for seeds nearing end of viability. Competitors track inventory but don't warn about aging seeds | LOW | Compare purchase year + species viability curve; flag seeds below 50% expected germination |
| Scrape-and-store from Botanical Interests URL | Paste a URL, get all packet data automatically. Competitors require manual data entry or have generic databases that don't match your actual seed packets | MEDIUM | On-demand scraper for BI product pages; store structured data + images locally |
| "What should I start this week" notifications | Seedtime has task lists but users complain about imprecise 2-week windows. Specific date-based action items tied to frost dates are better | LOW | Computed from schedule; filter to current/upcoming week |
| Seed age at a glance in inventory | Color-coded or visual indicator showing viability status (fresh / good / declining / expired) across entire inventory | LOW | Simple UI enhancement on inventory list; green/yellow/orange/red based on viability estimate |
| Single binary, zero config | Unlike Seedtime (cloud SaaS) or spreadsheets, this is a self-contained local app. No account, no subscription, no data leaving your machine | LOW | Architectural advantage from Rust + SQLite + Maud stack; not a feature to build but to preserve |
| Planting schedule derived from actual packet data | Most apps use generic crop databases. This app uses the specific data from your actual seed packets (scraped from BI), which may differ by variety | MEDIUM | Parse BI-specific fields: days to maturity, indoor start timing, transplant timing |
| Over-sow recommendation for old seeds | If germination rate is estimated at 60%, suggest planting more seeds to compensate | LOW | Simple math: desired plants / estimated germination rate = seeds to sow |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Multi-supplier seed catalog | Users buy from many sources | Scraping multiple sites is a maintenance nightmare; each site has different data formats; massively increases scope | Start with Botanical Interests only (per PROJECT.md). Allow manual seed entry for other sources with basic fields (species, variety, days to maturity) |
| Garden bed layout planner | Popular in competitors like GrowVeg | Completely different product category (spatial planning vs temporal scheduling); huge complexity for companion planting, spacing, bed dimensions | Stay focused on *when* to plant, not *where*. Link out to existing garden planners if needed |
| Harvest tracking / garden journal | Seedtime and others offer this | Scope creep into journaling app; per PROJECT.md out of scope; splits focus from core scheduling value | Keep scope to pre-harvest: inventory + schedule. Users can journal elsewhere |
| Weather API integration | Real-time frost alerts seem useful | Adds external dependency, API costs, complexity; for a single hardcoded location the frost dates are well-known and stable | Use static frost date data for Halifax MA. User can manually adjust if needed |
| Social / sharing features | Community seed swaps are popular | Single-user app; authentication, networking, moderation are massive scope | Out of scope per PROJECT.md |
| Full catalog pre-scrape | Having all BI products pre-loaded sounds convenient | Bulk scraping is fragile, potentially violates ToS, creates stale data, wastes storage | On-demand scraping per PROJECT.md. Only fetch what user actually owns |
| Multi-location support | Users may have multiple garden plots | Adds location management complexity; per PROJECT.md hardcoded to Halifax MA | Hardcode zone 6b. If needed later, make frost dates configurable rather than full location system |
| Succession planting scheduler | Stagger plantings for continuous harvest | Significant scheduling complexity; multiple planting dates per crop; complicates calendar | Defer to v2. If needed, let users manually add the same seed to their season list multiple times with different target dates |
| Purchase / e-commerce integration | "Buy more of this seed" links | Per PROJECT.md out of scope; affiliate links add commercial complexity | At most, link back to the BI product URL already stored from scraping |

## Feature Dependencies

```
[Seed Scraper (BI product pages)]
    |
    v
[Seed Inventory (CRUD)]
    |
    +----> [Viability Estimation] ----> [Viability Warnings]
    |                                       |
    |                                       v
    |                              [Over-sow Recommendations]
    |
    +----> [Season Selection ("grow this year")]
               |
               v
          [Planting Schedule Engine]
               |
               +----> [Calendar/Timeline View]
               |
               +----> [Action List / Task View]
               |
               +----> ["This Week" View]
```

### Dependency Notes

- **Seed Inventory requires Scraper:** Inventory entries are created by scraping BI product pages. The scraper must work before inventory is useful (though manual entry could be a fallback).
- **Viability Estimation requires Inventory:** Needs species + purchase year from inventory records to calculate estimated germination rate.
- **Planting Schedule requires Season Selection + Inventory:** Must know which seeds the user wants to grow this season, and must have the planting data (days to maturity, weeks before frost) from scraped packet info.
- **All schedule views require Schedule Engine:** Calendar, action list, and "this week" views are different presentations of the same computed schedule data.
- **Over-sow Recommendations enhance Viability:** Optional layer on top of viability estimation that suggests planting quantities.

## MVP Definition

### Launch With (v1)

Minimum viable product -- what's needed to validate the concept.

- [ ] Seed scraper for Botanical Interests product pages -- extracts variety name, species, days to maturity, planting instructions, seed count, images
- [ ] Seed inventory CRUD -- add by BI URL, view all seeds, edit, delete
- [ ] Purchase year tracking with viability estimate display -- show estimated germination % based on species and age
- [ ] Season selection -- mark which seeds to grow this year
- [ ] Planting schedule generation -- compute start-indoors and transplant-outdoors dates from packet data + Halifax MA frost dates
- [ ] Action list view -- sorted list of upcoming planting actions with dates
- [ ] Basic calendar/timeline view -- visual display of the season schedule

### Add After Validation (v1.x)

Features to add once core is working.

- [ ] Viability warnings -- proactive alerts for seeds nearing end of useful life ("use or lose" notifications)
- [ ] Over-sow recommendations -- "plant 8 seeds instead of 5 to account for reduced germination"
- [ ] "This week" focused view -- filter schedule to just current and upcoming week's actions
- [ ] Seed age visual indicators -- color-coded inventory list showing viability at a glance (green/yellow/orange/red)
- [ ] Manual seed entry -- add seeds from non-BI sources with manual data entry for key fields
- [ ] Print-friendly schedule -- printable version of the planting schedule to post in the garden shed

### Future Consideration (v2+)

Features to defer until product-market fit is established.

- [ ] Succession planting support -- multiple planting dates per crop for continuous harvest
- [ ] Configurable frost dates -- allow adjusting last/first frost dates instead of hardcoded Halifax MA
- [ ] Seed quantity tracking -- track how many seeds remain in each packet after planting
- [ ] Historical schedule comparison -- "what did I plant last year and when"
- [ ] Scraper resilience improvements -- handle BI site redesigns, fallback to manual entry

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| BI product page scraper | HIGH | HIGH | P1 |
| Seed inventory CRUD | HIGH | LOW | P1 |
| Purchase year + viability estimate | HIGH | MEDIUM | P1 |
| Season seed selection | HIGH | LOW | P1 |
| Planting schedule engine | HIGH | MEDIUM | P1 |
| Action list view | HIGH | LOW | P1 |
| Calendar/timeline view | MEDIUM | MEDIUM | P1 |
| Viability warnings | MEDIUM | LOW | P2 |
| Over-sow recommendations | MEDIUM | LOW | P2 |
| "This week" view | MEDIUM | LOW | P2 |
| Visual viability indicators | LOW | LOW | P2 |
| Manual seed entry (non-BI) | MEDIUM | LOW | P2 |
| Print-friendly schedule | LOW | LOW | P2 |
| Succession planting | MEDIUM | HIGH | P3 |
| Configurable frost dates | LOW | LOW | P3 |
| Seed quantity tracking | LOW | MEDIUM | P3 |
| Historical comparison | LOW | MEDIUM | P3 |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

## Competitor Feature Analysis

| Feature | Seedtime | GrowVeg | Smart Gardener | Spreadsheets | Seeds App (Ours) |
|---------|----------|---------|----------------|--------------|-----------------|
| Seed inventory tracking | No (calendar-focused) | No | Limited | Manual, flexible | Yes, auto-populated from BI |
| Planting calendar | Yes, 2,670+ varieties | Yes | Yes | Manual | Yes, from actual packet data |
| Location-based frost dates | Yes, by zip code | Yes | Yes | Manual lookup | Hardcoded Halifax MA |
| Seed viability/age tracking | No | No | No | Manual if you think of it | Yes, automatic by species |
| Data entry method | Select from database | Select from database | Select from database | Type everything | Paste BI URL, auto-scrape |
| Offline capability | No (cloud SaaS) | No (cloud SaaS) | No (cloud SaaS) | Yes (local file) | Yes (local SQLite) |
| Cost | Free tier + paid | Paid subscription | Free tier + paid | Free | Free (self-hosted) |
| Garden layout | No | Yes (primary feature) | Yes | No | No (out of scope) |
| Journal/notes | Yes | Yes | Limited | Manual | No (out of scope) |
| Task list | Yes (auto-generated) | Limited | Limited | Manual | Yes (from schedule) |

### Key Competitive Insights

1. **No competitor combines seed inventory management with planting scheduling.** Seedtime does scheduling but not inventory. Spreadsheet users track inventory but calculate schedules manually. This is the gap.

2. **No competitor tracks seed viability.** This is a genuine unmet need. Gardeners know old seeds germinate poorly but have no tool to estimate viability systematically.

3. **Auto-populating from a seed company URL is novel.** Every competitor requires manual selection from a generic database or manual data entry. Scraping actual product data means variety-specific accuracy.

4. **Offline-first local app is underserved.** All major competitors are cloud SaaS. Privacy-conscious gardeners and those with poor internet have no good option.

## Viability Estimation Reference Data

Key input for the viability feature. Species-level germination viability by seed age (years since purchase, stored in cool/dry conditions):

| Species Family | 1 yr | 2 yr | 3 yr | 4 yr | 5 yr | 6+ yr |
|---------------|------|------|------|------|------|-------|
| Onion, Leek, Chive | Good | Poor | Expired | - | - | - |
| Parsnip, Parsley | Good | Poor | Expired | - | - | - |
| Corn, Sweet | Good | Good | Poor | Expired | - | - |
| Pepper | Good | Good | Good | Poor | Expired | - |
| Bean, Pea | Good | Good | Good | Poor | Expired | - |
| Carrot, Lettuce | Good | Good | Good | Poor | Expired | - |
| Tomato | Good | Good | Good | Good | Poor | Expired |
| Cucumber, Melon, Squash | Good | Good | Good | Good | Good | Poor |
| Brassica (Cabbage, Broccoli) | Good | Good | Good | Good | Poor | Expired |
| Radish | Good | Good | Good | Good | Poor | Expired |

**Confidence:** MEDIUM -- These are approximations from university extension services. Actual viability depends heavily on storage conditions. Good enough for estimates, not for guarantees.

**Sources:**
- [University of Nebraska Extension - Seed Storage and Germination](https://go.unl.edu/seedstorage)
- [Illinois Extension - Seed Viability](https://extension.illinois.edu/sites/default/files/seed_viability.pdf)
- [AlboPepper Seed Viability Chart](https://albopepper.com/seed-viability-chart.php)
- [High Mowing Seeds Viability Chart](https://www.highmowingseeds.com/blog/seed-viability-chart/)

## Sources

- [Seedtime Garden Planner](https://seedtime.us/) -- Primary competitor, calendar-focused planning
- [Garden Savvy Garden Manager](https://gardensavvy.com/garden-manager/) -- Inventory tracking with supplier links
- [Johnny's Seeds Planting Calculator](https://www.johnnyseeds.com/growers-library/seed-planting-schedule-calculator.html) -- Seed starting date calculator
- [Garden Betty Seed Starting Calculator](https://gardenbetty.com/seed-starting-calculator/) -- Location-based planting dates
- [Sow Right Seeds Planting Calculator](https://sowrightseeds.com/blogs/planters-library/planting-calculator-for-sowing-calendar) -- Zip code based planting calendar
- [Botanical Interests - How to Read a Seed Packet](https://www.botanicalinterests.com/pages/how-to-read-a-seed-packet) -- Data fields on BI packets
- [Permies Forum - App for managing seed collection](https://permies.com/t/63002/app-managing-seed-collection) -- User pain points
- [Steel Raven Farms - Digital Seed Inventory](https://steelravenfarms.com/blog/how-to-create-a-digital-seed-inventory/) -- What gardeners track
- [Mitten Gardening - Seed Inventory Spreadsheet](https://mittengardening.com/seed-inventory-spreadsheet/) -- Spreadsheet tracking patterns

---
*Feature research for: Garden seed inventory management and planting scheduler*
*Researched: 2026-03-08*
