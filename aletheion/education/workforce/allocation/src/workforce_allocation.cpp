#ifndef WORKFORCE_ALLOCATION_HPP
#define WORKFORCE_ALLOCATION_HPP

#include <cstdint>
#include <cstddef>
#include <array>
#include <cmath>

constexpr uint32_t WORKFORCE_ALLOCATION_VERSION = 20260310;
constexpr size_t MAX_WORK_PROJECTS = 8192;
constexpr size_t MAX_WORKER_ASSIGNMENTS = 262144;
constexpr size_t MAX_SKILL_REQUIREMENTS = 65536;
constexpr size_t MAX_COMMUNITY_PARTNERS = 2048;
constexpr double PHOENIX_UNEMPLOYMENT_TARGET_PCT = 3.5;
constexpr double INDIGENOUS_EMPLOYMENT_TARGET_PCT = 0.70;

enum class ProjectType : uint8_t {
    INFRASTRUCTURE = 0, ENVIRONMENTAL = 1, SOCIAL_SERVICES = 2, EDUCATION = 3,
    HEALTHCARE = 4, PUBLIC_SAFETY = 5, ARTS_CULTURE = 6, TECHNOLOGY = 7,
    AGRICULTURE = 8, ENERGY = 9, TRANSPORTATION = 10, HOUSING = 11
};

enum class WorkerStatus : uint8_t {
    AVAILABLE = 0, ASSIGNED = 1, IN_TRAINING = 2, ON_LEAVE = 3,
    UNEMPLOYED = 4, UNDEREMPLOYED = 5, RETIRED = 6, DISABLED = 7
};

enum class AssignmentStatus : uint8_t {
    PENDING = 0, ACTIVE = 1, COMPLETED = 2, CANCELLED = 3,
    EXTENDED = 4, TERMINATED = 5, TRANSFERRED = 6
};

struct WorkProject {
    uint64_t project_id;
    ProjectType project_type;
    char project_name[128];
    char description[512];
    uint32_t required_workers;
    uint32_t assigned_workers;
    uint64_t start_date_ns;
    uint64_t end_date_ns;
    double budget_usd;
    double spent_usd;
    uint32_t location_zone_id;
    bool indigenous_priority;
    bool disability_accommodation;
    bool living_wage_compliant;
    bool accessibility_compliant;
    uint32_t community_partner_id;
    double completion_pct;
    uint8_t status;
    uint64_t created_at_ns;
    uint64_t updated_at_ns;
};

struct Worker {
    uint64_t worker_id;
    char citizen_did[32];
    uint8_t age;
    WorkerStatus status;
    double skill_level;
    uint64_t skills[64];
    uint8_t skill_count;
    uint32_t certifications[32];
    uint8_t certification_count;
    double target_wage_usd_hour;
    double current_wage_usd_hour;
    uint16_t work_experience_years;
    bool indigenous_community;
    bool disability_status;
    bool low_income_status;
    bool veteran_status;
    bool formerly_incarcerated;
    uint32_t current_assignment_id;
    uint64_t last_assignment_ns;
    uint32_t total_assignments;
    double performance_score;
    uint64_t registered_at_ns;
    uint64_t updated_at_ns;
};

struct WorkerAssignment {
    uint64_t assignment_id;
    uint64_t worker_id;
    uint64_t project_id;
    AssignmentStatus status;
    double wage_usd_hour;
    uint64_t start_date_ns;
    uint64_t end_date_ns;
    uint64_t actual_end_ns;
    double hours_worked;
    double performance_rating;
    bool completed_training;
    bool living_wage_achieved;
    bool indigenous_preference_applied;
    bool disability_accommodation_provided;
    uint64_t created_at_ns;
    uint64_t updated_at_ns;
};

struct CommunityPartner {
    uint32_t partner_id;
    char partner_name[128];
    uint8_t partner_type;
    char service_area[256];
    uint32_t workers_served;
    uint32_t placements_made;
    double placement_success_rate;
    bool indigenous_owned;
    bool disability_owned;
    bool nonprofit_status;
    bool accessibility_compliant;
    uint64_t partnership_start_ns;
    uint64_t last_collaboration_ns;
    double funding_received_usd;
    uint8_t status;
};

