# FINAL YEAR PROJECT REPORT

**[INSTITUTION / COLLEGE — e.g. COLLEGE OF INFORMATION AND COMMUNICATION TECHNOLOGY, DEPARTMENT OF COMPUTER SCIENCE AND ENGINEERING]**

**REPORT TITLE: SMART WASTE COLLECTION SYSTEM FOR DAR ES SALAAM — A MOBILE AND WEB-BASED CUSTOMER REGISTRY, PAYMENT AND COLLECTION TRACKING SYSTEM**

**NAME: [STUDENT NAME]  REGISTRATION NO: [REG NO]**

**SUPERVISOR'S NAME: [SUPERVISOR]**

**SIGNATURE: ………………………………   DATE: ……………………………….**

---

## DECLARATION

I declare that this report and the work described in it is my own work, with any
contributions from others expressly acknowledged and/or cited. I declare that the work in
this report was carried out in accordance with the regulations of [the University] and has
not been presented to any other University for examination either in Tanzania or overseas.

SIGNED: ………………………………   DATE: ………………………………

SUPERVISOR: ………………………………

SIGNATURE: ………………………………   DATE: ………………………………

---

## ABSTRACT

Dar es Salaam generates about 5,733 tonnes of municipal solid waste per day (UN-Habitat
WaCT survey, 2021), collected through a franchising model in which more than 30 licensed
contractors and neighbourhood (mtaa) collectors collect Refuse Collection Charges (RCC)
from households and businesses. The sector is still largely manual: customer records are
paper-based, payment compliance in low-income wards sits between 30% and 50% (CSE, 2021),
collectors sometimes charge wrong tariffs or demand duplicate payments, and managers have
no live, map-based picture of their service zones. The one documented smart intervention —
the Smart-TRACS pilot (Green WastePro Ltd, Kisutu and Mchafukoge wards, 2018) — proved the
model: after digitising customers with smart tags and a central database, identified
service users rose by 129%, revenue collection by 13.5% and revenue-collection coverage by
55% (Pellatt & Palfreman, 2023) — but it remained a closed, single-contractor tool that was
never published or replicated.

This project designs and develops a **Smart Waste Collection System (SWCS) for Dar es
Salaam**: a mobile and web-based platform in which (1) every service user is registered
against a unique property tag with its tariff category and GPS position; (2) RCC payments
are recorded digitally with receipts tied to the tag number and mobile-money remote
payment; (3) collectors' services are logged so that coverage per ward and per contractor
is visible on a map; and (4) municipal councils get a supervision dashboard that produces
the records the law already demands — six-monthly activity records (Reg 60 of GN
263/2009), weighbridge tonnage records (Reg 9(5)) and fee-bearing permits (GN 154/2008 →
GN 387/2021). This report covers Chapters One to Three: the Introduction, the Literature
Review (including the legal framework from the Environmental Management Act, Cap. 191,
its Solid Waste Management Regulations GN 263/2009 and GN 190/2016, and the Fees and
Charges Regulations GN 154/2008 and GN 387/2021) and the Methodology, which adopts
Object-Oriented Analysis and Design (OOAD) with evolutionary prototyping. The system's
data layer is already delivered as a working prototype: a Rust geodata pipeline
(osm-fetch-dsm) that parses the official OpenStreetMap Tanzania extract (Geofabrik,
125.8 million nodes) and produces verified GeoJSON layers of Dar es Salaam — the region
boundary, all 102 official wards (NBS 2022 census) with their municipality assigned, the
road network (1,973 major + 82,941 minor lines) and 81 real waste sites, each tagged with
its containing ward and municipality by point-in-polygon computation — rendered in a
canvas map module that runs on desktop and mobile, with field research instruments
(questionnaires and interview guides in English and Kiswahili) prepared for requirements
gathering and later user evaluation.

---

## TABLE OF CONTENTS

- CHAPTER ONE INTRODUCTION
- CHAPTER TWO LITERATURE REVIEW
- CHAPTER THREE METHODOLOGY
- REFERENCES

*(Chapters Four — System Analysis and Design, Five — Implementation, and Six — Conclusion
and Recommendations follow in the next stage of the report, mirroring the structure of the
Bus Tracking System final-year report used as the formatting model.)*

---

# CHAPTER ONE INTRODUCTION

## 1.1 General Introduction

Dar es Salaam is Tanzania's commercial capital and one of the fastest growing cities in
Africa, with a population that has passed five million across the five municipalities of
Ilala (under Dar es Salaam City Council), Kinondoni, Temeke, Ubungo and Kigamboni. Every
day the city generates about 5,733 tonnes of municipal solid waste (UN-Habitat Waste Wise
Cities WaCT survey, 2021), and every day most of that waste passes through a chain that is
still organised largely on paper: households and businesses pay a Refuse Collection Charge
(RCC) to a contracted waste collector; the collector — a licensed company, group or
neighbourhood (mtaa) collector — hauls the waste to transfer points and to the Pugu
Kinyamwezi landfill; the municipal council supervises the contractors, records tonnage at
the landfill gate, and remits commissions.

