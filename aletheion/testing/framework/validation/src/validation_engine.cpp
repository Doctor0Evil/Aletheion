#ifndef VALIDATION_ENGINE_HPP
#define VALIDATION_ENGINE_HPP

#include <cstdint>
#include <cstddef>
#include <array>
#include <cmath>

constexpr uint32_t VALIDATION_ENGINE_VERSION = 20260310;
constexpr size_t MAX_TEST_SUITES = 1024;
constexpr size_t MAX_TEST_CASES = 65536;
constexpr size_t MAX_VALIDATION_RULES = 8192;
constexpr size_t MAX_ASSERTION_FAILURES = 32768;
constexpr double MIN_TEST_COVERAGE_PCT = 85.0;
constexpr double MIN_VALIDATION_SCORE = 0.90;

enum class TestType : uint8_t {
    UNIT = 0, INTEGRATION = 1, SYSTEM = 2, ACCEPTANCE = 3,
    PERFORMANCE = 4, SECURITY = 5, COMPLIANCE = 6, ACCESSIBILITY = 7
};

enum class TestStatus : uint8_t {
    PENDING = 0, RUNNING = 1, PASSED = 2, FAILED = 3,
    SKIPPED = 4, BLOCKED = 5, INCONCLUSIVE = 6
};

enum class ValidationRuleType : uint8_t {
    SCHEMA = 0, CONSTRAINT = 1, BUSINESS_LOGIC = 2, SECURITY = 3,
    PERFORMANCE = 5, COMPLIANCE = 6, ACCESSIBILITY = 7
};

struct TestCase {
    uint64_t test_id;
    TestType test_type;
    char test_name[128];
    char description[512];
    uint64_t suite_id;
    uint64_t subsystem_id;
    TestStatus status;
    uint64_t execution_time_ms;
    uint64_t last_run_ns;
    uint64_t pass_count;
    uint64_t fail_count;
    double coverage_pct;
    bool automated;
    bool critical;
    bool biotic_treaty_related;
    bool indigenous_rights_related;
};

struct TestSuite {
    uint64_t suite_id;
    char suite_name[128];
    uint64_t subsystem_id;
    uint64_t test_case_ids[1024];
    size_t test_case_count;
    uint64_t total_executions;
    uint64_t total_passes;
    uint64_t total_failures;
    double pass_rate_pct;
    double average_execution_time_ms;
    uint64_t last_run_ns;
    bool operational;
};

struct ValidationRule {
    uint64_t rule_id;
    ValidationRuleType rule_type;
    char rule_name[128];
    char rule_expression[1024];
    uint64_t subsystem_id;
    bool mandatory;
    bool automated;
    uint64_t validation_count;
    uint64_t pass_count;
    uint64_t fail_count;
    double compliance_rate_pct;
    uint64_t last_validated_ns;
    bool active;
};

struct AssertionFailure {
    uint64_t failure_id;
    uint64_t test_id;
    uint64_t rule_id;
    char failure_message[512];
    char expected_value[256];
    char actual_value[256];
    uint64_t timestamp_ns;
    uint8_t severity;
    bool resolved;
    uint64_t resolved_ns;
    char resolution_notes[256];
};

class TestValidationEngine {
private:
    uint64_t engine_id_;
    char city_code_[8];
    TestCase test_cases_[MAX_TEST_CASES];
    size_t test_case_count_;
    TestSuite test_suites_[MAX_TEST_SUITES];
    size_t suite_count_;
    ValidationRule validation_rules_[MAX_VALIDATION_RULES];
    size_t rule_count_;
    AssertionFailure assertion_failures_[MAX_ASSERTION_FAILURES];
    size_t failure_count_;
    uint64_t total_test_executions_;
    uint64_t total_test_passes_;
    uint64_t total_test_failures_;
    double overall_pass_rate_pct_;
    double average_coverage_pct_;
    double validation_score_;
    uint64_t critical_failures_;
    uint64_t compliance_violations_;
    uint64_t audit_checksum_;
    uint64_t last_full_test_run_ns_;
    