class WorkforceAllocationManager {
private:
    uint64_t manager_id_;
    char city_code_[8];
    WorkProject projects_[MAX_WORK_PROJECTS];
    size_t project_count_;
    Worker workers_[262144];
    size_t worker_count_;
    WorkerAssignment assignments_[MAX_WORKER_ASSIGNMENTS];
    size_t assignment_count_;
    CommunityPartner partners_[MAX_COMMUNITY_PARTNERS];
    size_t partner_count_;
    uint64_t total_placements_;
    uint64_t successful_placements_;
    double average_placement_wage_;
    double unemployment_rate_;
    double indigenous_employment_rate_;
    double disability_employment_rate_;
    uint64_t living_wage_placements_;
    uint64_t last_allocation_run_ns_;
    uint64_t audit_checksum_;
    
    void UpdateAuditChecksum() {
        uint64_t sum = 0;
        sum ^= project_count_ * worker_count_ * assignment_count_;
        sum ^= total_placements_;
        sum ^= living_wage_placements_;
        for (size_t i = 0; i < worker_count_; ++i) {
            sum ^= workers_[i].worker_id * static_cast<uint64_t>(workers_[i].status);
        }
        audit_checksum_ = sum;
    }
    
    double ComputeSkillMatch(const Worker& worker, const WorkProject& project) {
        return 0.8;
    }
    
public:
    WorkforceAllocationManager(uint64_t manager_id, const char* city_code, uint64_t init_ns)
        : manager_id_(manager_id), project_count_(0), worker_count_(0),
          assignment_count_(0), partner_count_(0), total_placements_(0),
          successful_placements_(0), average_placement_wage_(0.0),
          unemployment_rate_(0.0), indigenous_employment_rate_(0.0),
          disability_employment_rate_(0.0), living_wage_placements_(0),
          last_allocation_run_ns_(init_ns), audit_checksum_(0) {
        for (int i = 0; i < 8 && city_code[i] != '\0'; ++i) {
            city_code_[i] = city_code[i];
        }
    }
    
    bool RegisterWorkProject(const WorkProject& project) {
        if (project_count_ >= MAX_WORK_PROJECTS) return false;
        if (!project.accessibility_compliant) return false;
        if (!project.living_wage_compliant) return false;
        projects_[project_count_] = project;
        project_count_++;
        UpdateAuditChecksum();
        return true;
    }
    
    bool RegisterWorker(const Worker& worker) {
        if (worker_count_ >= 262144) return false;
        workers_[worker_count_] = worker;
        worker_count_++;
        UpdateAuditChecksum();
        return true;
    }
    
    bool RegisterCommunityPartner(const CommunityPartner& partner) {
        if (partner_count_ >= MAX_COMMUNITY_PARTNERS) return false;
        if (!partner.accessibility_compliant) return false;
        partners_[partner_count_] = partner;
        partner_count_++;
        UpdateAuditChecksum();
        return true;
    }
    
    bool AssignWorkerToProject(uint64_t worker_id, uint64_t project_id, 
                               double wage_usd_hour, uint64_t start_ns, uint64_t now_ns) {
        if (assignment_count_ >= MAX_WORKER_ASSIGNMENTS) return false;
        Worker* worker = nullptr;
        WorkProject* project = nullptr;
        for (size_t i = 0; i < worker_count_; ++i) {
            if (workers_[i].worker_id == worker_id) {
                worker = &workers_[i];
                break;
            }
        }
        for (size_t i = 0; i < project_count_; ++i) {
            if (projects_[i].project_id == project_id) {
                project = &projects_[i];
                break;
            }
        }
        if (!worker || !project) return false;
        if (worker->status != WorkerStatus::AVAILABLE) return false;
        if (project->assigned_workers >= project->required_workers) return false;
        WorkerAssignment assignment;
        assignment.assignment_id = assignment_count_;
        assignment.worker_id = worker_id;
        assignment.project_id = project_id;
        assignment.status = AssignmentStatus::ACTIVE;
        assignment.wage_usd_hour = wage_usd_hour;
        assignment.start_date_ns = start_ns;
        assignment.end_date_ns = project->end_date_ns;
        assignment.actual_end_ns = 0;
        assignment.hours_worked = 0;
        assignment.performance_rating = 0;
        assignment.completed_training = false;
        assignment.living_wage_achieved = wage_usd_hour >= 22.50;
        assignment.indigenous_preference_applied = worker->indigenous_community && project->indigenous_priority;
        assignment.disability_accommodation_provided = worker->disability_status && project->disability_accommodation;
        assignment.created_at_ns = now_ns;
        assignment.updated_at_ns = now_ns;
        assignments_[assignment_count_] = assignment;
        assignment_count_++;
        worker->status = WorkerStatus::ASSIGNED;
        worker->current_assignment_id = project_id;
        worker->current_wage_usd_hour = wage_usd_hour;
        project->assigned_workers++;
        total_placements_++;
        if (assignment.living_wage_achieved) {
            living_wage_placements_++;
        }
        UpdateAuditChecksum();
        return true;
    }
    
