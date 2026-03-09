// Aletheion Community Resilience & Recovery Network v20260310
// License: BioticTreaty_v3
// Compliance: FEMA_Community_Resilience_2026_Arizona_Emergency_Management_BioticTreaty_v3

const RESILIENCE_NETWORK_VERSION = 20260310;
const MAX_COMMUNITY_HUBS = 1024;
const MAX_RESILIENCE_PROJECTS = 8192;
const MAX_VOLUNTEER_NETWORKS = 16384;
const MAX_RECOVERY_CASES = 65536;
const TARGET_RECOVERY_TIME_DAYS = 30;
const MIN_RESILIENCE_SCORE = 0.70;

class CommunityHub {
    constructor(hubId, hubName, latitude, longitude, capacity) {
        this.hubId = hubId;
        this.hubName = hubName;
        this.latitude = latitude;
        this.longitude = longitude;
        this.capacity = capacity;
        this.currentOccupancy = 0;
        this.status = 'INACTIVE';
        this.servicesOffered = [];
        this.staffCount = 0;
        this.volunteerCount = 0;
        this.resourcesAvailable = {};
        this.accessibilityCompliant = true;
        this.multilingualSupport = true;
        this.indigenousCulturalCompetency = false;
        this.operational = true;
        this.activatedAtNs = 0;
        this.deactivatedAtNs = 0;
        this.lastInspectionNs = 0;
    }
    activate(nowNs) {
        if (!this.operational) return false;
        this.status = 'ACTIVE';
        this.activatedAtNs = nowNs;
        return true;
    }
    deactivate(nowNs) {
        this.status = 'INACTIVE';
        this.deactivatedAtNs = nowNs;
        this.currentOccupancy = 0;
        return true;
    }
    occupancyRatio() {
        return this.capacity > 0 ? this.currentOccupancy / this.capacity : 0.0;
    }
    addService(service) {
        this.servicesOffered.push(service);
    }
    computeHubResilienceScore() {
        let score = 0.0;
        score += this.servicesOffered.length >= 5 ? 0.3 : this.servicesOffered.length * 0.06;
        score += this.volunteerCount > 50 ? 0.25 : this.volunteerCount / 200;
        score += this.accessibilityCompliant ? 0.15 : 0.0;
        score += this.multilingualSupport ? 0.15 : 0.0;
        score += this.indigenousCulturalCompetency ? 0.15 : 0.0;
        return Math.min(1.0, score);
    }
}

class ResilienceProject {
    constructor(projectId, projectName, projectType, communityHubId) {
        this.projectId = projectId;
        this.projectName = projectName;
        this.projectType = projectType;
        this.communityHubId = communityHubId;
        this.status = 'PLANNING';
        this.startDateNs = 0;
        this.targetEndDateNs = 0;
        this.actualEndDateNs = 0;
        this.budgetUsd = 0.0;
        this.spentUsd = 0.0;
        this.volunteersAssigned = 0;
        this.beneficiariesCount = 0;
        this.completionPct = 0.0;
        this.resilienceImpactScore = 0.0;
        this.communityEngagementScore = 0.0;
        this.sustainabilityScore = 0.0;
        this.createdAtNs = Date.now() * 1000000;
        this.lastUpdatedNs = Date.now() * 1000000;
    }
    start(nowNs) {
        if (this.status !== 'PLANNING') return false;
        this.status = 'ACTIVE';
        this.startDateNs = nowNs;
        return true;
    }
    complete(nowNs) {
        if (this.status !== 'ACTIVE') return false;
        this.status = 'COMPLETED';
        this.actualEndDateNs = nowNs;
        this.completionPct = 100.0;
        return true;
    }
    durationDays(nowNs) {
        const end = this.actualEndDateNs > 0 ? this.actualEndDateNs : nowNs;
        return (end - this.startDateNs) / 86400000000000;
    }
    computeProjectImpact() {
        const timeEfficiency = this.actualEndDateNs > 0 ? 
            Math.min(1.0, TARGET_RECOVERY_TIME_DAYS / this.durationDays(Date.now() * 1000000)) : 0.5;
        const budgetEfficiency = this.budgetUsd > 0 ? 
            Math.min(1.0, 1.0 - (this.spentUsd / this.budgetUsd - 1.0).Math.max(0)) : 1.0;
        const beneficiaryReach = this.beneficiariesCount > 1000 ? 1.0 : this.beneficiariesCount / 1000;
        this.resilienceImpactScore = (timeEfficiency * 0.35 + budgetEfficiency * 0.25 + 
            beneficiaryReach * 0.40).Math.min(1.0);
        return this.resilienceImpactScore;
    }
}

