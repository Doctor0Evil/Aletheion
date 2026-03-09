#ifndef EVENT_COORDINATOR_HPP
#define EVENT_COORDINATOR_HPP

#include <cstdint>
#include <cstddef>
#include <array>
#include <cmath>

constexpr uint32_t EVENT_COORDINATOR_VERSION = 20260310;
constexpr size_t MAX_EVENTS = 32768;
constexpr size_t MAX_VENUES = 2048;
constexpr size_t MAX_PARTICIPANTS = 524288;
constexpr size_t MAX_CULTURAL_ACTIVITIES = 8192;
constexpr double PHOENIX_MAX_OUTDOOR_TEMP_F = 105.0;

enum class EventType : uint8_t {
    CEREMONIAL = 0, EDUCATIONAL = 1, ARTISTIC = 2, ATHLETIC = 3,
    COMMUNITY_MEETING = 4, FESTIVAL = 5, WORKSHOP = 6, MARKET = 7,
    SPIRITUAL = 8, MEMORIAL = 9, CELEBRATION = 10, TRIBAL = 11
};

enum class VenueType : uint8_t {
    INDOOR = 0, OUTDOOR = 1, HYBRID = 2, VIRTUAL = 3,
    SACRED_SITE = 4, COMMUNITY_CENTER = 5, PARK = 6, SCHOOL = 7
};

enum class EventStatus : uint8_t {
    PLANNED = 0, OPEN_REGISTRATION = 1, ACTIVE = 2, COMPLETED = 3,
    CANCELLED = 4, POSTPONED = 5, EMERGENCY_CLOSED = 6
};

struct CommunityEvent {
    uint64_t event_id;
    EventType event_type;
    char event_name[128];
    char description[512];
    uint64_t organizer_did;
    uint32_t venue_id;
    uint64_t start_date_ns;
    uint64_t end_date_ns;
    uint32_t expected_participants;
    uint32_t registered_participants;
    uint32_t max_capacity;
    EventStatus status;
    bool indigenous_led;
    bool tribal_consent_obtained;
    bool accessibility_compliant;
    bool heat_safety_plan;
    bool emergency_protocols;
    double budget_usd;
    double actual_cost_usd;
    uint64_t created_at_ns;
    uint64_t last_updated_ns;
};

struct EventVenue {
    uint32_t venue_id;
    VenueType venue_type;
    char venue_name[128];
    double latitude;
    double longitude;
    double area_sqft;
    uint32_t max_capacity;
    uint32_t current_occupancy;
    bool accessibility_compliant;
    bool climate_controlled;
    bool emergency_exits_verified;
    bool indigenous_land_acknowledged;
    double shade_coverage_pct;
    double water_station_count;
    double cooling_capacity_kw;
    bool operational;
    uint64_t last_inspection_ns;
};

struct EventParticipant {
    uint64_t participant_id;
    uint64_t citizen_did;
    uint32_t event_id;
    uint64_t registration_ns;
    uint64_t check_in_ns;
    uint64_t check_out_ns;
    bool attended;
    bool accessibility_needs;
    bool indigenous_community;
    bool youth_participant;
    bool senior_participant;
    bool volunteer;
    uint8_t safety_clearance;
};

struct CulturalActivity {
    uint32_t activity_id;
    char activity_name[128];
    EventType activity_type;
    uint32_t tribe_affiliation;
    char language_used[32];
    uint32_t knowledge_keeper_id;
    bool sacred_content;
    bool recording_permitted;
    bool public_access;
    bool youth_appropriate;
    double cultural_significance_score;
    uint64_t first_practiced_ns;
    uint64_t last_practiced_ns;
    uint32_t transmission_count;
    bool preservation_priority;
};

class CommunityEventCoordinator {
private:
    uint64_t coordinator_id_;
    char city_code_[8];
    CommunityEvent events_[MAX_EVENTS];
    size_t event_count_;
    EventVenue venues_[MAX_VENUES];
    size_t venue_count_;
    EventParticipant participants_[MAX_PARTICIPANTS];
    size_t participant_count_;
    CulturalActivity activities_[MAX_CULTURAL_ACTIVITIES];
    size_t activity_count_;
    uint64_t total_events_held_;
    uint64_t total_participants_served_;
    uint64_t indigenous_events_;
    uint64_t tribal_consent_events_;
    double average_attendance_rate_;
    double community_engagement_score_;
    uint64_t heat_safety_activations_;
    uint64_t emergency_closures_;
    uint64_t audit_checksum_;
    uint64_t last_optimization_ns_;
    