    void CompleteAssignment(uint64_t assignment_id, uint64_t actual_end_ns, 
                           double performance_rating, uint64_t now_ns) {
        for (size_t i = 0; i < assignment_count_; ++i) {
            if (assignments_[i].assignment_id == assignment_id) {
                assignments_[i].status = AssignmentStatus::COMPLETED;
                assignments_[i].actual_end_ns = actual_end_ns;
                assignments_[i].performance_rating = performance_rating;
                assignments_[i].updated_at_ns = now_ns;
                successful_placements_++;
                for (size_t j = 0; j < worker_count_; ++j) {
                    if (workers_[j].worker_id == assignments_[i].worker_id) {
                        workers_[j].status = WorkerStatus::AVAILABLE;
                        workers_[j].total_assignments++;
                        workers_[j].last_assignment_ns = actual_end_ns;
                        workers_[j].performance_score = performance_rating;
                        break;
                    }
                }
                for (size_t j = 0; j < project_count_; ++j) {
                    if (projects_[j].project_id == assignments_[i].project_id) {
                        projects_[j].completion_pct = 100.0;
                        projects_[j].status = 2;
                        break;
                    }
                }
                UpdateAuditChecksum();
                return;
            }
        }
    }
    
    double ComputeUnemploymentRate() {
        uint64_t unemployed = 0;
        for (size_t i = 0; i < worker_count_; ++i) {
            if (workers_[i].status == WorkerStatus::UNEMPLOYED) {
                unemployed++;
            }
        }
        if (worker_count_ == 0) return 0.0;
        unemployment_rate_ = static_cast<double>(unemployed) / worker_count_;
        return unemployment_rate_;
    }
    
    double ComputeIndigenousEmploymentRate() {
        uint64_t indigenous_workers = 0;
        uint64_t indigenous_employed = 0;
        for (size_t i = 0; i < worker_count_; ++i) {
            if (workers_[i].indigenous_community) {
                indigenous_workers++;
                if (workers_[i].status == WorkerStatus::ASSIGNED) {
                    indigenous_employed++;
                }
            }
        }
        if (indigenous_workers == 0) return 0.0;
        indigenous_employment_rate_ = static_cast<double>(indigenous_employed) / indigenous_workers;
        return indigenous_employment_rate_;
    }
    
    double ComputeDisabilityEmploymentRate() {
        uint64_t disabled_workers = 0;
        uint64_t disabled_employed = 0;
        for (size_t i = 0; i < worker_count_; ++i) {
            if (workers_[i].disability_status) {
                disabled_workers++;
                if (workers_[i].status == WorkerStatus::ASSIGNED) {
                    disabled_employed++;
                }
            }
        }
        if (disabled_workers == 0) return 0.0;
        disability_employment_rate_ = static_cast<double>(disabled_employed) / disabled_workers;
        return disability_employment_rate_;
    }
    