class VolunteerNetwork {
    constructor(networkId, networkName, specialty, communityHubId) {
        this.networkId = networkId;
        this.networkName = networkName;
        this.specialty = specialty;
        this.communityHubId = communityHubId;
        this.volunteers = [];
        this.volunteerCount = 0;
        this.activeVolunteers = 0;
        this.trainedVolunteers = 0;
        this.certifiedVolunteers = 0;
        this.hoursContributed = 0;
        this.deploymentsCount = 0;
        this.satisfactionScore = 0.0;
        this.retentionRate = 0.0;
        this.createdAtNs = Date.now() * 1000000;
        this.lastActivityNs = Date.now() * 1000000;
    }
    addVolunteer(volunteer) {
        this.volunteers.push(volunteer);
        this.volunteerCount++;
        if (volunteer.active) this.activeVolunteers++;
        if (volunteer.trained) this.trainedVolunteers++;
        if (volunteer.certified) this.certifiedVolunteers++;
    }
    computeNetworkCapacity() {
        const trainingRate = this.trainedVolunteers / this.volunteerCount.max(1);
        const certificationRate = this.certifiedVolunteers / this.volunteerCount.max(1);
        const activationRate = this.activeVolunteers / this.volunteerCount.max(1);
        return (trainingRate * 0.35 + certificationRate * 0.35 + activationRate * 0.30).Math.min(1.0);
    }
}

class RecoveryCase {
    constructor(caseId, citizenDid, disasterType, severity, reportedNs) {
        this.caseId = caseId;
        this.citizenDid = citizenDid;
        this.disasterType = disasterType;
        this.severity = severity;
        this.reportedNs = reportedNs;
        this.status = 'OPEN';
        this.assignedCaseWorker = '';
        this.servicesProvided = [];
        this.fundingApprovedUsd = 0.0;
        this.fundingDisbursedUsd = 0.0;
        this.housingAssistance = false;
        this.medicalAssistance = false;
        this.financialAssistance = false;
        this.counselingProvided = false;
        this.resolvedNs = 0;
        this.satisfactionRating = 0.0;
        this.followUpRequired = false;
    }
    resolve(nowNs, satisfactionRating) {
        this.status = 'RESOLVED';
        this.resolvedNs = nowNs;
        this.satisfactionRating = satisfactionRating;
    }
    recoveryTimeDays(nowNs) {
        const end = this.resolvedNs > 0 ? this.resolvedNs : nowNs;
        return (end - this.reportedNs) / 86400000000000;
    }
}