The legal machinery for this chain is well developed. The Environmental Management Act,
Cap. 191 (EMA 2004) makes local government authorities responsible for waste collection
and disposal in their areas; the Environmental (Solid Waste Management) Regulations of
2009 (GN 263/2009, amended GN 190/2016) require permits for waste collection and
transportation, covered and self-tipping vehicles, weighbridge receipts, and six-monthly
activity records to the council; and the fee side of the same permit system is priced by
the Environmental Management (Fees and Charges) Regulations — first issued as GN 154 of
2008 and currently applied through GN 387 of 2021.

What is not developed is the information layer. Customer registers are handwritten,
payments are recorded in receipt books, zones are described by street names instead of
digital boundaries, and managers cannot see, on a map, which buildings are served, which
wards generate how much waste, or where the nearest dumpsite or transfer point is. Where
digitisation has been tried, it worked: the Smart-TRACS pilot in Kisutu and Mchafukoge
wards replaced paper registers with smart tags, a digital customer database and mobile
money payments — and identified 129% more service users, collected 13.5% more revenue and
expanded revenue-collection coverage by 55% (Pellatt & Palfreman, 2023).

This project therefore designs and develops the missing information layer as a working
system, not as a study: a **Smart Waste Collection System** whose modules are (1) a
customer registry that gives every service user a unique property tag, a tariff category
and a GPS position; (2) a payment module that records RCC receipts against tag numbers and
supports mobile-money remote payment; (3) a collection tracking module that logs services
per property and per zone so coverage is measurable; (4) a zone map module that renders
the city — wards, municipalities, roads and waste facilities — on verified administrative
boundaries; and (5) a municipal supervision dashboard that turns the same records into the
reports the law already requires. The system stands on an open, verified geodata layer of
Dar es Salaam produced from the official OpenStreetMap data by a purpose-built Rust
pipeline, and is grounded in requirements gathered from the laws, audits and the officers
and contractors who will use it.

## 1.2 Statement of the Problem

In Dar es Salaam, waste collection revenue and service quality depend on records that do
not exist in digital form. Four concrete problems follow from this:

1. **No reliable customer database.** Collectors charge wrong tariffs and demand duplicate
   payments because no digitised register of service users and their categories exists;
   this was documented before digitisation in the Smart-TRACS wards (Pellatt & Palfreman,
   2023).
2. **Low and invisible payment compliance.** Payment compliance in low-income areas is
   estimated at 30–50% (CSE, 2021), and because payment points are not mapped, managers
   cannot see where non-payment concentrates or how collection coverage changes over
   time.
3. **Zones and facilities are not mapped.** Ward and mtaa service areas, dumpsites,
   transfer points and routes exist in by-laws and tender annexes but not as digital
   boundaries, so planning (who collects where, which site serves which ward) is done
   verbally.
4. **Permit and fee administration is fragmented.** A contractor needs a collection or
   transportation permit from NEMC (fees under GN 387/2021, originally GN 154/2008),
   vehicle inspection certificates under GN 177/2001, OSHA registration, and a council
   contract — but there is no single checklist or digital record tying these obligations
   to each contractor's zone.

No existing system serves Dar es Salaam at city scale with these four problems solved
together. The Smart-TRACS pilot proved the concept but covered two wards with one
contractor, was never published as a product, and cannot be replicated from its paper;
the councils' point-of-sale registration mandate (2017) records payments but does not
register customers or map coverage. That is the gap this project fills: **designing and
developing an open, replicable Smart Waste Collection System** that any of the five
municipalities and their contractors can adopt.

## 1.3 Project Objectives

### 1.3.1 Main Objective

To design and develop a mobile and web-based **Smart Waste Collection System for Dar es
Salaam** that digitises service-user registration with unique property tags, records and
tracks Refuse Collection Charge payments including mobile money, logs collection services
against mapped administrative zones, and gives municipal councils live supervision
dashboards.

### 1.3.2 Specific Objectives

1. **To design and develop the customer registry module** — every service user registered
   against a unique property tag (the Smart-TRACS pattern) with property category and
   tariff (residence, shop, office, restaurant, hotel, education centre), GPS position and
   contact details, replacing handwritten registers.
2. **To design and develop the RCC payment module** — payment recording with receipts tied
   to the tag number, tariff enforcement that prevents wrong charges and duplicate
   payments, and mobile-money remote payment so a resident can pay even when absent.
3. **To design and develop the collection tracking module** — collectors and supervisors
   log services per property and per zone, so payment compliance and collection coverage
   become visible per ward, per contractor and over time.
4. **To design and develop the zone map module** — the system's map layer rendering the
   region, municipalities, wards, roads and every known waste facility on verified
   OpenStreetMap-derived boundaries, with each facility attributed to its ward and
   municipality by point-in-polygon computation.
5. **To design and develop the municipal supervision dashboard** — coverage and compliance
   per contractor, CPI scoring support, and automatic preparation of the legally required
   artefacts: six-monthly activity records (Reg 60 of GN 263/2009), tonnage records (Reg
   9(5)) and permit/fee status under GN 154/2008 → GN 387/2021.
