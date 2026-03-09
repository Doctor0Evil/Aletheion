// Aletheion :: Phoenix GOD-City
// Purpose: Offline-capable citizen/augmented-citizen panel for viewing corridor states,
// submitting consent/constraints, and subscribing to workflow notifications.
// Derived from ERM architecture, GOD-City workflows, and neurorights/Synthexis interfaces.[file:29][file:30]

#include <iostream>
#include <string>
#include <vector>
#include <map>
#include <optional>
#include <functional>

// --------- Core Types ---------

enum class CorridorTier {
    STATE,
    SERVICE,
    GOVERNANCE
};

enum class WorkflowDomain {
    WATER,
    HEAT,
    WASTE,
    MOBILITY,
    BIOSAFETY,
    GOVERNANCE_SIGNAL
};

enum class ConsentScope {
    NONE,
    VIEW_ONLY,
    PARTICIPATE_ANONYMIZED,
    PARTICIPATE_IDENTIFIED
};

enum class ConstraintType {
    RATE_LIMIT,
    TIME_WINDOW,
    GEOfENCE,
    BIOMETRIC_EXCLUSION,
    DATA_RETENTION
};

enum class NotificationChannel {
    LOCAL_ONLY,
    EDGE_MESH,
    CLOUD_MIRRORED
};

struct CorridorId {
    std::string city;        // e.g. "Phoenix"[file:27][file:29]
    std::string code;        // e.g. "PHX-CANAL-NE-07"
    CorridorTier tier;
};

struct WorkflowId {
    std::string name;        // e.g. "phx_awp_canal_recharge_v1"[file:29]
    WorkflowDomain domain;
    CorridorId corridor;
};

struct Constraint {
    ConstraintType type;
    std::string value;       // free-form, interpreted by governance engine (e.g. "max_3_events_per_day")
};

struct ConsentRecord {
    WorkflowId workflow;
    ConsentScope scope;
    std::vector<Constraint> constraints;
    bool neurorights_safe;   // pre-screened by LexEthos/Synthexis modules.[file:29][file:30]
};

struct NotificationPreference {
    WorkflowId workflow;
    NotificationChannel channel;
    bool critical_only;
};

struct CorridorState {
    WorkflowId workflow;
    std::string status;              // "idle", "running", "degraded", "paused_by_treaty"
    std::map<std::string, double> indicators; // KER, heat_index, flow_rate, etc.[file:29]
};

// --------- Local-Only Storage Shells ---------
// In real deployment these bind to an embedded KV-store or secure file.[file:30]

class LocalConsentStore {
public:
    void upsert(const ConsentRecord &record) {
        std::string key = record.workflow.name + "@" + record.workflow.corridor.code;
        records_[key] = record;
    }

    std::optional<ConsentRecord> get(const WorkflowId &wf) const {
        std::string key = wf.name + "@" + wf.corridor.code;
        auto it = records_.find(key);
        if (it == records_.end()) return std::nullopt;
        return it->second;
    }

    std::vector<ConsentRecord> all() const {
        std::vector<ConsentRecord> out;
        out.reserve(records_.size());
        for (const auto &kv : records_) out.push_back(kv.second);
        return out;
    }

private:
    std::map<std::string, ConsentRecord> records_;
};

class LocalNotificationStore {
public:
    void upsert(const NotificationPreference &pref) {
        std::string key = pref.workflow.name + "@" + pref.workflow.corridor.code;
        prefs_[key] = pref;
    }

    std::optional<NotificationPreference> get(const WorkflowId &wf) const {
        std::string key = wf.name + "@" + wf.corridor.code;
        auto it = prefs_.find(key);
        if (it == prefs_.end()) return std::nullopt;
        return it->second;
    }

    std::vector<NotificationPreference> all() const {
        std::vector<NotificationPreference> out;
        out.reserve(prefs_.size());
        for (const auto &kv : prefs_) out.push_back(kv.second);
        return out;
    }

private:
    std::map<std::string, NotificationPreference> prefs_;
};

// --------- Neurorights & Treaty Guardrails ---------