    void UpdateAuditChecksum() {
        uint64_t sum = 0;
        sum ^= test_case_count_ * suite_count_ * rule_count_;
        sum ^= total_test_executions_;
        sum ^= total_test_failures_;
        sum ^= critical_failures_;
        for (size_t i = 0; i < test_case_count_; ++i) {
            sum ^= test_cases_[i].test_id * static_cast<uint64_t>(test_cases_[i].status);
        }
        audit_checksum_ = sum;
    }
    
public:
    TestValidationEngine(uint64_t engine_id, const char* city_code, uint64_t init_ns)
        : engine_id_(engine_id), test_case_count_(0), suite_count_(0),
          rule_count_(0), failure_count_(0), total_test_executions_(0),
          total_test_passes_(0), total_test_failures_(0), overall_pass_rate_pct_(0.0),
          average_coverage_pct_(0.0), validation_score_(1.0), critical_failures_(0),
          compliance_violations_(0), audit_checksum_(0), last_full_test_run_ns_(init_ns) {
        for (int i = 0; i < 8 && city_code[i] != '\0'; ++i) {
            city_code_[i] = city_code[i];
        }
    }
    
    bool RegisterTestCase(const TestCase& test_case) {
        if (test_case_count_ >= MAX_TEST_CASES) return false;
        test_cases_[test_case_count_] = test_case;
        test_case_count_++;
        UpdateAuditChecksum();
        return true;
    }
    
    bool RegisterTestSuite(const TestSuite& suite) {
        if (suite_count_ >= MAX_TEST_SUITES) return false;
        test_suites_[suite_count_] = suite;
        suite_count_++;
        UpdateAuditChecksum();
        return true;
    }
    
    bool RegisterValidationRule(const ValidationRule& rule) {
        if (rule_count_ >= MAX_VALIDATION_RULES) return false;
        validation_rules_[rule_count_] = rule;
        rule_count_++;
        UpdateAuditChecksum();
        return true;
    }
    
    void RecordTestExecution(uint64_t test_id, bool passed, uint64_t execution_time_ms, uint64_t now_ns) {
        for (size_t i = 0; i < test_case_count_; ++i) {
            if (test_cases_[i].test_id == test_id) {
                test_cases_[i].last_run_ns = now_ns;
                test_cases_[i].execution_time_ms = execution_time_ms;
                if (passed) {
                    test_cases_[i].status = TestStatus::PASSED;
                    test_cases_[i].pass_count++;
                    total_test_passes_++;
                } else {
                    test_cases_[i].status = TestStatus::FAILED;
                    test_cases_[i].fail_count++;
                    total_test_failures_++;
                    if (test_cases_[i].critical) {
                        critical_failures_++;
                    }
                }
                total_test_executions_++;
                overall_pass_rate_pct_ = static_cast<double>(total_test_passes_) / 
                                        total_test_executions_.max(1) * 100.0;
                UpdateAuditChecksum();
                return;
            }
        }
    }
    
    void RecordAssertionFailure(const AssertionFailure& failure) {
        if (failure_count_ >= MAX_ASSERTION_FAILURES) return;
        assertion_failures_[failure_count_] = failure;
        failure_count_++;
        if (failure.severity >= 4) {
            compliance_violations_++;
        }
        UpdateAuditChecksum();
    }
    
    void ResolveFailure(uint64_t failure_id, uint64_t now_ns, const char* notes) {
        for (size_t i = 0; i < failure_count_; ++i) {
            if (assertion_failures_[i].failure_id == failure_id) {
                assertion_failures_[i].resolved = true;
                assertion_failures_[i].resolved_ns = now_ns;
                for (size_t j = 0; j < 256 && notes[j] != '\0'; ++j) {
                    assertion_failures_[i].resolution_notes[j] = notes[j];
                }
                UpdateAuditChecksum();
                return;
            }
        }
    }
    
    double ComputeTestCoverage() {
        if (test_case_count_ == 0) return 0.0;
        double total_coverage = 0.0;
        size_t valid_tests = 0;
        for (size_t i = 0; i < test_case_count_; ++i) {
            if (test_cases_[i].last_run_ns > 0) {
                total_coverage += test_cases_[i].coverage_pct;
                valid_tests++;
            }
        }
        if (valid_tests == 0) return 0.0;
        average_coverage_pct_ = total_coverage / valid_tests;
        return average_coverage_pct_;
    }
    