6. **To evaluate the system prototype with its intended users** — LGA officers, ward and
   mtaa officers and contractors — using structured questionnaires and interview guides in
   English and Kiswahili, measuring usability and fit against the documented legal and
   operational requirements.

## 1.4 Significance of the Project

- **For contractors and mtaa collectors:** a digital customer and zone map reduces
  duplicate payments and tariff disputes — the effect measured in the Smart-TRACS pilot
  (+129% identified users, +13.5% revenue).
- **For municipal councils (Ilala/DCC, Kinondoni, Temeke, Ubungo, Kigamboni):** live
  coverage and payment dashboards make supervision, CPI scoring and the six-monthly
  reporting under Reg 60 of GN 263/2009 cheaper and evidence-based.
- **For NEMC and the Director of Environment:** fee administration under GN 154/2008 /
  GN 387/2021 becomes checkable, because every permitted facility (landfill, dumpsite,
  recycling plant) can be located on the map with its annual compliance charge.
- **For residents:** receipts tied to a property tag end duplicate-payment disputes, and
  mobile-money remote payment lets a household pay even when nobody is home.
- **For researchers and students:** the system is open and replicable — the geodata
  pipeline, the zone layers and the module designs can be reused for any Tanzanian
  municipality.

## 1.5 Scope and Limitation of the Project

The system covers Dar es Salaam region — the five municipalities of Ilala (under DCC),
Kinondoni, Temeke, Ubungo and Kigamboni — and is delivered as a working prototype
consisting of:

- **The zone map module (working prototype):** verified GeoJSON layers generated from the
  official Geofabrik Tanzania extract by the osm-fetch-dsm Rust pipeline, rendered in a
  canvas-based web map with ward/municipality legend, search, layer toggles, road and
  facility details — usable on desktop and mobile.
- **The registry, payment, tracking and dashboard modules (design + first implementation,
  completed in Chapters Four and Five):** web dashboard and mobile-friendly data entry,
  with field data collection through the open-source ODK Collect app — the same app the
  Smart-TRACS teams used, so field crews need no proprietary software.
- **Open data only:** OpenStreetMap (ODbL) and public documents; no proprietary map or
  data licensing.

Limitations, stated honestly:

- OSM does not yet map district-level boundaries for the five municipalities; districts
  are therefore *derived* as unions of ward polygons, and wards missing from OSM leave
  visible gaps (listed in the generated summary, not filled by guessing).
- Of 81 waste sites in the bounding box, 46 fall inside the region boundary; some
  facilities known from documents (e.g. the Pugu Kinyamwezi landfill, the Mtoni Temeke
  tender site) are not separately tagged in OSM and are cited from documents instead.
- The mobile-money integration is designed for Tanzania's gateways (mobile network
  operator money and licensed aggregators) but is connected to a sandbox in the prototype;
  no live financial system is connected in this stage.
- Live GPS tracking of collection trucks is out of scope for this stage (the natural next
  phase, using the same GPS approach as the Bus Tracking System model); likewise IoT
  bin-fill sensors.

## 1.6 Project Tasks Distribution

The project proceeds as: (1) requirements evidence collection — laws, audit reports,
academic studies and the Smart-TRACS case assembled into a verified document pack; (2)
requirements analysis — actors, legal artefacts the system must produce, functional and
non-functional requirements; (3) data-channel testing (Overpass mirrors, ohsome, OSM API
0.6, Geofabrik, Nominatim) and the Rust geodata pipeline development; (4) zone-layer
generation (wards, districts, roads, waste facilities with zone attribution); (5) zone map
module development (canvas rendering, legend, search, details); (6) field instrument
design (English and Kiswahili) for requirements gathering and later evaluation; (7)
registry, payment, tracking and dashboard module design and implementation; (8) testing —
pipeline unit checks, link verification, browser end-to-end tests and user evaluation —
and documentation.

## 1.7 Organization of the Report

- **Chapter One** gives the general introduction, the statement of the problem, the main
  and specific objectives, the significance, the scope and limitations, and the
  organisation of the report.
- **Chapter Two** reviews the literature: solid waste management in Dar es Salaam, the
  system stakeholders, the legal and fee framework (EMA Cap. 191; GN 263/2009 and GN
  190/2016; GN 154/2008 and GN 387/2021), current existing systems including the
  Smart-TRACS pilot, and the research gap.
- **Chapter Three** presents the methodology: Object-Oriented Analysis and Design with
  evolutionary prototyping, the system's requirement sources (legal documents, field
  instruments, OSM/Geofabrik geodata), tools and materials, and the phases from
  requirement gathering to testing.
- Chapters Four (System Analysis and Design), Five (Implementation) and Six (Conclusion
  and Recommendations) follow in the next stage of the report.

---

# CHAPTER TWO LITERATURE REVIEW

## 2.1 Solid Waste Management in Dar es Salaam

