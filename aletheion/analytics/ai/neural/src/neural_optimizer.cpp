#ifndef NEURAL_OPTIMIZER_HPP
#define NEURAL_OPTIMIZER_HPP

#include <cstdint>
#include <cstddef>
#include <array>
#include <cmath>

constexpr uint32_t NEURAL_OPTIMIZER_VERSION = 20260310;
constexpr size_t MAX_NEURAL_NETWORKS = 256;
constexpr size_t MAX_LAYERS_PER_NETWORK = 128;
constexpr size_t MAX_NEURONS_PER_LAYER = 4096;
constexpr size_t MAX_TRAINING_BATCHES = 65536;
constexpr double LEARNING_RATE_DEFAULT = 0.001;
constexpr double REGULARIZATION_LAMBDA = 0.01;
constexpr double CONVERGENCE_THRESHOLD = 1e-6;

enum class ActivationFunction : uint8_t {
    RELU = 0, SIGMOID = 1, TANH = 2, SOFTMAX = 3,
    LINEAR = 4, LEAKY_RELU = 5, GELU = 6, SWISH = 7
};

enum class OptimizationAlgorithm : uint8_t {
    SGD = 0, ADAM = 1, RMSPROP = 2, ADAGRAD = 3,
    ADAMW = 4, NADAM = 5, LAMB = 6, LARS = 7
};

enum class NetworkPurpose : uint8_t {
    CLASSIFICATION = 0, REGRESSION = 1, GENERATIVE = 2,
    REINFORCEMENT = 3, TRANSFORMER = 4, CNN = 5, RNN = 6, GNN = 7
};

struct NeuralLayer {
    uint32_t layer_id;
    uint32_t neuron_count;
    ActivationFunction activation;
    double dropout_rate;
    bool batch_normalization;
    uint32_t input_features;
    uint32_t output_features;
    uint64_t weight_count;
    uint64_t bias_count;
    double l2_norm;
    double gradient_norm;
    bool trainable;
};

struct NeuralNetwork {
    uint64_t network_id;
    char network_name[64];
    NetworkPurpose purpose;
    NeuralLayer layers[MAX_LAYERS_PER_NETWORK];
    size_t layer_count;
    OptimizationAlgorithm optimizer;
    double learning_rate;
    double momentum;
    uint64_t total_parameters;
    uint64_t training_samples;
    uint64_t validation_samples;
    double training_loss;
    double validation_loss;
    double accuracy;
    double precision;
    double recall;
    double f1_score;
    uint64_t training_epochs;
    uint64_t last_trained_ns;
    bool converged;
    bool overfitting_detected;
    bool bias_audited;
    double fairness_score;
    bool operational;
};

struct TrainingBatch {
    uint64_t batch_id;
    uint64_t network_id;
    uint32_t batch_size;
    uint32_t sequence_length;
    double loss;
    double gradient_norm;
    uint64_t timestamp_ns;
    bool completed;
    bool validated;
};

class NeuralNetworkOptimizer {
private:
    uint64_t optimizer_id_;
    char city_code_[8];
    NeuralNetwork networks_[MAX_NEURAL_NETWORKS];
    size_t network_count_;
    TrainingBatch batches_[MAX_TRAINING_BATCHES];
    size_t batch_count_;
    uint64_t total_training_epochs_;
    uint64_t total_inference_requests_;
    double average_inference_latency_ms_;
    double average_training_loss_;
    double gpu_utilization_pct_;
    double tpu_utilization_pct_;
    uint64_t energy_consumption_kwh_;
    uint64_t carbon_footprint_kg_co2e_;
    uint64_t audit_checksum_;
    uint64_t last_optimization_ns_;
    
    void UpdateAuditChecksum() {
        uint64_t sum = 0;
        sum ^= network_count_ * batch_count_;
        sum ^= total_training_epochs_;
        sum ^= total_inference_requests_;
        for (size_t i = 0; i < network_count_; ++i) {
            sum ^= networks_[i].network_id * static_cast<uint64_t>(networks_[i].operational);
        }
        audit_checksum_ = sum;
    }
    