    double ComputeValidationScore() {
        if (rule_count_ == 0) return 1.0;
        double total_compliance = 0.0;
        size_t mandatory_rules = 0;
        for (size_t i = 0; i < rule_count_; ++i) {
            if (validation_rules_[i].active) {
                validation_rules_[i].compliance_rate_pct = 
                    static_cast<double>(validation_rules_[i].pass_count) /
                    validation_rules_[i].validation_count.max(1) * 100.0;
                total_compliance += validation_rules_[i].compliance_rate_pct;
                if (validation_rules_[i].mandatory) {
                    mandatory_rules++;
                }
            }
        }
        validation_score_ = total_compliance / rule_count_.max(1);
        if (compliance_violations_ > 0) {
            validation_score_ -= compliance_violations_ * 0.02;
        }
        return validation_score_.max(0.0);
    }
    
    struct EngineStatus {
        uint64_t engine_id;
        char city_code[8];
        size_t total_test_cases;
        size_t passed_test_cases;
        size_t failed_test_cases;
        size_t total_suites;
        size_t operational_suites;
        size_t total_rules;
        size_t active_rules;
        size_t total_failures;
        size_t resolved_failures;
        uint64_t total_executions;
        double overall_pass_rate_pct;
        double average_coverage_pct;
        double validation_score;
        uint64_t critical_failures;
        uint64_t compliance_violations;
        uint64_t last_full_test_run_ns;
        uint64_t last_update_ns;
    };
    
    EngineStatus GetStatus(uint64_t now_ns) {
        EngineStatus status;
        status.engine_id = engine_id_;
        for (int i = 0; i < 8; ++i) status.city_code[i] = city_code_[i];
        status.total_test_cases = test_case_count_;
        status.passed_test_cases = 0;
        status.failed_test_cases = 0;
        for (size_t i = 0; i < test_case_count_; ++i) {
            if (test_cases_[i].status == TestStatus::PASSED) status.passed_test_cases++;
            if (test_cases_[i].status == TestStatus::FAILED) status.failed_test_cases++;
        }
        status.total_suites = suite_count_;
        status.operational_suites = 0;
        for (size_t i = 0; i < suite_count_; ++i) {
            if (test_suites_[i].operational) status.operational_suites++;
        }
        status.total_rules = rule_count_;
        status.active_rules = 0;
        for (size_t i = 0; i < rule_count_; ++i) {
            if (validation_rules_[i].active) status.active_rules++;
        }
        status.total_failures = failure_count_;
        status.resolved_failures = 0;
        for (size_t i = 0; i < failure_count_; ++i) {
            if (assertion_failures_[i].resolved) status.resolved_failures++;
        }
        status.total_executions = total_test_executions_;
        status.overall_pass_rate_pct = overall_pass_rate_pct_;
        status.average_coverage_pct = ComputeTestCoverage();
        status.validation_score = ComputeValidationScore();
        status.critical_failures = critical_failures_;
        status.compliance_violations = compliance_violations_;
        status.last_full_test_run_ns = last_full_test_run_ns_;
        status.last_update_ns = now_ns;
        return status;
    }
    
    bool VerifyAuditIntegrity() const {
        uint64_t sum = 0;
        sum ^= test_case_count_ * suite_count_ * rule_count_;
        sum ^= total_test_executions_;
        sum ^= total_test_failures_;
        sum ^= critical_failures_;
        for (size_t i = 0; i < test_case_count_; ++i) {
            sum ^= test_cases_[i].test_id * static_cast<uint64_t>(test_cases_[i].status);
        }
        return sum == audit_checksum_;
    }
    
    void RunFullTestSuite(uint64_t now_ns) {
        last_full_test_run_ns_ = now_ns;
        ComputeTestCoverage();
        ComputeValidationScore();
        UpdateAuditChecksum();
    }
    
    bool MeetsDeploymentCriteria() {
        double coverage = ComputeTestCoverage();
        double validation = ComputeValidationScore();
        double pass_rate = overall_pass_rate_pct_;
        return coverage >= MIN_TEST_COVERAGE_PCT && 
               validation >= MIN_VALIDATION_SCORE && 
               pass_rate >= 95.0 &&
               critical_failures_ == 0;
    }
};

#endif