    void UpdateAuditChecksum() {
        uint64_t sum = 0;
        sum ^= event_count_ * venue_count_ * participant_count_;
        sum ^= total_events_held_;
        sum ^= total_participants_served_;
        sum ^= heat_safety_activations_;
        for (size_t i = 0; i < event_count_; ++i) {
            sum ^= events_[i].event_id * static_cast<uint64_t>(events_[i].status);
        }
        audit_checksum_ = sum;
    }
    
    bool IsHeatSafe(uint64_t event_start_ns, VenueType venue_type) {
        if (venue_type == VenueType::INDOOR || venue_type == VenueType::VIRTUAL) {
            return true;
        }
        uint64_t forecast_temp = GetTemperatureForecast(event_start_ns);
        return forecast_temp < PHOENIX_MAX_OUTDOOR_TEMP_F;
    }
    
    uint64_t GetTemperatureForecast(uint64_t timestamp_ns) {
        return 95;
    }
    
public:
    CommunityEventCoordinator(uint64_t coordinator_id, const char* city_code, uint64_t init_ns)
        : coordinator_id_(coordinator_id), event_count_(0), venue_count_(0),
          participant_count_(0), activity_count_(0), total_events_held_(0),
          total_participants_served_(0), indigenous_events_(0),
          tribal_consent_events_(0), average_attendance_rate_(0.0),
          community_engagement_score_(0.0), heat_safety_activations_(0),
          emergency_closures_(0), audit_checksum_(0), last_optimization_ns_(init_ns) {
        for (int i = 0; i < 8 && city_code[i] != '\0'; ++i) {
            city_code_[i] = city_code[i];
        }
    }
    
    bool RegisterEvent(const CommunityEvent& event) {
        if (event_count_ >= MAX_EVENTS) return false;
        events_[event_count_] = event;
        if (event.indigenous_led) indigenous_events_++;
        if (event.tribal_consent_obtained) tribal_consent_events_++;
        event_count_++;
        UpdateAuditChecksum();
        return true;
    }
    
    bool RegisterVenue(const EventVenue& venue) {
        if (venue_count_ >= MAX_VENUES) return false;
        venues_[venue_count_] = venue;
        venue_count_++;
        UpdateAuditChecksum();
        return true;
    }
    
    bool RegisterParticipant(const EventParticipant& participant) {
        if (participant_count_ >= MAX_PARTICIPANTS) return false;
        participants_[participant_count_] = participant;
        participant_count_++;
        total_participants_served_++;
        UpdateAuditChecksum();
        return true;
    }
    
    bool RegisterCulturalActivity(const CulturalActivity& activity) {
        if (activity_count_ >= MAX_CULTURAL_ACTIVITIES) return false;
        activities_[activity_count_] = activity;
        activity_count_++;
        UpdateAuditChecksum();
        return true;
    }
    
    bool ApproveEventForExecution(uint64_t event_id, uint64_t now_ns) {
        for (size_t i = 0; i < event_count_; ++i) {
            if (events_[i].event_id == event_id) {
                if (events_[i].indigenous_led && !events_[i].tribal_consent_obtained) {
                    return false;
                }
                VenueType vtype = VenueType::OUTDOOR;
                for (size_t j = 0; j < venue_count_; ++j) {
                    if (venues_[j].venue_id == events_[i].venue_id) {
                        vtype = venues_[j].venue_type;
                        break;
                    }
                }
                if (!IsHeatSafe(events_[i].start_date_ns, vtype)) {
                    heat_safety_activations_++;
                    return false;
                }
                events_[i].status = EventStatus::OPEN_REGISTRATION;
                events_[i].last_updated_ns = now_ns;
                UpdateAuditChecksum();
                return true;
            }
        }
        return false;
    }
    
    void CloseEventForHeatSafety(uint64_t event_id, uint64_t now_ns) {
        for (size_t i = 0; i < event_count_; ++i) {
            if (events_[i].event_id == event_id) {
                if (events_[i].status == EventStatus::ACTIVE) {
                    events_[i].status = EventStatus::EMERGENCY_CLOSED;
                    heat_safety_activations_++;
                    emergency_closures_++;
                    events_[i].last_updated_ns = now_ns;
                }
            }
        }
        UpdateAuditChecksum();
    }
    