    double ComputeActivation(double x, ActivationFunction func) {
        switch (func) {
            case ActivationFunction::RELU: return x > 0 ? x : 0;
            case ActivationFunction::SIGMOID: return 1.0 / (1.0 + std::exp(-x));
            case ActivationFunction::TANH: return std::tanh(x);
            case ActivationFunction::LEAKY_RELU: return x > 0 ? x : 0.01 * x;
            default: return x;
        }
    }
    
    double ComputeLoss(double predicted, double actual) {
        return 0.5 * (predicted - actual) * (predicted - actual);
    }
    
public:
    NeuralNetworkOptimizer(uint64_t optimizer_id, const char* city_code, uint64_t init_ns)
        : optimizer_id_(optimizer_id), network_count_(0), batch_count_(0),
          total_training_epochs_(0), total_inference_requests_(0),
          average_inference_latency_ms_(0.0), average_training_loss_(0.0),
          gpu_utilization_pct_(0.0), tpu_utilization_pct_(0.0),
          energy_consumption_kwh_(0), carbon_footprint_kg_co2e_(0),
          audit_checksum_(0), last_optimization_ns_(init_ns) {
        for (int i = 0; i < 8 && city_code[i] != '\0'; ++i) {
            city_code_[i] = city_code[i];
        }
    }
    
    bool RegisterNeuralNetwork(const NeuralNetwork& network) {
        if (network_count_ >= MAX_NEURAL_NETWORKS) return false;
        if (!network.bias_audited) return false;
        if (network.fairness_score < 0.8) return false;
        networks_[network_count_] = network;
        network_count_++;
        UpdateAuditChecksum();
        return true;
    }
    
    bool RecordTrainingBatch(const TrainingBatch& batch) {
        if (batch_count_ >= MAX_TRAINING_BATCHES) return false;
        batches_[batch_count_] = batch;
        batch_count_++;
        for (size_t i = 0; i < network_count_; ++i) {
            if (networks_[i].network_id == batch.network_id) {
                networks_[i].training_loss = batch.loss;
                total_training_epochs_++;
            }
        }
        UpdateAuditChecksum();
        return true;
    }
    
    void OptimizeNetworkHyperparameters(uint64_t network_id, uint64_t now_ns) {
        for (size_t i = 0; i < network_count_; ++i) {
            if (networks_[i].network_id == network_id) {
                if (networks_[i].overfitting_detected) {
                    networks_[i].learning_rate *= 0.5;
                    networks_[i].momentum *= 0.9;
                }
                if (networks_[i].converged) {
                    networks_[i].learning_rate *= 0.1;
                }
                networks_[i].last_trained_ns = now_ns;
            }
        }
        UpdateAuditChecksum();
    }
    
    double ComputeNetworkEfficiency(uint64_t network_id) {
        for (size_t i = 0; i < network_count_; ++i) {
            if (networks_[i].network_id == network_id) {
                double accuracy_score = networks_[i].accuracy;
                double efficiency_score = networks_[i].f1_score;
                double fairness_score = networks_[i].fairness_score;
                double energy_penalty = energy_consumption_kwh_ > 1000 ? 0.1 : 0.0;
                return (accuracy_score * 0.4 + efficiency_score * 0.3 + fairness_score * 0.3 - energy_penalty);
            }
        }
        return 0.0;
    }
    
    void DetectOverfitting(uint64_t network_id) {
        for (size_t i = 0; i < network_count_; ++i) {
            if (networks_[i].network_id == network_id) {
                double loss_gap = networks_[i].training_loss - networks_[i].validation_loss;
                if (loss_gap > 0.1) {
                    networks_[i].overfitting_detected = true;
                } else {
                    networks_[i].overfitting_detected = false;
                }
            }
        }
    }
    
