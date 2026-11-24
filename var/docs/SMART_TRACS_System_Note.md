# SMART-TRACS: WHAT IT ACTUALLY IS - SOFTWARE OR APP? CAN YOU DOWNLOAD AN APK?

> **Short answer: Smart-TRACS is a SYSTEM, not a public app. There is no public APK.**
> The only installable app in the whole chain is ODK Collect, a free open-source Android
> app anyone can download today (link verified below). The Smart-TRACS-branded parts
> (smart tags + customer database + RCC collection screens) were an in-house build for
> one contractor (Green WastePro Ltd, Ilala/DCC) and were never published to Google Play
> or any app store.

Source of everything below: Pellatt & Palfreman (2023), *GeoJournal* 88:5221-5245,
open access - full text already in this pack at
`full-text-extracts/Pellatt_Palfreman_2023_SmartTRACS_GreenWastePro_DSM.txt`
(PDF copy: `districts/01_ILALA_DCC/pdf/ILALA_Pellatt_Palfreman_2023_SmartTRACS_GreenWastePro_DCC_GeoJournal.pdf`).
Contractor profile: `analysis-md/contractors/ILALA_Contractor_X_GreenWastePro.md`.

---

## 1. WHAT THE SYSTEM IS MADE OF (each part REAL, quoted from the paper)

| # | Component | Type | Public / private? |
|---|---|---|---|
| 1 | **Smart tags** - a unique identifier placed on each service user's property entrance, carrying the customer number used for payments and complaints | Physical tag + ID scheme | PRIVATE (in-house) |
| 2 | **Centralized database** of digitized customers - building footprint, name, address, floors, GPS coordinates, plot number, bill-payer name/phone/email, property type (by-law category), tariff, service-needs assessment | Backend software | PRIVATE (in-house) |
| 3 | **ODK Collect** (Open Data Kit) - the open-source Android app data collectors used for the door-to-door digital survey | Android app | **PUBLIC - free download** (verified link in section 3) |
| 4 | **OpenStreetMap building footprints** - one polygon per building, digitized by OMDTZ from World Bank 2017 BRT drone imagery + 2016 COWI fixed-wing imagery | GIS data layer | PUBLIC (OSM) |
| 5 | **QGIS** - used to randomly sample the 421 surveyed service users | Desktop GIS software | PUBLIC (free) |
| 6 | **RCC collection application** - what revenue collectors used to log payments against the tag number; the paper shows "an example of the Smart-TRACS application for RCC collection" (Fig. 12) and states collectors were "issued strict targets and thus collected more RCCs... as it is being tracked" | Field screens / dashboard | PRIVATE (in-house, never published) |
| 7 | **Mobile-money remote payment** - service users could pay RCC remotely by mobile money if not home; payment "registered with the municipality and company" | Payment integration | PRIVATE (integration layer) |
| 8 | **Google Forms satisfaction survey** run on a smartphone (421 users) | Web form | PUBLIC tool, PRIVATE survey |

## 2. WHO BUILT IT AND WHERE IT RAN (REAL)

- **Operator:** Green WastePro Ltd (GWPL), the licensed contractor for **Kisutu + Mchafukoge
  wards** (downtown Ilala, under Dar es Salaam City Council).
- **Partners:** DCC (approval and support) + **I4ID** (Institution for Inclusive Development,
  UK FCDO-funded programme) + **OMDTZ** (OpenMap Development Tanzania - did the mapping and
  provided technical expertise).
- **Pilot start:** December 2018 (tags installed from Dec 2018 onward).
- **Scale:** ~2,296 building/service points digitized; 421 service users surveyed
  (201 Mchafukoge, 211 Kisutu).
- **Measured results:** +129% identified service users, +13.5% revenue collection,
  +55% revenue-collection coverage.

## 3. THE ONLY APK YOU CAN ACTUALLY INSTALL (verified 2026-11-23)

| App | Role in Smart-TRACS | Link | Status |
|---|---|---|---|
| **ODK Collect** (org.odk.collect.android) | the app field teams used to survey every building and service user | https://play.google.com/store/apps/details?id=org.odk.collect.android | **HTTP 200 verified** - free, open source (also on F-Droid and odk.zipware.org? no - official channels only: Play Store + F-Droid + getodk.org) |
| QGIS (desktop, not APK) | random sampling + mapping | https://qgis.org | public, free |
| OpenStreetMap | building-footprint baselayer | https://www.openstreetmap.org | public |

## 4. WHY THERE IS NO SMART-TRACS APK (honest reasoning, flagged as reasoning not fact)

1. The paper (our only primary source) names **no software vendor, no app-store listing,
   no package name** - it describes tags + database + ODK Collect + payment integration.
2. It was a **single-contractor pilot for two wards** (~2,296 points), built with donor
   programme support (I4ID), not a commercial product for sale.
3. Web searches (2026-11-23) find **no Google Play / App Store entry, no vendor website,
   no GitHub repo** under the name Smart-TRACS / "Smart Technology Recording City Solution".
