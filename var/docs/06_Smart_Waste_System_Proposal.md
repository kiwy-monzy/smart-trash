# 06 - PROPOSED SMART WASTE COLLECTION SYSTEM (THE NEW RECOMMENDED SYSTEM)
## A Smart Waste Collection System for Dar es Salaam

**Working name: "Dar Smart Waste"**

---

## 1. WHY "SMART"? (WHAT MAKES IT SMART)

The current system is **blind** - no household data, no proof of service, no
tonnage measurement, no sorting incentives. The smart system turns every step
into **visible, usable data**:

```
SMART =  S ensing      (bin sensors, truck GPS, weighbridges)
         M obile       (app + USSD + mobile money - everyone included)
         A utomation   (automatic billing, route optimization, alerts)
         R eporting    (one city dashboard, contract KPIs)
         T raceability (every waste stream traced from household to landfill)
```

**The 9 "smart" elements and what each fixes:**

| # | Smart feature | Serves | Fixes problem (doc 02) |
|---|---------------|--------|------------------------|
| 1 | **QR-coded bins/households** - every bin/household has an identity | Citizens, Contractor, Council | No customer registry (#1); no proof-of-service (#6) |
| 2 | **Citizen app + USSD `*xxx#`** - schedule, complaints, payments | Citizens | No complaint system (#10); low RCC compliance (#1) |
| 3 | **IoT fill sensors on public/commercial bins** - "I am full" alerts | Council, Contractor | Trucks waste time on empty bins; overflowing points |
| 4 | **GPS + weighbridge on trucks** - every trip has weight + route | Council, Contractor | No tonnage data (#8); unverified work |
| 5 | **Automatic RCC billing via mobile money** (M-Pesa, Tigo Pesa, Airtel Money, Halopesa) | Households, Council, Contractor | Cash payments (#2); 30-day disbursement delays (#3) |
| 6 | **AI route optimization** - shortest efficient collection routes | Contractor | Fuel costs; unreliable timing |
| 7 | **Sorting incentives** - points for separated plastics/organics | Citizens | No sorting (#7); ~1% recovery |
| 8 | **City GIS dashboard** - illegal dumping heatmap, coverage, tonnage, GHG | Council, NEMC, PORALG | No NEMC-LGA data sharing (CAG finding); weak M&E |
| 9 | **KPI-based digital contracts** - contractor payments tied to verified KPIs | Council, Contractor | Disbursement delays (#3) + no proof (#6) |

## 2. SYSTEM ARCHITECTURE

### 2.1 Four Layers
```
┌─────────────────────────────────────────────────────────────┐
│  LAYER 4: USER APPLICATIONS                                 │
│  |-- Citizen App (Android/iOS) + USSD *xxx#                 │
│  |-- Business Portal (accounts, bins, payments)             │
│  |-- Contractor App (routes, proof-of-collection, e-RCC)    │
│  |-- Council/City Dashboard (GIS, KPIs, complaints)         │
├─────────────────────────────────────────────────────────────┤
│  LAYER 3: PLATFORM (cloud backend)                          │
│  |-- Customer Registry (households, businesses, mtaa, zone) │
│  |-- Billing & Payments Engine (RCC, mobile money, receipts)│
│  |-- Route Optimization Engine (AI/ML)                      │
│  |-- Complaint/CRM Workflow (complaint -> contractor task)  │
│  |-- Contract & KPI Module (contractor performance scoring) │
│  |-- Data Lake (tonnes, GPS, sensors, complaints, GHG)      │
├─────────────────────────────────────────────────────────────┤
│  LAYER 2: FIELD HARDWARE                                    │
│  |-- QR codes on bins/households + scanners on crew phones  │
│  |-- GPS trackers on all contractor trucks                  │
│  |-- IoT fill sensors (public bins, businesses, points)     │
│  |-- Weighbridges (transfer stations + landfill)            │
│  |-- Camera/photo verification of collection                │
├─────────────────────────────────────────────────────────────┤
│  LAYER 1: MONEY & LAW INTEGRATIONS                          │
│  |-- M-Pesa / Tigo Pesa / Airtel / Halopesa APIs            │
│  |-- Municipal OSR Platform (exists from DMDP I)            │
│  |-- Inter-municipal company accounts (DMDP2)               │
│  |-- NEMC reporting API (closes the CAG data-sharing gap)   │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 A Typical Day of Data Flow
1. **6:00 AM**: the route engine pushes today's route to Zone X trucks -
   optimized using sensor fill levels and yesterday's complaints.
2. **7:30 AM**: the crew reaches Umoja street and scans the household QR ->
   the system records "household #1234 collected 07:32" + a bin photo. If the
   bin is absent, the crew records "missed" - the household gets an SMS/USSD
   notice.
3. **10:00 AM**: the truck arrives at the transfer station - the
   **weighbridge** records "today's trip: 6.2 tonnes from Zone X". Data hits
   the dashboard.
4. **12:00 PM**: 5 open complaints from yesterday are auto-injected into
   today's route.
5. **Month-end**: the billing engine sends RCC e-invoices to every Zone X
   household (SMS + app + USSD). Payments land via mobile money ->
   **auto-split: contractor share + council share**, instantly. The 30-day
   delay ceases to exist.
6. **Month-end (2)**: the KPI module scores every contractor: coverage %,
   tonnes collected, complaints closed, RCC collection % - this score drives
   payment, renewal, or termination.

## 3. USER JOURNEYS PER STAKEHOLDER

### 3.1 Citizens / Households
**What they get:**
- **USSD `*xxx#` (no internet needed)** and/or an **App**:
  1. *Schedule* - "your waste is collected Monday & Thursday"
  2. *Pay RCC* - directly via mobile money; instant SMS receipt
  3. *Report* - "waste not collected today" / "skip overflowing" - photo + GPS
  4. *Learn* - 3R tips in Kiswahili
  5. *Points* - earn points for separated recyclables
- **QR on their bin/bag** - their identity in the system

**"Smart" for the citizen:**
- No more "where did my money go" - a digital receipt every month
- Complaints get a **ticket number and a 48-hour service-level target**
- Recycling points accumulate - redeemable against next month's RCC
  (discount) or goods (via a waste bank)

### 3.2 Businesses (shops, hotels, factories, hospitals)
**What they get:**
- **Account portal**: bin types, monthly weight data, invoices, schedules
- **IoT bins for larger businesses** - request pickup automatically at 80% fill
- **Compliance dashboard** - proof of legal disposal for NEMC inspections
- **Automatic EPR reporting** - plastic producers can demonstrate their waste
  enters the recycling chain (the CAG flagged EPR as unenforced - this makes
  it easy)

**"Smart" for business:**
- No more "estimated" charges - **real weight data** (weighbridge)
- Organic sorting (hotels, food businesses) -> council compost -> **lower RCC
  rates** (incentive)

### 3.3 Contractors
**What they get:**
- **Contractor app**: AI-optimized daily routes (20-30% fuel savings), QR
  scanning per household/bin, photo proof, mobile-money e-RCC collection
- **KPI dashboard** - see weak spots before the council does
- **Instant revenue split** - the contractor share lands directly in its bank
  account from each RCC payment - **no more waiting 30 days**
- **Digital contracts** - KPIs, renewals and penalties all visible

**"Smart" for the contractor:**
- Verified recurring revenue data -> **bank creditworthiness** -> ability to
  finance more trucks
- No more disputes with the council about work done - the data speaks

### 3.4 Councils / City (Municipalities + DCC + Inter-municipal Company)
**What they get:**
- **One city GIS dashboard**: coverage heatmap, illegal dumping, complaints,
  tonnes per zone, cost per tonne
- **The official customer registry** - 1.55M+ registered households (VPO
  identified the missing registry as a core problem)
- **KPI-based contract management** - automatic contractor scoring
- **Data-sharing APIs with NEMC/PORALG** - closing the CAG finding
- **GHG tracker** - methane captured, fleet emissions - valuable for climate
  finance (carbon credits)

**"Smart" for the council:**
- Contractor payments follow **data-verified KPIs** -
  performance-based contracting
- Tariff-setting stops being guesswork - **real cost-per-ton data**

## 4. IMPLEMENTATION ROADMAP

### Phase 0: Foundation (Months 0-6)
- Register households in 2 pilot zones per municipality (10 zones total)
- QR codes on bins/bags; crew phones with the pilot app
- USSD + mobile money integration for RCC billing
- Crew training (PPE + data) - incorporating REPOA findings

### Phase 1: Pilot (Months 6-18)
- GPS on all pilot-zone trucks
- Weighbridge data from the first DMDP2 transfer station
- KPI-based contract amendments with pilot contractors
- Points/recycling incentive pilot (with waste pickers and the GAIA/NALI
  zero-waste groups)

### Phase 2: Scale (Years 2-3)
- Full coverage across all 5 municipalities
- IoT sensors on public collection points + large businesses
- One unified city dashboard (DCC + inter-municipal company)
- NEMC/PORALG data-sharing APIs

### Phase 3: Deepen (Years 3-5)
- Full-scale AI route optimization
- Carbon credits / climate finance reporting
- Waste-to-energy data integration (landfill gas from DMDP2 landfills)
- Recyclables marketplace (waste bank -> processors)

### Start-up Costs (ESTIMATE - order-of-magnitude)
| Item | Est. US$ |
|------|----------|
| Platform backend + apps + USSD | 250k-500k (one-off) + 50k/yr ops |
| QR codes + crew phones + scanners | 100k-150k |
| GPS trackers (300+ trucks) | 60k-90k + 20k/yr |
| IoT sensors (1,000 bins) | 150k-300k + connectivity |
| Weighbridges (already in DMDP2 scope) | (already funded) |
| Training + change management | 100k-200k |
| **Total start-up** | **~US$ 700k - 1.2M** (< 1% of DMDP2 Component 2) |

> Against the US$117M Component 2 budget, digitization costs **<1%** while
> enabling 20-40% collection-cost savings (route efficiency) plus higher RCC
> collection rates (formalized billing).

## 5. RISKS AND MITIGATIONS

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Vandalism of sensors/GPS | Medium | Low-cost sensors; GPS inside cabins; community ownership via mtaa committees |
| Low digital literacy | Medium | **USSD for everything** (no internet needed); radio campaigns; street agents |
| Connectivity gaps | Medium | Offline-first apps (sync later); SMS fallback |
| Contractor resistance to transparency | High | KPI clauses in renewals; successful pilot zones as showcase |
| Cash culture | High | Digital-first incentives: instant receipts; SMS confirmation builds trust |
| Data privacy | Medium | Compliance with the Personal Data Protection Act, 2022; minimal household data (name, phone, payment) |
| Power outages | High | Solar-powered sensors; offline caching |

## 6. SUCCESS KPIs (5-YEAR TARGETS)

| KPI | Now (2024) | Target (5 yrs) |
|-----|------------|----------------|
| Waste collected | ~36-40% | **70%+** |
| Households registered in the system | 0 (no registry) | **80%+ of serviced zones** |
| RCC collected digitally | <10% (cash-based) | **60%+** |
| Median complaint resolution time | no system | **48 hours** |
| Recycling/recovery rate | ~1% | **10-15%** |
| Municipal cost per tonne | no data | **-25% (route + billing efficiency)** |
| Contracts scored by verified KPIs | none | **100% of contracts** |
| Waste workers with PPE + training | 50% untrained | **100%** |

## 7. THE WHOLE "SMART" IN 5 SENTENCES

1. **Every household/bin has a QR** -> every collection has proof
   (photo + GPS + timestamp).
2. **RCC is paid by phone** (app/USSD/mobile money) -> receipts for everyone,
   auto-split between contractor and council - **the 30-day delay dies**.
3. **Trucks carry GPS + weighbridges** -> the council pays for **verified
   tonnes**, not promises.
4. **AI plans routes** -> 20-30% less fuel, wider coverage - including
   low-income areas.
5. **One city dashboard** -> NEMC, PORALG and all 5 municipalities see the
   same data - closing the CAG gap for good.

---

*Built on doc 02 (problems) and doc 04 (current system). References: doc 07.*