class NeurorightsGuard {
public:
    bool evaluate(const WorkflowId &wf, ConsentScope scope, const std::vector<Constraint> &constraints) const {
        // Ultra-conservative local checker: never allows scope escalation for governance-signal or biosafety
        // without explicit corridor treaty flags.[file:29]
        if (wf.domain == WorkflowDomain::BIOSAFETY || wf.domain == WorkflowDomain::GOVERNANCE_SIGNAL) {
            if (scope != ConsentScope::VIEW_ONLY) return false;
        }

        // Enforce presence of at least one data retention constraint if not view-only.
        if (scope == ConsentScope::PARTICIPATE_ANONYMIZED || scope == ConsentScope::PARTICIPATE_IDENTIFIED) {
            bool hasRetention = false;
            for (const auto &c : constraints) {
                if (c.type == ConstraintType::DATA_RETENTION) {
                    hasRetention = true;
                    break;
                }
            }
            if (!hasRetention) return false;
        }

        // Offline panel errs on the side of under-permissioning.
        return true;
    }
};

// --------- Corridor Snapshot Provider (Edge-Only Stub) ---------

class CorridorSnapshotProvider {
public:
    // In real system this reads from ERM edge cache; here we expose a stub.[file:30]
    std::vector<CorridorState> list_states(const std::vector<WorkflowDomain> &domains) const {
        std::vector<CorridorState> states;
        for (auto domain : domains) {
            WorkflowId wf{
                .name = demo_workflow_name(domain),
                .domain = domain,
                .corridor = CorridorId{
                    .city = "Phoenix",
                    .code = demo_corridor_code(domain),
                    .tier = CorridorTier::SERVICE
                }
            };
            CorridorState st;
            st.workflow = wf;
            st.status = "running";
            st.indicators = demo_indicators(domain);
            states.push_back(st);
        }
        return states;
    }

private:
    static std::string demo_workflow_name(WorkflowDomain d) {
        switch (d) {
            case WorkflowDomain::WATER: return "phx_canal_recharge_daily";
            case WorkflowDomain::HEAT: return "phx_heat_tree_mitigation";
            case WorkflowDomain::WASTE: return "phx_waste_corridor_balance";
            case WorkflowDomain::MOBILITY: return "phx_mobility_calm_routes";
            case WorkflowDomain::BIOSAFETY: return "phx_cyboquatic_safestep";
            case WorkflowDomain::GOVERNANCE_SIGNAL: return "phx_governance_signal_window";
        }
        return "unknown";
    }

    static std::string demo_corridor_code(WorkflowDomain d) {
        switch (d) {
            case WorkflowDomain::WATER: return "PHX-CANAL-NE-07";
            case WorkflowDomain::HEAT: return "PHX-CORE-HEAT-01";
            case WorkflowDomain::WASTE: return "PHX-WASTE-LOOP-03";
            case WorkflowDomain::MOBILITY: return "PHX-MOBILITY-NEIGH-02";
            case WorkflowDomain::BIOSAFETY: return "PHX-BIOTIC-CELL-05";
            case WorkflowDomain::GOVERNANCE_SIGNAL: return "PHX-GOV-SIGNAL-01";
        }
        return "PHX-UNKNOWN";
    }

    static std::map<std::string, double> demo_indicators(WorkflowDomain d) {
        std::map<std::string, double> m;
        switch (d) {
            case WorkflowDomain::WATER:
                m["flow_cfs"] = 12.4;
                m["ker_K"] = 0.92;
                m["ker_R"] = 0.14;
                break;
            case WorkflowDomain::HEAT:
                m["heat_index_c"] = 39.5;
                m["canopy_delta_pct"] = 3.1;
                break;
            case WorkflowDomain::WASTE:
                m["load_pct"] = 61.0;
                break;
            case WorkflowDomain::MOBILITY:
                m["avg_travel_time_min"] = 14.2;
                break;
            case WorkflowDomain::BIOSAFETY:
                m["contact_risk"] = 0.03;
                break;
            case WorkflowDomain::GOVERNANCE_SIGNAL:
                m["signal_density"] = 0.7;
                break;
        }
        return m;
    }
};

// --------- CLI-Based Panel Shell ---------

class CitizenAugmentedPanel {
public:
    CitizenAugmentedPanel()
        : guard_(), consent_store_(), notif_store_(), snapshot_provider_() {}