4. Fig. 12 in the paper proves an internal RCC-collection application existed, but the paper
   does not identify who coded it or on what stack - only that GWPL used it.

> If you want to REPLICATE what Smart-TRACS did, everything needed is public:
> ODK Collect (survey app) + OpenStreetMap footprints (baselayer) + QGIS (sampling/maps)
> + a customer database + mobile-money payment API (M-Pesa / Tigo Pesa / Airtel Money
> business APIs) + printable tags with unique customer numbers. That combination IS
> Smart-TRACS in substance. This pack's own proposal doc
> `analysis-md/06_Smart_Waste_System_Proposal.md` builds on exactly this pattern.

## 5. THINGS NOT TO CONFUSE IT WITH (checked 2026-11-23)

| Name found online | What it actually is | Verdict |
|---|---|---|
| **"TRACS 4 Citizens"** (Google Play, App Store) | Traffic e-ticket app of **Sindh Police, PAKISTAN** (tracs.sindhpolice.gov.pk) | UNRELATED - different country, different subject |
| **TRACS Mobile / NAPA TRACS / TRACS Direct** | Asphalt/transport business software (USA) | UNRELATED |
| **smarttracs.com** | Domain resolves (HTTP 200) but automated check could not identify the content (no readable title; JS-heavy page) | UNVERIFIED - no evidence it is the DSM system; do not trust it without opening it yourself in a browser |
| **SURTRAC** (Pittsburgh) | Adaptive traffic-signal control research system | UNRELATED - traffic, and a different name |

## 6. ONE-LINE ANSWERS (for the field notes)

- **Software or application?** It is a software SYSTEM (tags + database + collection app
  + payment integration) delivered around one free public Android app, ODK Collect.
- **APK?** No Smart-TRACS APK exists publicly. If someone offers you one, ask for its
  source - there is no official channel. Download ODK Collect instead (link above).
- **Link?** The durable public artefact is the open-access paper:
  https://doi.org/10.1007/s10708-023-10917-3 (HTTP 200 verified 2026-11-23) - and the pack
  already holds the PDF + full text extract.

## 7. WHAT IS ACTUALLY "SMART" IN THE SYSTEM (the smart parts, per the reports)

The paper's own logic: a smart city *unlocks city data* to improve "efficiency, resilience,
innovation and inclusivity", instead of "closed and unconnected systems resulting in silos".
Smart-TRACS is smart because each piece below turns something that was invisible, paper-based
or cash-only into tracked digital data.

**The 8 documented smart elements (all REAL, quoted/derived from Pellatt & Palfreman 2023):**

| # | Smart element | What it replaced (the "dumb" before) | Evidence |
|---|---|---|---|
| 1 | **Smart tag** - unique ID on each customer's property entrance | No customer identity: collectors could not prove who was a registered service user; people denied being customers | Fig. 8; "place a unique identifier... onto each service user's property that connects to a centralized database" |
| 2 | **Digitized customer database** (~2,296 points) - footprint, name, address, floors, GPS, plot no., bill-payer contact, property type (by-law category), tariff, service-needs assessment | No registry at all - council and contractor did not know who owed what, at which tariff | Figs. 9-11; service user types "following customer digitization" |
| 3 | **OSM building footprints as free baselayer** - OMDTZ digitized every building from World Bank 2017 BRT drone + 2016 COWI aerial imagery | Conventional surveying/drones of their own = expensive | "OMDTZ used high-[res imagery]... for the field mapping" |
| 4 | **ODK Collect digital door-to-door survey** - data straight from phones to the GWPL server | Paper forms + retyping | "obtained the following information through a digital survey via the open-source software Open D[ata Kit]... uploading to the Green Wastepro Ltd server" |
| 5 | **Tracked RCC collection app** - every payment logged against the tag number | Collectors "charging the wrong RCC tariff" or under-collecting undetected; collectors skipped residential (low-tariff) users and chased shops/hotels | Fig. 12 "Smart-TRACS usage for RCC collection and tracking"; "issued strict targets and thus collected more RCCs... as it is being tracked"; DCC Head of Environment confirmed increased deposits |
| 6 | **Mobile-money remote payment** - pay RCC remotely if not home; payment "registered with the municipality and company" | Cash-only, pay in person during business hours, no receipt trail | Satisfaction survey items: "more secure with payments... registered with the municipality and company"; "able to plan better for bill payment" |
| 7 | **QGIS random sampling + satisfaction survey** - 421 users surveyed to MEASURE the effect | Anecdote-based evaluation | "digital service user satisfaction survey using Google Forms"; QGIS random sample |
| 8 | **The data loop back into operations** - GWPL used the dataset for route/service planning, then self-funded expansion to Kivukoni ward and bought a dedicated recycling truck from increased revenues | One-off collection with data never reused | GWPL management interviews; "purchasing a dedicated recycling truck due to increased revenues... as a direct impact of Smart-TRACS" |

**The effect (measured, FY2018 -> FY2019):** +129% identified service users,
+13.5% revenue collection, +55% payment coverage - installed at **~US$3 per service user**.