    struct ManagerStatus {
        uint64_t manager_id;
        char city_code[8];
        size_t total_projects;
        size_t active_projects;
        size_t total_workers;
        size_t employed_workers;
        size_t unemployed_workers;
        size_t indigenous_workers;
        size_t indigenous_employed;
        size_t disabled_workers;
        size_t disabled_employed;
        size_t total_assignments;
        size_t active_assignments;
        size_t community_partners;
        uint64_t total_placements;
        uint64_t successful_placements;
        uint64_t living_wage_placements;
        double unemployment_rate;
        double indigenous_employment_rate;
        double disability_employment_rate;
        double average_placement_wage;
        uint64_t last_allocation_run_ns;
        uint64_t last_update_ns;
    };
    
    ManagerStatus GetStatus(uint64_t now_ns) {
        ManagerStatus status;
        status.manager_id = manager_id_;
        for (int i = 0; i < 8; ++i) status.city_code[i] = city_code_[i];
        status.total_projects = project_count_;
        status.active_projects = 0;
        for (size_t i = 0; i < project_count_; ++i) {
            if (projects_[i].status == 1) status.active_projects++;
        }
        status.total_workers = worker_count_;
        status.employed_workers = 0;
        status.unemployed_workers = 0;
        status.indigenous_workers = 0;
        status.indigenous_employed = 0;
        status.disabled_workers = 0;
        status.disabled_employed = 0;
        for (size_t i = 0; i < worker_count_; ++i) {
            if (workers_[i].status == WorkerStatus::ASSIGNED) {
                status.employed_workers++;
            } else if (workers_[i].status == WorkerStatus::UNEMPLOYED) {
                status.unemployed_workers++;
            }
            if (workers_[i].indigenous_community) {
                status.indigenous_workers++;
                if (workers_[i].status == WorkerStatus::ASSIGNED) {
                    status.indigenous_employed++;
                }
            }
            if (workers_[i].disability_status) {
                status.disabled_workers++;
                if (workers_[i].status == WorkerStatus::ASSIGNED) {
                    status.disabled_employed++;
                }
            }
        }
        status.total_assignments = assignment_count_;
        status.active_assignments = 0;
        for (size_t i = 0; i < assignment_count_; ++i) {
            if (assignments_[i].status == AssignmentStatus::ACTIVE) {
                status.active_assignments++;
            }
        }
        status.community_partners = partner_count_;
        status.total_placements = total_placements_;
        status.successful_placements = successful_placements_;
        status.living_wage_placements = living_wage_placements_;
        status.unemployment_rate = ComputeUnemploymentRate();
        status.indigenous_employment_rate = ComputeIndigenousEmploymentRate();
        status.disability_employment_rate = ComputeDisabilityEmploymentRate();
        status.average_placement_wage = average_placement_wage_;
        status.last_allocation_run_ns = last_allocation_run_ns_;
        status.last_update_ns = now_ns;
        return status;
    }
    
    double ComputeWorkforceEquityIndex() {
        ManagerStatus status = GetStatus(last_allocation_run_ns_);
        double overall_employment = status.employed_workers / status.total_workers.max(1);
        double indigenous_equity = status.indigenous_employment_rate;
        double disability_equity = status.disability_employment_rate;
        double living_wage_rate = static_cast<double>(living_wage_placements_) / 
                                 total_placements_.max(1);
        double unemployment_penalty = status.unemployment_rate * 0.3;
        return (overall_employment * 0.30 + indigenous_equity * 0.25 + 
                disability_equity * 0.25 + living_wage_rate * 0.20 - unemployment_penalty).max(0.0);
    }
    
    bool VerifyAuditIntegrity() const {
        uint64_t sum = 0;
        sum ^= project_count_ * worker_count_ * assignment_count_;
        sum ^= total_placements_;
        sum ^= living_wage_placements_;
        for (size_t i = 0; i < worker_count_; ++i) {
            sum ^= workers_[i].worker_id * static_cast<uint64_t>(workers_[i].status);
        }
        return sum == audit_checksum_;
    }
    
    void RunAllocationOptimization(uint64_t now_ns) {
        last_allocation_run_ns_ = now_ns;
        ComputeUnemploymentRate();
        ComputeIndigenousEmploymentRate();
        ComputeDisabilityEmploymentRate();
        UpdateAuditChecksum();
    }
};

#endif