    void run() {
        bool running = true;
        while (running) {
            print_main_menu();
            int choice = read_int();
            switch (choice) {
                case 1: view_corridor_states(); break;
                case 2: manage_consent(); break;
                case 3: manage_notifications(); break;
                case 4: list_local_profile(); break;
                case 0: running = false; break;
                default: std::cout << "Unknown choice.\n"; break;
            }
        }
    }

private:
    NeurorightsGuard guard_;
    LocalConsentStore consent_store_;
    LocalNotificationStore notif_store_;
    CorridorSnapshotProvider snapshot_provider_;

    static int read_int() {
        int v;
        std::cout << "> ";
        std::cin >> v;
        return v;
    }

    static std::string read_line() {
        std::string s;
        std::getline(std::cin >> std::ws, s);
        return s;
    }

    void print_main_menu() {
        std::cout << "\n=== Aletheion Citizen/Augmented Panel (Offline Edge Shell) ===\n";
        std::cout << "1) View corridor states\n";
        std::cout << "2) Manage consent\n";
        std::cout << "3) Manage notifications\n";
        std::cout << "4) Show local profile summary\n";
        std::cout << "0) Exit\n";
    }

    void view_corridor_states() {
        std::vector<WorkflowDomain> domains = {
            WorkflowDomain::WATER,
            WorkflowDomain::HEAT,
            WorkflowDomain::WASTE,
            WorkflowDomain::MOBILITY,
            WorkflowDomain::BIOSAFETY,
            WorkflowDomain::GOVERNANCE_SIGNAL
        };
        auto states = snapshot_provider_.list_states(domains);
        std::cout << "\n--- Corridor States (Local Snapshot) ---\n";
        for (const auto &st : states) {
            std::cout << "[" << st.workflow.corridor.code << "] "
                      << st.workflow.name << " :: status=" << st.status << "\n";
            for (const auto &kv : st.indicators) {
                std::cout << "   - " << kv.first << " = " << kv.second << "\n";
            }
        }
    }

    void manage_consent() {
        std::cout << "\n--- Manage Consent ---\n";
        WorkflowId wf = prompt_workflow_identifier();
        auto existing = consent_store_.get(wf);
        if (existing) {
            std::cout << "Existing consent found. Scope=" << scope_to_string(existing->scope)
                      << " neurorights_safe=" << (existing->neurorights_safe ? "yes" : "no") << "\n";
        } else {
            std::cout << "No consent recorded yet.\n";
        }

        std::cout << "Select new scope:\n";
        std::cout << "0) NONE\n1) VIEW_ONLY\n2) PARTICIPATE_ANONYMIZED\n3) PARTICIPATE_IDENTIFIED\n";
        int s = read_int();
        ConsentScope scope = ConsentScope::NONE;
        if (s == 1) scope = ConsentScope::VIEW_ONLY;
        else if (s == 2) scope = ConsentScope::PARTICIPATE_ANONYMIZED;
        else if (s == 3) scope = ConsentScope::PARTICIPATE_IDENTIFIED;

        std::vector<Constraint> constraints = prompt_constraints();
        bool ok = guard_.evaluate(wf, scope, constraints);
        if (!ok) {
            std::cout << "Requested consent violates neurorights/treaties; keeping previous record.\n";
            return;
        }

        ConsentRecord rec{.workflow = wf, .scope = scope, .constraints = constraints, .neurorights_safe = true};
        consent_store_.upsert(rec);
        std::cout << "Consent stored locally (will sync when a safe Synthexis channel is available).\n";
    }

    void manage_notifications() {
        std::cout << "\n--- Manage Notifications ---\n";
        WorkflowId wf = prompt_workflow_identifier();
        auto existing = notif_store_.get(wf);
        if (existing) {
            std::cout << "Existing preference: channel=" << channel_to_string(existing->channel)
                      << " critical_only=" << (existing->critical_only ? "yes" : "no") << "\n";
        } else {
            std::cout << "No preference recorded yet.\n";
        }

        std::cout << "Select channel:\n";
        std::cout << "0) LOCAL_ONLY\n1) EDGE_MESH\n2) CLOUD_MIRRORED\n";
        int c = read_int();
        NotificationChannel ch = NotificationChannel::LOCAL_ONLY;
        if (c == 1) ch = NotificationChannel::EDGE_MESH;
        else if (c == 2) ch = NotificationChannel::CLOUD_MIRRORED;

        std::cout << "Critical-only notifications? (0=no, 1=yes)\n";
        int crit = read_int();
        bool critical_only = (crit == 1);

        NotificationPreference pref{.workflow = wf, .channel = ch, .critical_only = critical_only};
        notif_store_.upsert(pref);
        std::cout << "Notification preference stored locally.\n";
    }