class CommunityResilienceNetwork {
    constructor(networkId, cityCode, region) {
        this.networkId = networkId;
        this.cityCode = cityCode;
        this.region = region;
        this.communityHubs = new Map();
        this.hubCount = 0;
        this.resilienceProjects = new Map();
        this.projectCount = 0;
        this.volunteerNetworks = new Map();
        this.volunteerNetworkCount = 0;
        this.recoveryCases = new Map();
        this.caseCount = 0;
        this.totalVolunteers = 0;
        this.totalVolunteerHours = 0;
        this.totalFundingDistributedUsd = 0.0;
        this.averageRecoveryTimeDays = 0.0;
        this.communityResilienceIndex = 0.0;
        this.lastMajorDisasterNs = Date.now() * 1000000;
    }
    registerCommunityHub(hub) {
        if (this.hubCount >= MAX_COMMUNITY_HUBS) return false;
        if (!hub.accessibilityCompliant) return false;
        this.communityHubs.set(hub.hubId, hub);
        this.hubCount++;
        return true;
    }
    registerResilienceProject(project) {
        if (this.projectCount >= MAX_RESILIENCE_PROJECTS) return false;
        this.resilienceProjects.set(project.projectId, project);
        this.projectCount++;
        return true;
    }
    registerVolunteerNetwork(network) {
        if (this.volunteerNetworkCount >= MAX_VOLUNTEER_NETWORKS) return false;
        this.volunteerNetworks.set(network.networkId, network);
        this.volunteerNetworkCount++;
        this.totalVolunteers += network.volunteerCount;
        return true;
    }
    openRecoveryCase(case_) {
        if (this.caseCount >= MAX_RECOVERY_CASES) return false;
        this.recoveryCases.set(case_.caseId, case_);
        this.caseCount++;
        return true;
    }
    closeRecoveryCase(caseId, nowNs, satisfactionRating) {
        const case_ = this.recoveryCases.get(caseId);
        if (!case_) return false;
        case_.resolve(nowNs, satisfactionRating);
        this.updateAverageRecoveryTime(case_.recoveryTimeDays(nowNs));
        return true;
    }
    updateAverageRecoveryTime(newTimeDays) {
        const resolvedCases = Array.from(this.recoveryCases.values()).filter(c => c.status === 'RESOLVED').length;
        this.averageRecoveryTimeDays = (this.averageRecoveryTimeDays * (resolvedCases - 1) + newTimeDays) / 
            resolvedCases.max(1);
    }
    computeCommunityResilienceIndex() {
        const hubScores = Array.from(this.communityHubs.values()).map(h => h.computeHubResilienceScore());
        const avgHubScore = hubScores.length > 0 ? hubScores.reduce((a, b) => a + b, 0) / hubScores.length : 0.0;
        const projectScores = Array.from(this.resilienceProjects.values())
            .filter(p => p.status === 'COMPLETED').map(p => p.computeProjectImpact());
        const avgProjectScore = projectScores.length > 0 ? projectScores.reduce((a, b) => a + b, 0) / projectScores.length : 0.0;
        const volunteerCapacity = Array.from(this.volunteerNetworks.values())
            .map(v => v.computeNetworkCapacity());
        const avgVolunteerCapacity = volunteerCapacity.length > 0 ? 
            volunteerCapacity.reduce((a, b) => a + b, 0) / volunteerCapacity.length : 0.0;
        const recoveryRate = Array.from(this.recoveryCases.values())
            .filter(c => c.status === 'RESOLVED').length / this.caseCount.max(1);
        const recoveryScore = this.averageRecoveryTimeDays <= TARGET_RECOVERY_TIME_DAYS ? 1.0 : 
            TARGET_RECOVERY_TIME_DAYS / this.averageRecoveryTimeDays;
        this.communityResilienceIndex = (avgHubScore * 0.25 + avgProjectScore * 0.25 + 
            avgVolunteerCapacity * 0.20 + recoveryRate * 0.15 + recoveryScore * 0.15).Math.min(1.0);
        return this.communityResilienceIndex;
    }
    identifyResilienceGaps() {
        const gaps = [];
        for (const [, hub] of this.communityHubs) {
            const hubScore = hub.computeHubResilienceScore();
            if (hubScore < MIN_RESILIENCE_SCORE) {
                gaps.push({
                    type: 'COMMUNITY_HUB',
                    hubId: hub.hubId,
                    hubName: hub.hubName,
                    resilienceScore: hubScore,
                    missingServices: this.identifyMissingServices(hub),
                    priority: hubScore < 0.5 ? 'CRITICAL' : 'HIGH'
                });
            }
        }
        return gaps.sort((a, b) => a.resilienceScore - b.resilienceScore);
    }
    identifyMissingServices(hub) {
        const requiredServices = ['medical', 'counseling', 'legal', 'housing', 'financial', 'translation'];
        return requiredServices.filter(s => !hub.servicesOffered.includes(s));
    }
    getNetworkStatus(nowNs) {
        const activeHubs = Array.from(this.communityHubs.values()).filter(h => h.status === 'ACTIVE').length;
        const activeProjects = Array.from(this.resilienceProjects.values()).filter(p => p.status === 'ACTIVE').length;
        const completedProjects = Array.from(this.resilienceProjects.values()).filter(p => p.status === 'COMPLETED').length;
        const resolvedCases = Array.from(this.recoveryCases.values()).filter(c => c.status === 'RESOLVED').length;
        this.computeCommunityResilienceIndex();
        return {
            networkId: this.networkId,
            cityCode: this.cityCode,
            region: this.region,
            totalCommunityHubs: this.hubCount,
            activeCommunityHubs: activeHubs,
            totalResilienceProjects: this.projectCount,
            activeProjects,
            completedProjects,
            totalVolunteerNetworks: this.volunteerNetworkCount,
            totalVolunteers: this.totalVolunteers,
            totalVolunteerHours: this.totalVolunteerHours,
            totalRecoveryCases: this.caseCount,
            resolvedCases,
            totalFundingDistributedUsd: this.totalFundingDistributedUsd,
            averageRecoveryTimeDays: this.averageRecoveryTimeDays,
            communityResilienceIndex: this.communityResilienceIndex,
            lastMajorDisasterNs: this.lastMajorDisasterNs,
            lastUpdateNs: nowNs
        };
    }
    computeRecoveryEffectiveness() {
        const recoveryRate = Array.from(this.recoveryCases.values())
            .filter(c => c.status === 'RESOLVED').length / this.caseCount.max(1);
        const avgSatisfaction = Array.from(this.recoveryCases.values())
            .filter(c => c.status === 'RESOLVED' && c.satisfactionRating > 0)
            .reduce((sum, c) => sum + c.satisfactionRating, 0) / 
            Array.from(this.recoveryCases.values()).filter(c => c.status === 'RESOLVED').length.max(1);
        const timeEfficiency = this.averageRecoveryTimeDays <= TARGET_RECOVERY_TIME_DAYS ? 1.0 : 
            TARGET_RECOVERY_TIME_DAYS / this.averageRecoveryTimeDays;
        return (recoveryRate * 0.4 + avgSatisfaction * 0.3 + timeEfficiency * 0.3).Math.min(1.0);
    }
}

module.exports = {
    CommunityHub,
    ResilienceProject,
    VolunteerNetwork,
    RecoveryCase,
    CommunityResilienceNetwork,
    VERSION: RESILIENCE_NETWORK_VERSION,
    TARGET_RECOVERY_TIME_DAYS,
    MIN_RESILIENCE_SCORE,
};
