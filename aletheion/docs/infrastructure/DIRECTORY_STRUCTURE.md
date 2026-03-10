# Aletheion Directory Structure Map

* **Projected Files:** 3,000+
* **Current Files:** 34 (Core Complete)
* **Depth:** 10+ Levels
* **Context:** Phoenix, Arizona

## Root Structure
```
aletheion/
├── README.md
├── LICENSE
├── CONTRIBUTING.md
├── core/
│   ├── environmental/      (400+ files: Water, Air, Heat, Mar, Wetlands)
│   ├── energy/             (300+ files: Solar, Battery, Mesh, Grid)
│   ├── waste/              (200+ files: Lifecycle, Recycling, Disposal)
│   ├── transport/          (300+ files: Routing, Mobility, AV, Freight)
│   ├── agriculture/        (200+ files: Vertical, Water, Climate, Crops)
│   ├── health/             (300+ files: Biosignal, Privacy, Medical, BCI)
│   ├── governance/         (300+ files: Voting, Contracts, Dispute, Law)
│   ├── security/           (400+ files: Encryption, Threat, Zero-Knowledge)
│   ├── optimization/       (200+ files: Predictive, Maintenance, AI)
│   └── emergency/          (100+ files: Haboob, Flood, Fire, Evac)
├── interface/
│   ├── citizen/            (150+ files: Android, Web, Feedback, Dashboard)
│   ├── governance/         (50+ files: Voting UI, Proposal View)
│   └── health/             (50+ files: Biosignal UI, Consent Mgmt)
├── deployment/
│   ├── installer/          (50+ files: Lua Scripts, Configs, Backups)
│   ├── config/             (50+ files: Phoenix Profile, Network, Keys)
│   └── workflow/           (100+ files: Automation, CI/CD, Tests)
└── docs/
    ├── adr/                (50+ files: Architectural Decision Records)
    ├── standards/          (20+ files: Production Protocols, STD_001/002)
    ├── infrastructure/     (20+ files: Directory Maps, Install Guides)
    └── research/           (50+ files: Phoenix Data, Indigenous Protocols)
```

## File Naming Convention
- **Format:** `snake_case.rs`, `snake_case.aln`, `snake_case.lua`
- **Prefix:** Subsystem abbreviation (e.g., `env_risk_engine.rs`, `gov_voting_contract.aln`)
- **Version:** Included in file header comments (e.g., `V1.0.0`)
- **Uniqueness:** Hash-checked to prevent duplicates.

## Indexing Strategy
- **Deep Paths:** Files placed in deepest relevant directory (e.g., `core/environmental/water/mar/vault/src/`).
- **Tags:** Each file header includes K/E/R scores, Chain (ERM/SMART), and Rights tags.
- **Search:** Optimized for grep/ripgrep across headers for compliance auditing.

## Growth Plan
- **Phase 1 (Core):** 34 files (Complete).
- **Phase 2 (Subsystem):** 500 files (Environmental, Energy, Security).
- **Phase 3 (Interface):** 1,000 files (Citizen Apps, Dashboards).
- **Phase 4 (Scale):** 3,000+ files (Full City Deployment).

---
*Structure designed for offline indexing and rapid compliance auditing.*