Dar es Salaam's waste system is a franchising chain. Under by-laws made from the Local
Government (Urban Authorities) Act, Cap. 288 and the EMA, each municipality licenses
contractors who collect RCC from premises and deliver waste to the Pugu Kinyamwezi
landfill (the city's main engineered site, gate fee TZS 1,500–2,000 per tonne for
household waste in 2021–2023) or to transfer points. More than 30 real contractors and
groups are documented across the five municipalities — among them Green WastePro Ltd
(Ilala's Kisutu/Mchafukoge contractor and the Smart-TRACS pilot operator), Magomeni Taka
Group, Mandela Group, Vingunguti Group and Wakusanya Taka Bonyokwa Cooperative (Ilala);
17 named cleansing contractors (Kamugisha, 2019) plus Bunju Group (Kinondoni); Umoja Wetu
ni Ushindi Wetu (Ubungo, TZS 70M Saranga award); Kimuchi Group (CPI 84%), Kisonge Group
(CPI 66%) and Orphan & Widow Group (CPI 47%) in Mbagala ward, Temeke (Magombola, 2021);
and Taka Ajira Company plus market-based companies in Kigamboni.

Performance is uneven. The CSE (2021) ecosystem assessment found payment compliance of
30–50% in low-income areas, tariff structures that vary by property category (residence,
shop, office, restaurant, hotel, education centre — the same categories digitised in the
Smart-TRACS survey and carried into this system's registry module), and mtaa-level
collectors who remit to councils through the "deposit-all" model — in Ilala, collectors
deposit 100% of RCC into the council account and receive back the remainder after a
commission often around 10% of the tender amount (Ilomo & Charles, 2025). The Controller
and Auditor General's annual reports for 2021/22 and 2022/23 document continuing
weaknesses in contract supervision and revenue records across LGAs, and the World Bank's
DMDP Phase II (P180298) recorded **zero progress** on its solid waste indicators at the
March 2024 restructuring — while roads and flood components achieved their targets —
showing that money alone has not digitised the sector.

## 2.2 Identifying System Stakeholders

The stakeholders of the Smart Waste Collection System map directly onto the actors
already named in the laws and contracts:

- **NEMC and the Director of Environment** — issue permits (Reg 9, GN 263/2009), receive
  six-monthly records (Reg 60), administer fees under GN 154/2008 / GN 387/2021.
- **The five municipal councils and their environmental officers** — supervise
  contractors, keep CPI scores, operate the landfill gate and record tonnage (Reg 9(5)).
- **Ward and mtaa officers (WEOs, mtaa executive officers)** — know every building and
  every payer; the natural owners of the customer database.
- **Licensed contractors and CBO groups** — the primary users: routes, customer registers,
  payment tracking, OSHA compliance (s.62 PPE duty), vehicle inspection (GN 177/2001).
- **Households, businesses and institutions** — the payers, reachable by mobile money and
  reachable through the smart-tag customer number for complaints.
- **Informal waste pickers and recyclers** — documented at ~1.5 t/day of recyclables in
  the Smart-TRACS zone alone; any smart system must include, not displace, them.

## 2.3 Legal and Fee Framework

### 2.3.1 The Environmental Management Act, Cap. 191

The EMA 2004 (Act No. 20 of 2004) assigns waste management functions to local government
authorities and empowers the Minister to make regulations, including under **section
230(2)(h)** for fees and charges.

### 2.3.2 Solid Waste Management Regulations — GN 263/2009 and GN 190/2016

GN 263/2009 (with its 2016 amendment GN 190/2016) is the operational core: Reg 9
(permit to collect/transport; application documents), Reg 9(5) (weighbridge receipts and
tonnage records), Reg 10 (vehicle standards: covered, watertight, self-tipping, loading
height 1.4–1.6 m), Reg 53 (records at facilities) and Reg 60 (records of the licensed
activity to the LGA every six months, forwarded to NEMC). The system's dashboard module
is designed to produce exactly these artefacts, not to replace them.

### 2.3.3 Fees and Charges — GN 154 of 2008 and GN 387 of 2021

The scanned Government Notice examined for this report, **GN 154 of 2008 — the
Environmental Management (Fees and Charges) Regulations, 2008** (made under s.230(2)(h)
of Cap. 191, published 5 September 2008, signed by the Minister of State, Vice President's
Office, Batilda S. Burian), sets the original price list for the permit system. Its
regulations define "authority" to include local government authorities, require payment
for every licence or permit granted by the Council (NEMC) or an authority, allow the
collection duty to be **delegated to an agent** (reg. 7 — the same delegation logic later
used for RCC collection), and make all fees non-refundable and non-transferable (reg. 8).
Its Schedule prices, among others: EIA applications (TZS 20,000) and certificates,
registration of environmental experts (TZS 100,000–1,500,000 by category), and — item C1
of the annual compliance charges — **"Sanitary landfills, Dumpsite & waste water treatment
plants: TZS 1,000,000 per year"**.

The current instrument, **GN 387 of 2021** (the Environmental Management (Fees and
Charges) Regulations, 2021, held in this project's law pack), carries the schedule
forward: annual environmental compliance monitoring and audit fees now list **"Landfill:
TZS 750,000"** and **"Dumpsite: TZS 1,500,000"** separately — a deliberate policy
difference that prices open dumpsites *higher* than engineered landfills — plus waste
collection/transportation permit fees (e.g. TZS 5,000,000 for collection above 5 tonnes
per day) and recycling/recovery plant registration (TZS 300,000). The evolution from
GN 154/2008 to GN 387/2021 shows the State using fee design as waste-policy: an operator
of an open dumpsite now pays double the fee of a landfill operator, which the system's
facility ranking should reflect when listing candidates for upgrading.

Other instruments relevant to the system: OSHA Act 2003 (workplace registration, medical
examinations s.24, first aid s.58, PPE duty s.62), Road Traffic Act Cap 168 with GN
177/2001 (vehicle inspection certificates), and the Personal Data Protection Act, 2022 —
because a digitised customer database (names, phone numbers, payments) is personal-data
processing that the Smart-TRACS survey already flagged as a public concern, and the
system must therefore implement consent, access control and data-minimisation.

## 2.4 Current Existing Systems

**[1] Smart-TRACS (Dar es Salaam, 2018).** The closest existing system, and the pilot this
project generalises to a city-scale product. Green WastePro Ltd, with DCC approval, the
I4ID programme and OpenMap Development Tanzania, placed unique **smart tags** on ~2,296
service points, digitised customers into a **centralised database** via the open-source
ODK Collect app, built on OpenStreetMap building footprints (2017 World Bank drone
imagery), and enabled mobile-money payment against each tag's customer number. Measured
results: **+129% identified service users, +13.5% revenue collection, +55% coverage**;
421 users surveyed reported easier, more secure payments and faster complaint handling by
quoting the tag number. Limitations: single contractor, two wards, never published as a
product, no public map, and no replication package — which is exactly the gap this
project closes by developing the same registry-plus-tracking pattern as an open,
city-scale system.

**[2] POS-based RCC registration (2017 mandate).** Councils required point-of-sale
registration of RCC payments centrally (documented in Pellatt & Palfreman, 2023);
adoption is partial and receipts are not tied to mapped premises.

**[3] International smart waste systems.** Seoul's RFID pay-as-you-throw bins and food
waste charging cut waste volumes significantly; Songdo (Korea) and Spanish smart-fleet
deployments show route optimisation with fill-level sensors; Maputo, Mozambique uses
citizen-driven phone reporting of waste issues to the municipality (Kaza et al., 2016).
These systems assume infrastructure and payment rails that differ from Dar es Salaam's;
their transferable idea is the *data loop* this system implements: identify the customer →
record the payment → map the service → feed the dashboard.

**[4] Open geodata infrastructure.** OpenStreetMap in Dar es Salaam is unusually rich
because of Ramani Huria and OMDTZ mapping (drone imagery from the World Bank's DMDP).
This project's pipeline verified that OSM holds ward polygons for the region, 81 waste
sites in the bounding box (46 in-region), and 84,914 road line features — but no district
boundaries, which the pipeline therefore derives from ward unions and labels as derived.
This open geodata becomes the system's zone map layer at no licensing cost.

**Research gap.** No existing system for Dar es Salaam combines (a) a digitised customer
registry with unique property tags, (b) RCC payment recording with mobile money and
tariff enforcement, (c) collection tracking against mapped administrative zones, and (d)
municipal supervision dashboards that produce the legal artefacts (Reg 9 permits, Reg 60
records, GN 387/2021 fee schedules). Smart-TRACS proved the model in two wards but
remains closed and unpublished; POS registration is partial; international systems assume
different infrastructure. This project designs and develops the missing city-scale
system on open data.

---

# CHAPTER THREE METHODOLOGY

This chapter elaborates the methodology used to develop the Smart Waste Collection
System, using Object-Oriented Analysis and Design (OOAD) together with evolutionary
prototyping — the same methodological frame used by the Bus Tracking System final-year
report that serves as this report's formatting model — adapted to a system whose
foundation is verified open geodata and whose requirements are anchored in the legal
framework of the sector.

## 3.1 Research and Development Approach

The project follows OOAD: the system is modelled around objects whose attributes and
relationships mirror the legal reality of waste collection. The core object model is:
**Customer** (registered against a unique property tag, with tariff category, GPS
position and contact); **Payment** (RCC amount, receipt tied to the tag number, channel —
cash, POS or mobile money); **Collector / Contractor** (licensed operator serving defined
zones, holding permits and vehicle certificates); **Zone** (Region → Municipality/District
→ Ward → Mtaa, with digital boundaries); **Facility** (landfill, transfer station,
dumpsite, recycling plant, with operator and the annual compliance charge of GN
387/2021); **Permit** (Reg 9 collection/transportation permits); and **Report** (Reg 60
six-monthly records, Reg 9(5) tonnage records).

Development proceeds in evolutionary prototypes: each module is delivered, evaluated and
corrected, and the next prototype grows out of it. The delivered prototypes to date are
the system's zone data layer (the geodata pipeline and its outputs) and the zone map
module (the canvas web map); the registry, payment, tracking and dashboard modules follow
the same cycle in Chapters Four and Five. The approach is demonstrated by the pipeline's
own evolution through five debug-and-verify cycles (node-index fix, delta-refs fix,
region-union fix, JSON coordinate fix, memory refactor) — each prototype was tested
against the evidence, corrected, and re-verified.

## 3.2 Data Collection (System Requirement Sources)

### 3.2.1 Case Study of Documents and Records

The primary requirement source is a verified document pack of 50+ downloaded public
documents, each with its original web link and HTTP status recorded: the EMA Cap 191; GN
263/2009 and GN 190/2016; the scanned **GN 154 of 2008** (Fees and Charges) retrieved from
TanzLII and read through its embedded scan text (with OCR-checked wording, e.g. the TZS
1,000,000 annual charge for "Sanitary landfills, Dumpsite & waste water treatment
plants"); GN 387/2021; OSHA Act 2003; CSE (2021); Pellatt & Palfreman (2023, GeoJournal);
Ilomo & Charles (2025); UN-Habitat WaCT (2023); VPO National Environmental Master Plan
2022–2032; CAG annual general reports 2021/22 and 2022/23; the World Bank DMDP documents;
and the PPRA tender records. Every requirement in this report traces to a file in the
pack.

### 3.2.2 Field Instruments (Requirement Gathering and Evaluation)

Two instrument sets were designed — one for LGA officers (municipal environmental
officers, revenue officers, ward and mtaa officers) and one for contractors — each with a
structured questionnaire (A–J / A–I sections, ~36 and ~32 questions), a semi-structured
interview guide, and full Kiswahili versions (Fomu ya Maswali 3A, Miongozo ya Mahojiano
3B, Nyaraka 3C). Every question is anchored to a law, audit finding or dataset in the
pack (Reg 60 six-monthly reports, mtaa RCC flows, CPI scores, gate fees, GN 387/2021
fees), and an oral informed-consent script in English and Kiswahili precedes every
interview. These instruments serve two phases of the system build: requirement gathering
before design, and user evaluation of the prototype (Specific Objective 6).

### 3.2.3 Geodata Collection (OpenStreetMap and Geofabrik)

The system's zone layer is generated from the official OpenStreetMap Tanzania extract
(Geofabrik `tanzania-261122.osm.pbf`, 707 MB, 125,807,820 nodes), downloaded with a
range-partitioned fetch and parsed in three streaming passes. Where the full-planet
Overpass mirrors were unreachable from the development environment (all returned HTTP 403
or served only regional subsets), the extract-based approach proved the robust route.
Complementary services — Nominatim (ward→district lookup) and the OSM API 0.6 (relation
cross-checks, e.g. region relation 7202037) — were used for verification. For field data
collection the system uses the open-source **ODK Collect** app (org.odk.collect.android) —
the same app the Smart-TRACS field teams used — whose official demo forms and blank
XLSForm template were verified as downloadable during this project, so field crews can be
trained with no licensing cost.

## 3.3 Tools and Materials

| Tool | Use in the system | Status |
|---|---|---|
| **Rust + osmpbf crate (osm-fetch-dsm)** | Zone-layer generator: parses the PBF, collects roads, waste sites and admin relations, chains boundary member-ways into rings, assigns zones by point-in-polygon, writes GeoJSON | delivered |
| **Geofabrik Tanzania PBF** | The single authoritative geodata source (ODbL); replaces blocked live-query services | delivered |
| **Nominatim (OSM lookup API)** | Ward→district mapping used in the zone layer's ward map file | delivered |
| **Next.js + TypeScript Canvas 2D** | Zone map module: projection, offscreen base-layer caching, pan/zoom/pinch, legend with ward search, facility and road details | delivered (prototype) |
| **ODK Collect (org.odk.collect.android)** | Mobile field data collection for customer registration — the app the Smart-TRACS teams used; official demo forms verified for training | selected; forms designed in Ch 4 |
| **Next.js + Prisma (SQLite/PostgreSQL) backend** | Customer registry, payment recording, collection tracking and dashboard modules | designed in Ch 4; implemented in Ch 5 |
| **Mobile money (Tanzania MNO money / aggregators)** | Remote RCC payment channel; receipt tied to the property tag number | sandbox integration in prototype |
| **serde_json** | GeoJSON FeatureCollection output with 6-decimal coordinate precision | delivered |
| **tesseract + pdftotext** | Reading the scanned GN 154/2008 publication (embedded scan text + OCR cross-check) | delivered |
| **pandoc** | Converting this report between Markdown and Word for submission | delivered |
| **Agent-based browser testing** | End-to-end verification of the zone map module (rendering, selection, toggles, mobile layout) | delivered |

## 3.4 Requirement Gathering and Analysis Phase

Requirements were extracted from three convergent sources: (a) the legal artefacts the
system must produce or respect (permits under Reg 9; weighbridge records under Reg 9(5);
six-monthly records under Reg 60; fee receipts under GN 154/2008/GN 387/2021; PPE records
under OSHA s.62); (b) the operational gaps documented in the literature (customer
database, tariff enforcement, coverage measurement); and (c) the field instruments
designed for LGA officers and contractors. The analysis produced:

**Functional requirements** (mapped to the specific objectives): register a service user
against a unique property tag with tariff category and GPS position (FR1); record an RCC
payment with an itemised receipt tied to the tag number and prevent wrong-tariff and
duplicate charges (FR2); accept remote mobile-money payment against the tag number (FR3);
log collection services per property and per zone (FR4); render the administrative zone
map with wards, municipalities, roads and waste facilities, each facility attributed to
its ward and municipality (FR5); show coverage, compliance and CPI support per contractor
and per ward (FR6); prepare the Reg 60 six-monthly record and Reg 9(5) tonnage record
formats automatically (FR7); route resident complaints by tag number (FR8).

**Non-functional requirements**: offline tolerance for field data entry (ODK Collect
works offline and syncs later); low-bandwidth operation; personal-data protection in line
with the Personal Data Protection Act, 2022 (consent, access control, data minimisation);
bilingual English/Kiswahili interface; and honest-data discipline — unknown values are
left empty and flagged, never guessed.

**The object model** derived from the analysis: an administrative hierarchy (Region →
District → Ward) that OSM only partially contains — requiring the documented derivation
of district unions — and a facility model in which every WasteSite carries kind (landfill
/ waste_disposal / waste_transfer_station / recycling), operator, and zone attributes,
with unknown zones left empty rather than inferred.

## 3.5 Design Phase

### 3.5.1 System Architecture

The system is a mobile-and-web platform in three tiers: (1) **field tier** — ODK Collect
on Android (offline-capable forms for customer registration and service logging); (2)
**server tier** — a web backend holding the customer registry, payments, services,
permits and reports, exposing the zone layer and dashboards; (3) **supervision tier** —
the council dashboard (web) rendering coverage, compliance and the legal report formats.
The zone map module runs inside the web tiers on a Canvas 2D renderer so that the full
city (84,914 road segments) stays responsive without heavy map servers.

### 3.5.2 Zone Data Layer Design

Key design decisions visible in the delivered data-layer prototype:

1. **One bbox, honest clipping.** The Dar es Salaam bounding box (S -7.15, W 38.98, N
   -6.60, E 39.65) clips roads but never silently truncates sites — every feature carries
   an `in_region` flag.
2. **Derived-but-labelled boundaries.** District polygons are unions of ward polygons
   (OSM lacks district relations); each district feature names its derivation and the
   number of wards unioned (Ilala 23, Kinondoni 14, Temeke 14, Ubungo 8, Kigamboni 7).
   The ward layer is completed to the official 102 wards of the NBS 2022 census (Ilala
   36, Kinondoni 20, Temeke 23, Ubungo 14, Kigamboni 9): 70 OSM boundary polygons, 30
   wards anchored at real OSM point objects, and 2 wards with no OSM object (Majohe,
   Mianzini) listed as gaps — never fabricated.
3. **Zones by computation, not assertion.** Each facility's `zone_ward` / `zone_district`
   come from point-in-polygon at generation time; the ward→district fallback map is
   versioned and cited.
4. **Canvas rendering with cached base layer.** The heavy layers (region, districts,
   wards, roads) render into an offscreen canvas; pan/zoom blits it and overlays the
   dynamic layer (facilities, selection, zone highlight), keeping interaction smooth at
   84,914 road segments.
5. **Even-odd polygon fills** so multipolygon districts and holes render correctly
   without winding-order assumptions.

### 3.5.3 Registry and Payment Design (for Chapters Four and Five)

The registry follows the measured Smart-TRACS pattern: a unique tag per service point,
digitised through ODK Collect with the ward and mtaa pre-filled from the zone layer; the
payment module enforces the tariff table per property category, issues receipts against
the tag number, and records mobile-money payments so that the "deposit-all" remittance
flow documented by Ilomo & Charles (2025) can be reconciled digitally. Report formats
follow the law exactly: Reg 60 records every six months, Reg 9(5) tonnage receipts at the
gate.

## 3.6 Implementation Phase (prototype completed)

Delivered prototypes: (a) **the system's zone data layer** — `osm-fetch-dsm`, the Rust
pipeline (Cargo project, osmpbf + serde_json) with CLI options `--bbox`, `--wardmap`,
`--source-date`; (b) its eight GeoJSON / JSON outputs (region, districts, wards including
the 102-ward completion, roads_major, roads_minor, dumpsites, summary, wardmap); (c) **the
zone map module** — a canvas web map with the full 102-ward legend grouped by
municipality with search, clickable wards and roads whose OSM name, class, surface and
approximate length appear in a properties card, layer toggles, facility selection with
ward-zone highlighting, scale bar and attribution; (d) the field instruments pack
(English + Kiswahili) and the requirement evidence pack (50+ verified documents). The
pipeline's final verified counts: 102 official wards covered (70 OSM boundary polygons +
30 point anchors + 2 documented gaps), 81 waste sites (46 in-region), 1,973 major + 82,941
minor road features, 99 unclosed boundary rings dropped (counted, not silently ignored).

The registry, payment, tracking and dashboard modules follow in Chapters Four and Five.

## 3.7 Testing Phase

Testing combined: unit-level verification inside the Rust pipeline (node-hit accounting —
28,526,968 of 28,526,968 needed coordinates found; ring closure and area checks;
point-in-polygon self-tests against known locations such as the Sinza ward relation
6505583); link verification of every download (HTTP 200 checks recorded in the pack's
link file); and end-to-end browser testing of the zone map module — initial render, zoom
levels (scale bar 10 km → 2 km), ward legend clicks (102 wards verified grouped 36 / 20 /
23 / 14 / 9), facility selection with zone highlight, road detail cards, layer toggles,
hover tooltips, minor-roads loading and the mobile viewport — each verified with
screenshots and console-error checks. Known limitations recorded during testing (wards
missing in OSM, sites in neighbouring Pwani within the bbox, scanned-document OCR noise
in GN 154/2008) are listed in the outputs instead of being hidden. The registry, payment
and dashboard modules will be tested in the same pattern — unit checks, browser
end-to-end flows and the user evaluation of Specific Objective 6.

---

## REFERENCES

1. United Republic of Tanzania, *Environmental Management Act*, Cap. 191 (Act No. 20 of
   2004). Pack file: `laws-and-regulations/EMA_2004_Act_No20.pdf`.
2. United Republic of Tanzania, *Environmental Management (Solid Waste Management)
   Regulations*, GN 263 of 2009; amendment GN 190 of 2016. Pack files:
   `laws-and-regulations/GN263_2009_SolidWaste_Regulations.pdf`,
   `GN190_2016_SolidWaste_Amendment.pdf`.
3. United Republic of Tanzania, *Environmental Management (Fees and Charges)
   Regulations*, GN 154 of 2008 (published 5/9/2008; scanned publication, TanzLII
   legislation record 324848). Pack files:
   `laws-and-regulations/GN154_2008_EnvMgmt_FeesAndCharges_Regulations.pdf`,
   `laws-full-text/GN154_2008_EnvMgmt_FeesAndCharges_Regulations.txt`.
4. United Republic of Tanzania, *Environmental Management (Fees and Charges)
   Regulations*, GN 387 of 2021. Pack file:
   `laws-and-regulations/GN387_2021_FeesAndCharges_Regulations.pdf`.
5. Pellatt, J. & Palfreman, J. (2023). Smart technology solution for a cleaner city: a
   case study of Dar es Salaam, Tanzania. *GeoJournal* 88, 5221–5245. DOI
   10.1007/s10708-023-10917-3. Pack extract:
   `full-text-extracts/Pellatt_Palfreman_2023_SmartTRACS_GreenWastePro_DSM.txt`.
6. CSE (2021). *Tanzania: An Assessment of the Solid Waste Management Ecosystem*.
   Pack file: `pdf-reports/CSE_Tanzania_SWM_Ecosystem_2021.pdf`.
7. Ilomo, D. & Charles, K. (2025). Grassroots SWM financing, Dar es Salaam.
   *Environmental Research Communications* 7, 015012. Pack file:
   `pdf-reports/ERC_IlomoCharles_2025_Grassroots_SWM_Financing.pdf`.
8. UN-Habitat (2023). Waste Wise Cities WaCT fact sheet, Dar es Salaam. Pack file:
   `pdf-reports/UNHabitat_WasteWiseCities_DSM_2023.pdf`.
9. Magombola, D.A. (2021). Assessment of Community Participation in Domestic Waste
   Management in Mbagala Ward, Temeke. *European Journal of Research in Social Sciences*
   9(1).
10. Kamugisha, J. (2019). PPP and solid waste: Kinondoni cleansing contractors roster.
    Pack file: `pdf-reports/ROSTER_Kamugisha2019_KinondoniCleansingContractors.pdf`.
11. CAG (2023). Annual General Reports on Local Government Authorities, FY 2021/22 and
    2022/23. Pack files: `pdf-reports/CAG_AnnualGeneralReport_LGAs_2021-22.pdf`,
    `CAG_AnnualGeneralReport_LGAs_2022-23.pdf`.
12. World Bank (2023–2024). DMDP Phase II (P180298) PID and Restructuring Paper. Pack
    files: `pdf-reports/WB_PID_2023_DSM_SolidWaste.pdf`, `WB_PAD_2024_DSM_ClimateResilience.pdf`.
13. OpenStreetMap contributors. OSM Tanzania extract, Geofabrik
    `tanzania-261122.osm.pbf` (ODbL). https://download.geofabrik.de/africa/tanzania.html
14. United Republic of Tanzania, *Occupational Safety and Health Act*, 2003;
    *Road Traffic Act*, Cap 168; GN 177 of 2001 (vehicle inspection);
    *Personal Data Protection Act*, 2022.
15. *Bus Tracking System — School Bus Tracking Mobile Based Application* (final year
    project report, College of ICT, UDSM) — formatting and chapter-structure model for
    this report.

---

*Honesty note: all figures in this report are traceable to files in the report pack
(`reports/`), to the generated GeoJSON `summary.json`, or to the cited laws. The report
describes a system to be developed: modules not yet implemented (registry, payment,
tracking, dashboard) are labelled as designed or planned, not delivered. Items that could
not be verified (e.g. OCR-noisy numerals in the scanned GN 154/2008) are presented with
their source and flagged rather than silently corrected.*