    void list_local_profile() {
        std::cout << "\n--- Local Profile Summary ---\n";
        std::cout << "Consents:\n";
        for (const auto &c : consent_store_.all()) {
            std::cout << " - " << c.workflow.name << "@" << c.workflow.corridor.code
                      << " :: scope=" << scope_to_string(c.scope)
                      << " neurorights_safe=" << (c.neurorights_safe ? "yes" : "no") << "\n";
        }
        std::cout << "Notifications:\n";
        for (const auto &n : notif_store_.all()) {
            std::cout << " - " << n.workflow.name << "@" << n.workflow.corridor.code
                      << " :: channel=" << channel_to_string(n.channel)
                      << " critical_only=" << (n.critical_only ? "yes" : "no") << "\n";
        }
    }

    static WorkflowId prompt_workflow_identifier() {
        std::cout << "Enter workflow name (e.g. phx_canal_recharge_daily):\n";
        std::string name = read_line();

        std::cout << "Domain: 0=WATER,1=HEAT,2=WASTE,3=MOBILITY,4=BIOSAFETY,5=GOVERNANCE_SIGNAL\n";
        int d = read_int();
        WorkflowDomain domain = WorkflowDomain::WATER;
        if (d == 1) domain = WorkflowDomain::HEAT;
        else if (d == 2) domain = WorkflowDomain::WASTE;
        else if (d == 3) domain = WorkflowDomain::MOBILITY;
        else if (d == 4) domain = WorkflowDomain::BIOSAFETY;
        else if (d == 5) domain = WorkflowDomain::GOVERNANCE_SIGNAL;

        std::cout << "Enter corridor code (e.g. PHX-CANAL-NE-07):\n";
        std::string code = read_line();

        CorridorId corridor{
            .city = "Phoenix",
            .code = code,
            .tier = CorridorTier::SERVICE
        };
        return WorkflowId{.name = name, .domain = domain, .corridor = corridor};
    }

    static std::vector<Constraint> prompt_constraints() {
        std::vector<Constraint> out;
        std::cout << "Add constraints? (1=yes, 0=no)\n";
        int add = read_int();
        while (add == 1) {
            std::cout << "Constraint type: 0=RATE_LIMIT,1=TIME_WINDOW,2=GEOfENCE,3=BIOMETRIC_EXCLUSION,4=DATA_RETENTION\n";
            int t = read_int();
            ConstraintType ct = ConstraintType::RATE_LIMIT;
            if (t == 1) ct = ConstraintType::TIME_WINDOW;
            else if (t == 2) ct = ConstraintType::GEOfENCE;
            else if (t == 3) ct = ConstraintType::BIOMETRIC_EXCLUSION;
            else if (t == 4) ct = ConstraintType::DATA_RETENTION;

            std::cout << "Enter constraint value (e.g. max_3_events_per_day, 22:00-06:00, city_block_1234, 24h):\n";
            std::string val = read_line();
            out.push_back(Constraint{.type = ct, .value = val});

            std::cout << "Add another? (1=yes, 0=no)\n";
            add = read_int();
        }
        return out;
    }

    static const char* scope_to_string(ConsentScope s) {
        switch (s) {
            case ConsentScope::NONE: return "NONE";
            case ConsentScope::VIEW_ONLY: return "VIEW_ONLY";
            case ConsentScope::PARTICIPATE_ANONYMIZED: return "PARTICIPATE_ANONYMIZED";
            case ConsentScope::PARTICIPATE_IDENTIFIED: return "PARTICIPATE_IDENTIFIED";
        }
        return "UNKNOWN";
    }

    static const char* channel_to_string(NotificationChannel c) {
        switch (c) {
            case NotificationChannel::LOCAL_ONLY: return "LOCAL_ONLY";
            case NotificationChannel::EDGE_MESH: return "EDGE_MESH";
            case NotificationChannel::CLOUD_MIRRORED: return "CLOUD_MIRRORED";
        }
        return "UNKNOWN";
    }
};

// --------- Entry Point ---------

int main() {
    CitizenAugmentedPanel panel;
    panel.run();
    return 0;
}