    void CheckConvergence(uint64_t network_id) {
        for (size_t i = 0; i < network_count_; ++i) {
            if (networks_[i].network_id == network_id) {
                if (networks_[i].training_loss < CONVERGENCE_THRESHOLD) {
                    networks_[i].converged = true;
                }
            }
        }
    }
    
    struct OptimizerStatus {
        uint64_t optimizer_id;
        char city_code[8];
        size_t total_networks;
        size_t operational_networks;
        size_t total_batches;
        uint64_t total_training_epochs;
        uint64_t total_inference_requests;
        double average_inference_latency_ms;
        double average_training_loss;
        double gpu_utilization_pct;
        double tpu_utilization_pct;
        uint64_t energy_consumption_kwh;
        uint64_t carbon_footprint_kg_co2e;
        uint64_t last_optimization_ns;
        uint64_t last_update_ns;
    };
    
    OptimizerStatus GetStatus(uint64_t now_ns) {
        OptimizerStatus status;
        status.optimizer_id = optimizer_id_;
        for (int i = 0; i < 8; ++i) status.city_code[i] = city_code_[i];
        status.total_networks = network_count_;
        status.operational_networks = 0;
        for (size_t i = 0; i < network_count_; ++i) {
            if (networks_[i].operational) status.operational_networks++;
        }
        status.total_batches = batch_count_;
        status.total_training_epochs = total_training_epochs_;
        status.total_inference_requests = total_inference_requests_;
        status.average_inference_latency_ms = average_inference_latency_ms_;
        status.average_training_loss = average_training_loss_;
        status.gpu_utilization_pct = gpu_utilization_pct_;
        status.tpu_utilization_pct = tpu_utilization_pct_;
        status.energy_consumption_kwh = energy_consumption_kwh_;
        status.carbon_footprint_kg_co2e = carbon_footprint_kg_co2e_;
        status.last_optimization_ns = last_optimization_ns_;
        status.last_update_ns = now_ns;
        return status;
    }
    
    double ComputeAI SustainabilityScore() {
        double efficiency_score = 0.0;
        size_t valid_networks = 0;
        for (size_t i = 0; i < network_count_; ++i) {
            if (networks_[i].operational) {
                efficiency_score += ComputeNetworkEfficiency(networks_[i].network_id);
                valid_networks++;
            }
        }
        if (valid_networks == 0) return 0.0;
        efficiency_score /= valid_networks;
        double energy_score = 1.0 - (energy_consumption_kwh_ / 10000.0);
        double carbon_score = 1.0 - (carbon_footprint_kg_co2e_ / 1000.0);
        return (efficiency_score * 0.5 + energy_score.max(0.0) * 0.25 + carbon_score.max(0.0) * 0.25);
    }
    
    bool VerifyAuditIntegrity() const {
        uint64_t sum = 0;
        sum ^= network_count_ * batch_count_;
        sum ^= total_training_epochs_;
        sum ^= total_inference_requests_;
        for (size_t i = 0; i < network_count_; ++i) {
            sum ^= networks_[i].network_id * static_cast<uint64_t>(networks_[i].operational);
        }
        return sum == audit_checksum_;
    }
    
    void RecordInferenceRequest(double latency_ms, uint64_t now_ns) {
        total_inference_requests_++;
        average_inference_latency_ms_ = (average_inference_latency_ms_ * (total_inference_requests_ - 1) + latency_ms) / total_inference_requests_;
        last_optimization_ns_ = now_ns;
        UpdateAuditChecksum();
    }
    
    void RecordEnergyConsumption(double kwh, double co2e_kg, uint64_t now_ns) {
        energy_consumption_kwh_ += static_cast<uint64_t>(kwh * 1000);
        carbon_footprint_kg_co2e_ += static_cast<uint64_t>(co2e_kg * 1000);
        last_optimization_ns_ = now_ns;
        UpdateAuditChecksum();
    }
};

#endif