    double ComputeAttendanceRate() {
        uint64_t total_attended = 0;
        uint64_t total_registered = 0;
        for (size_t i = 0; i < participant_count_; ++i) {
            if (participants_[i].attended) total_attended++;
            total_registered++;
        }
        if (total_registered == 0) return 0.0;
        average_attendance_rate_ = static_cast<double>(total_attended) / total_registered;
        return average_attendance_rate_;
    }
    
    double ComputeCommunityEngagementScore() {
        double participation_rate = static_cast<double>(total_participants_served_) / 
                                   (participant_count_.max(1) * 10);
        double indigenous_inclusion = static_cast<double>(indigenous_events_) / 
                                     event_count_.max(1);
        double accessibility_score = 0.0;
        size_t accessible_venues = 0;
        for (size_t i = 0; i < venue_count_; ++i) {
            if (venues_[i].accessibility_compliant) accessible_venues++;
        }
        if (venue_count_ > 0) accessibility_score = static_cast<double>(accessible_venues) / venue_count_;
        double heat_safety_score = 1.0 - (emergency_closures_ / event_count_.max(1));
        return (participation_rate * 0.30 + indigenous_inclusion * 0.30 + 
                accessibility_score * 0.20 + heat_safety_score * 0.20);
    }
    
    struct CoordinatorStatus {
        uint64_t coordinator_id;
        char city_code[8];
        size_t total_events;
        size_t active_events;
        size_t completed_events;
        size_t total_venues;
        size_t operational_venues;
        size_t total_participants;
        size_t total_cultural_activities;
        uint64_t total_events_held;
        uint64_t total_participants_served;
        uint64_t indigenous_events;
        uint64_t tribal_consent_events;
        double average_attendance_rate;
        double community_engagement_score;
        uint64_t heat_safety_activations;
        uint64_t emergency_closures;
        uint64_t last_optimization_ns;
        uint64_t last_update_ns;
    };
    
    CoordinatorStatus GetStatus(uint64_t now_ns) {
        CoordinatorStatus status;
        status.coordinator_id = coordinator_id_;
        for (int i = 0; i < 8; ++i) status.city_code[i] = city_code_[i];
        status.total_events = event_count_;
        status.active_events = 0;
        status.completed_events = 0;
        for (size_t i = 0; i < event_count_; ++i) {
            if (events_[i].status == EventStatus::ACTIVE) status.active_events++;
            if (events_[i].status == EventStatus::COMPLETED) status.completed_events++;
        }
        status.total_venues = venue_count_;
        status.operational_venues = 0;
        for (size_t i = 0; i < venue_count_; ++i) {
            if (venues_[i].operational) status.operational_venues++;
        }
        status.total_participants = participant_count_;
        status.total_cultural_activities = activity_count_;
        status.total_events_held = total_events_held_;
        status.total_participants_served = total_participants_served_;
        status.indigenous_events = indigenous_events_;
        status.tribal_consent_events = tribal_consent_events_;
        status.average_attendance_rate = ComputeAttendanceRate();
        status.community_engagement_score = ComputeCommunityEngagementScore();
        status.heat_safety_activations = heat_safety_activations_;
        status.emergency_closures = emergency_closures_;
        status.last_optimization_ns = last_optimization_ns_;
        status.last_update_ns = now_ns;
        return status;
    }
    
    bool VerifyAuditIntegrity() const {
        uint64_t sum = 0;
        sum ^= event_count_ * venue_count_ * participant_count_;
        sum ^= total_events_held_;
        sum ^= total_participants_served_;
        sum ^= heat_safety_activations_;
        for (size_t i = 0; i < event_count_; ++i) {
            sum ^= events_[i].event_id * static_cast<uint64_t>(events_[i].status);
        }
        return sum == audit_checksum_;
    }
    
    void OptimizeEventSchedule(uint64_t now_ns) {
        last_optimization_ns_ = now_ns;
        for (size_t i = 0; i < event_count_; ++i) {
            if (events_[i].status == EventStatus::PLANNED) {
                VenueType vtype = VenueType::OUTDOOR;
                for (size_t j = 0; j < venue_count_; ++j) {
                    if (venues_[j].venue_id == events_[i].venue_id) {
                        vtype = venues_[j].venue_type;
                        break;
                    }
                }
                if (!IsHeatSafe(events_[i].start_date_ns, vtype)) {
                    events_[i].status = EventStatus::POSTPONED;
                }
            }
        }
        UpdateAuditChecksum();
    }
};

#endif
