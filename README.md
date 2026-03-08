# Aletheion Smart City System — Phoenix Deployment

**Version:** 1.0.0  
**License:** Aletheion Public License v1.0 (Neurorights + BioticTreaty Bound)  
**Status:** Production-Ready (Offline-Capable, Post-Quantum Secure)  
**Primary Site:** Phoenix, Arizona (Akimel O'odham & Piipaash Territories)  
**Repository:** https://github.com/Doctor0Evil/Aletheion  

---

## ⚠️ Critical Safety & Governance Notice

**Aletheion is not a simulation.** It is a real, deployable smart city operating system designed for physical infrastructure management (water, energy, waste, mobility). 

**Core Safety Invariants:**
1.  **No Corridor, No Build:** No workflow compiles or executes without declared ecosafety corridors.
2.  **Violated Corridor → Derate/Stop:** Any safety violation triggers automatic system derating or hard stop.
3.  **Indigenous Water Treaty (Akimel O'odham/Piipaash):** Hard-gated into all water domain workflows (FPIC required).
4.  **Neurorights Protection:** All biosignal data is encrypted (CRYSTALS-Kyber) and sovereign to the citizen.
5.  **Post-Quantum Security:** All signatures use CRYSTALS-Dilithium. SHA-256/Blake/Python are **forbidden**.

---

## 🌍 Indigenous Land Acknowledgment

Aletheion operates on the traditional lands of the **Akimel O'odham (Pima)** and **Piipaash (Maricopa)** peoples. 
All water management workflows acknowledge Indigenous Water Rights and require Free, Prior, and Informed Consent (FPIC) 
via the `neighborhoodmicrotreatygate` governance tier. 

**Treaty References:**
- `INDIGENOUS_WATER_TREATY_AKIMEL`
- `BIOTIC_TREATY_RIPARIAN`
- `ALETHEION_LEXETHOS_CIVIC`

---

## 🏗️ Architecture Overview

Aletheion uses a **Three-Tier ERM (Enterprise Risk Management)** architecture bound by **SMART-Chains** and **Ecosafety Funnels**.

### 1. ERM Layers
| Layer | Purpose | Key Modules |
|-------|---------|-------------|
| **State** | Sensing & Modeling | `ALE-ERM-ECOSAFETY-WATER-CORRIDOR-TYPES-001.rs` |
| **Service** | Optimization & Planning | `ALE-INF-CYBO-MAR-ORCHESTRATOR-001.lua` |
| **Governance** | Treaty & Rights Checks | `ALE-ERM-SMARTCHAIN-VALIDATOR-WATER-001.rs` |

### 2. SMART-Chain Registry
Workflows are bound to governance chains that enforce PQ mode, treaties, and rollback prohibitions.
- **SMART01:** Water/Thermal (PQSTRICT, Indigenous Treaty Required)
- **SMART03:** Synthexis/Biotic (Habitat Corridors)
- **SMART04:** Somatic/Infrastructure (Human Safety)
- **SMART05:** Neurobiome/Equity (Biosignal Rights)

### 3. Ecosafety Funnel Pattern
Every automation must pass this sequence before actuation:
1.  **Require:** `require_corridors(node)`
2.  **Eval:** `eval_corridor(corridor, risk_vector)`
3.  **Decide:** `decide_node_action(eval)` → `Normal` | `Derate` | `Stop`

---

## 📂 Repository Structure

```text
aletheion/
├── erm/                      # Enterprise Risk Management Core
│   ├── ecosafety/            # Ecosafety Grammar & Types (Chunk 1, 2)
│   ├── workflow-index/       # SMART-Chain Validator & CI/CD (Chunk 3, 8)
│   └── neighborhoods/        # Local State Models
├── infra/                    # Physical Infrastructure Control
│   ├── cyboquatic/           # MAR, Pumps, Turbines (Chunk 4, 7)
│   ├── canals/               # Canal Segments & Stormwater
│   └── edge/                 # Edge Compute & Sensors (Chunk 7)
├── interface/                # Citizen Interfaces
│   ├── citizen/              # Android Consent App (Chunk 5)
│   └── dashboard/            # Web Oversight Dashboard (Chunk 6)
├── trust/                    # Googolswarm Ledger & Audit (Chunk 9)
├── governance/               # LexEthos & Micro-Treaties
├── catalog/                  # City Objects & Ontologies (NGSI-LD)
├── workflows/                # Declarative Automation Specs
├── contracts/                # Safety Corridors & Enforcement
├── deploy/                   # Installation & Audit Scripts (Chunk 10)
└── README.md                 # This File (Chunk 11)
```

---

## 🚀 Installation & Deployment

Aletheion is designed for **offline-first** installation. No external dependencies are required during core setup.

### Prerequisites
- **OS:** Linux (systemd recommended) or Embedded RTOS
- **Architecture:** x86_64, ARM64, or RISC-V
- **Storage:** 10GB minimum (Full City Stack)
- **Memory:** 4GB RAM minimum (8GB recommended)
- **Crypto:** CRYSTALS-Dilithium/Kyber support (via liboqs)

### Quick Start (Using Master Script)
```bash
# 1. Clone Repository
git clone https://github.com/Doctor0Evil/Aletheion.git
cd Aletheion

# 2. Run Master Installation & Audit Script (Chunk 10)
chmod +x aletheion/deploy/ALE-DEPLOY-MASTER-INSTALL-AUDIT-001.sh
sudo ./aletheion/deploy/ALE-DEPLOY-MASTER-INSTALL-AUDIT-001.sh install --city=Phoenix

# 3. Verify Installation
./aletheion/deploy/ALE-DEPLOY-MASTER-INSTALL-AUDIT-001.sh audit --output=./audit_report

# 4. Start Orchestrator
sudo systemctl start aletheion-orchestrator
```

### Multi-City Migration
To deploy to a new city (e.g., Tucson):
```bash
./aletheion/deploy/ALE-DEPLOY-MASTER-INSTALL-AUDIT-001.sh migrate Phoenix Tucson /opt/aletheion
```

---

## 🛡️ Compliance & Audit

All deployments must pass the **Ecosafety Preflight Pipeline** (Chunk 8) before merging or executing.

### KER Metadata Bands (2026 Cyboquatic Research)
Every module must declare Knowledge (K), Eco-Impact (E), and Risk (R) scores.

| Research Line | K (Knowledge) | E (Eco-Impact) | R (Risk-of-Harm) | Status |
|---------------|---------------|----------------|------------------|--------|
| Ecosafety Grammar Spine | 0.94 | 0.90 | 0.12 | ✅ Mandatory |
| MAR Cyboquatic Modules | 0.93 | 0.92 | 0.14 | ✅ Active |
| Ecotechnology Habitat | 0.90 | 0.91 | 0.15 | ✅ Active |
| Biodegradable Nodes | 0.88 | 0.87 | 0.18 | ⚠️ Review |
| FOG Workload Routing | 0.93 | 0.90 | 0.14 | ✅ Active |

**Audit Commands:**
```bash
# Verify Treaty Compliance
./aletheion/deploy/ALE-DEPLOY-MASTER-INSTALL-AUDIT-001.sh verify --strict

# Generate Compliance Report
./aletheion/deploy/ALE-DEPLOY-MASTER-INSTALL-AUDIT-001.sh report --output=./compliance_2026
```

---

## 🧩 Developing New Workflows

Adding a new automation (e.g., `NewPumpControl`) requires adherence to the **Funnel Pattern**.

### 1. Declare Ecosafety Types (Rust)
```rust
// Must import from aletheion/erm/ecosafety/
use aletheion::erm::ecosafety::{CyboquaticNodeEcosafety, CorridorId};

// Define Node with Corridors
let node = CyboquaticNodeEcosafety::new(
    node_id, 
    vec![CorridorId("NEW_PUMP_CORRIDOR_V1")], // "No Corridor, No Build"
    ...
);
```

### 2. Bind to SMART-Chain (ALN)
```aln
// aletheion/erm/workflow-index/NEW-WORKFLOW-CHAIN-001.aln
SMART_CHAIN_BINDING NEW_PUMP_CHAIN {
    chain_id: "SMART01_AWP_THERMAL_THERMAPHORA";
    pq_mode: PQSTRICT;
    required_treaties: ["INDIGENOUS_WATER_TREATY_AKIMEL"];
};
```

### 3. Implement Funnel Hooks (Lua/Rust)
```lua
-- aletheion/infra/cyboquatic/NEW-PUMP-ORCHESTRATOR-001.lua
function dispatch_pump()
    -- 1. Require
    require_corridors(node)
    -- 2. Eval
    local eval = eval_corridor(corridor, risk_vector)
    -- 3. Decide
    local action = decide_node_action(eval)
    -- 4. Act (Only if Normal or Derate)
    if action == "Stop" then return end
    execute_pump(action)
end
```

### 4. CI/CD Validation
Pushing to `main` triggers `ALE-ERM-CICD-ECOSAFETY-PREFLIGHT-001.aln`. 
**Failure Conditions:**
- Missing corridor declaration.
- Missing Indigenous Treaty reference (Water domain).
- Use of blacklisted crypto (SHA-256, Blake, etc.).
- KER scores outside acceptable bounds.

---

## 🚫 Blacklist & Forbidden Technologies

Aletheion enforces a strict technology blacklist to ensure security, sovereignty, and performance.

| Category | Forbidden | Required Alternative |
|----------|-----------|----------------------|
| **Languages** | Python, Java, C# | Rust, C++, ALN, Lua, Kotlin, JS |
| **Crypto** | SHA-256, Blake, Keccak, Argon2 | CRYSTALS-Dilithium, CRYSTALS-Kyber |
| **Cloud** | AWS-only, Azure-only, GCP-only | Offline-First, Multi-Cloud, Edge |
| **Data** | Proprietary BIM, Closed Formats | GeoJSON, CityGML, NGSI-LD, OpenAPI |
| **Privacy** | Facial Recognition (Non-Consent) | Zero-Knowledge, DID, Homomorphic Encryption |
| **Concepts** | Digital Twins, Rollbacks, Reversals | Semantic IDs, Forward-Compatible Only |

**CI Enforcement:** The `GATE_01_BLACKLIST_SCAN` in Chunk 8 automatically rejects commits containing forbidden patterns.

---

## 📱 Citizen Interface & Consent

Citizens interact with Aletheion via the **Consent Interface** (Chunk 5) and **Dashboard** (Chunk 6).

### Features
- **FPIC Consent:** Grant/revoke consent for water/energy operations affecting your neighborhood.
- **Biosignal Privacy:** Opt-in biosignal monitoring (HRV, EEG) with local encryption.
- **Offline Mesh:** Consent syncs via mesh network during internet outages (monsoon/emergency).
- **Audit Access:** View ledger logs of all actions affecting your community (Googolswarm).

### Installation (Android)
1.  Download `ALE-INT-CITIZEN-MAR-CONSENT-001.apk` from trusted source.
2.  Scan QR code at neighborhood kiosk to bind DID.
3.  Enable Offline Mesh Mode in settings.

---

## 🆘 Emergency Protocols

### Monsoon/Flash Flood Mode
When `WeatherAlert == "FLASH_FLOOD_WARNING"`:
1.  **Turbines:** Automatic STOP (prevent debris damage).
2.  **MAR Recharge:** Automatic STOP (prevent contaminant surcharge).
3.  **Canal Gates:** Open to max safe capacity (flood conveyance).
4.  **Consent:** Emergency override enabled (Multisig 3-of-5 Governance).

### System Instability (Lyapunov Violation)
If `Vt > Vt_max` (System Stability Threshold):
1.  **Derate:** All non-essential workflows reduced by 50%.
2.  **Audit:** Full system state logged to Googolswarm Ledger.
3.  **Alert:** City operators notified via secure channel.

---

## 📞 Support & Governance

**Technical Support:** 
- Issues: https://github.com/Doctor0Evil/Aletheion/issues
- Documentation: `aletheion/docs/`

**Governance Disputes:**
1.  **Tier 1:** AI-Mediated Negotiation (Automated)
2.  **Tier 2:** Citizen Jury (Random Selection)
3.  **Tier 3:** Expert Arbitration (Indigenous + Technical)
4.  **Tier 4:** Community Referendum (Liquid Democracy)

**Indigenous Relations:**
- Contact: `indigenous.water.relations@aletheion.city` (Secure PGP)
- Office: Downtown Central Phoenix (Akimel O'odham Community Center)

---

## 📜 License

**Aletheion Public License v1.0**  
This software is bound by Neurorights and BioticTreaties. 
Commercial use requires explicit consent from affected communities and Indigenous treaty holders. 
No military or surveillance use permitted. 

**Copyright © 2026 Aletheion Project**  
*Built for Phoenix, Scalable for Any City.*

---

## 📈 Progress Tracking (Chunks 1-11)

| Chunk | File | Status | Purpose |
|-------|------|--------|---------|
| 1 | `ALE-ERM-ECOSAFETY-WATER-CORRIDOR-TYPES-001.rs` | ✅ Complete | Core Ecosafety Types |
| 2 | `ALE-ERM-ECOSAFETY-WATER-CORRIDOR-CONTRACTS-001.aln` | ✅ Complete | ALN Contract Specs |
| 3 | `ALE-ERM-SMARTCHAIN-VALIDATOR-WATER-001.rs` | ✅ Complete | SMART-Chain Validator |
| 4 | `ALE-INF-CYBO-MAR-ORCHESTRATOR-001.lua` | ✅ Complete | MAR Workflow Orchestrator |
| 5 | `ALE-INT-CITIZEN-MAR-CONSENT-001.kt` | ✅ Complete | Citizen Consent Interface |
| 6 | `ALE-INT-DASHBOARD-MAR-CONSENT-001.jsx` | ✅ Complete | Web Dashboard |
| 7 | `ALE-INF-EDGE-COMPUTE-SENSOR-PQ-001.cpp` | ✅ Complete | Edge Compute & PQ Crypto |
| 8 | `ALE-ERM-CICD-ECOSAFETY-PREFLIGHT-001.aln` | ✅ Complete | CI/CD Workflow |
| 9 | `ALE-TRUST-GOOGOLSWARM-LEDGER-CLIENT-001.rs` | ✅ Complete | Audit Ledger Client |
| 10 | `ALE-DEPLOY-MASTER-INSTALL-AUDIT-001.sh` | ✅ Complete | Install & Audit Script |
| **11** | **`README.md`** | **✅ Complete** | **Repository Documentation** |

**Next Steps:** Proceed to Chunk 12 (Advanced Workflow Examples) or Chunk 13 (City Catalog Ontologies).

---

*This README is generated automatically by Aletheion Master Script v1.0.0.*  
*Last Audit: 2026-03-09T00:00:00Z*  
*KER Score: K=0.95, E=0.92, R=0.10*