**What is NOT smart in it (honest limits, per the paper):**
- No IoT sensors, no truck GPS, no fill sensors, no AI route optimization - it is
  **tags + database + tracking**, not an IoT platform.
- The paper names **data silos** as the core threat: "the high-risk and low-trust
  environment... promotes data silos that defeat the connectivity objective of smart city
  development", and a similar earlier solution "eventually failed due to a loss of contract
  and **no linkage to existing government systems**".
- Verdict of the paper itself: Smart-TRACS "could be considered an environmental, economic,
  and socially desirable solution" and supports the *transition* toward smart city status -
  it is not itself a full smart-city system.

**What the pack's own proposal adds to complete the "smart" (doc 06 "Dar Smart Waste"):**
the SMART formula - Sensing (bin sensors, truck GPS, weighbridges), Mobile (app + USSD +
mobile money), Automation (billing, route optimization), Reporting (one city dashboard),
Traceability (household-to-landfill) - i.e. doc 06's 9 elements are the scale-up of exactly
the Smart-TRACS pattern (identity -> registry -> tracked payment -> data reuse).

---

## 8. SAMPLE ODK FORM URLS TO SEE THE APP IN USE (verified 2026-11-23)

**Fastest way - zero setup, official demo inside the app itself:**

ODK Collect ships a built-in demo project. Quote from https://docs.getodk.org/collect-connect/
(fetched 2026-11-23):

> "If you'd just like to try out ODK Collect without setting up a server, you can use
> sample forms on the demo server. You can set this up by tapping **Try a demo** at the
> bottom of the screen when you first launch Collect. The demo project can be deleted later."

Steps: install ODK Collect (https://play.google.com/store/apps/details?id=org.odk.collect.android)
-> launch -> tap **Try a demo** -> sample forms download -> **Fill Blank Form**. This reproduces
exactly what the Smart-TRACS field teams did (fill ODK forms on Android, send to a server).

**Official ODK sample forms (XForm XML, direct downloads, HTTP 200 verified today,
each starts with a valid `<h:html xmlns="http://www.w3.org/2002/xforms"` header):**

| Form | Direct URL |
|---|---|
| Basic | https://raw.githubusercontent.com/getodk/sample-forms/master/xml-examples/Basic.xml |
| Birds (photos/media) | https://raw.githubusercontent.com/getodk/sample-forms/master/xml-examples/Birds.xml |
| Bugs | https://raw.githubusercontent.com/getodk/sample-forms/master/xml-examples/Bugs.xml |
| Training | https://raw.githubusercontent.com/getodk/sample-forms/master/xml-examples/Training.xml |
| Widgets | https://raw.githubusercontent.com/getodk/sample-forms/master/xml-examples/Widgets.xml |

Repo (more forms incl. Geo, Household, repeats, geolocation folders):
https://github.com/getodk/sample-forms

**XLSForm (Excel) examples for building/uploading a form to ODK Central or KoboToolbox
(both HTTP 200 verified, real .xlsx zip):**

- https://raw.githubusercontent.com/XLSForm/example-forms/master/start_screen.xlsx
- https://raw.githubusercontent.com/getodk/xlsform-template/master/ODK%20XLSForm%20Template.xlsx
  (the official ODK blank template)

**Dar es Salaam / Smart-TRACS specificity (honest status):**

- The closest verified Dar-specific evidence of the app in use is OMDTZ's article
  "OMDTZ Collects Data to Address Litter Management Challenges in Dar es Salaam" -
  household litter data collection "using open source tools, i.e., OpenDataKit and QGIS"
  (the same OMDTZ that digitized the buildings for Smart-TRACS):
  https://omdtanzania.medium.com/omdtz-collects-data-to-address-litter-management-challenges-in-dar-es-salaam-using-open-m
  - UNVERIFIED beyond search-index snippet: the article body is Cloudflare-blocked from
    this environment and the Wayback Machine API returned 429 on the day of checking.
- **No publicly downloadable Dar es Salaam-specific ODK form file exists** (web searched
  2026-11-23: only research PDFs, no public .xml/.xlsx form). UNVERIFIED items stay
  flagged; the generic ODK sample forms above are the ones you can actually load today.

**Dead ends checked so nobody wastes time on them (2026-11-23):**

- `sandbox.getodk.org` - does NOT exist (DNS NXDOMAIN).
- `demo.getodk.org` - resolves but only redirects to the getodk.org marketing site; the
  real demo is reached via the in-app **Try a demo** button, not a public URL.
- `enketo.org/demo` - 404. enketo.org itself is up (200) but has no public demo form page.
- Loading a raw .xml into Collect without a server requires adb
  (https://docs.getodk.org/collect-adb/) or ODK Briefcase - the Try-a-demo path above is easier.

---

*Note compiled 2026-11-23 from the pack's Pellatt & Palfreman (2023) full-text extract plus
same-day link checks (curl with browser user-agent). Section 8 link checks: all URLs above
returned HTTP 200 with expected content types on 2026-11-23; unverifiable items are labelled
UNVERIFIED above rather than guessed.*
