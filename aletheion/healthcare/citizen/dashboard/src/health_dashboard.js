// Aletheion Citizen Health Dashboard v20260310
// License: BioticTreaty_v3
// Compliance: HIPAA_1996_GDPR_2018_Neurorights_v1_WCAG_2.2_AAA

const HEALTH_DASHBOARD_VERSION = 20260310;
const MAX_HEALTH_METRICS = 128;
const MAX_HEALTH_ALERTS = 64;
const MAX_CARE_PROVIDERS = 32;
const DATA_REFRESH_INTERVAL_MS = 300000;

class HealthMetric {
    constructor(metricId, metricType, value, unit, timestampNs) {
        this.metricId = metricId;
        this.metricType = metricType;
        this.value = value;
        this.unit = unit;
        this.timestampNs = timestampNs;
        this.baselineValue = 0.0;
        this.targetRange = { min: 0.0, max: 0.0 };
        this.alertThresholds = { low: 0.0, high: 0.0 };
        this.trend = 'stable';
        this.dataPoints = [];
        this.consentGranted = false;
        this.sharingScope = 'provider_only';
    }
    computeTrend() {
        if (this.dataPoints.length < 2) {
            this.trend = 'stable';
            return;
        }
        const recent = this.dataPoints.slice(-7);
        const slope = (recent[recent.length - 1].value - recent[0].value) / recent.length;
        if (slope > 0.05) this.trend = 'increasing';
        else if (slope < -0.05) this.trend = 'decreasing';
        else this.trend = 'stable';
    }
    isWithinTargetRange() {
        return this.value >= this.targetRange.min && this.value <= this.targetRange.max;
    }
    requiresAlert() {
        return this.value < this.alertThresholds.low || this.value > this.alertThresholds.high;
    }
    addDataPoint(value, timestampNs) {
        this.dataPoints.push({ value, timestampNs });
        if (this.dataPoints.length > 30) {
            this.dataPoints.shift();
        }
        this.value = value;
        this.timestampNs = timestampNs;
        this.computeTrend();
    }
}

class HealthAlert {
    constructor(alertId, alertType, severity, metricId, message, timestampNs) {
        this.alertId = alertId;
        this.alertType = alertType;
        this.severity = severity;
        this.metricId = metricId;
        this.message = message;
        this.timestampNs = timestampNs;
        this.acknowledged = false;
        this.acknowledgedAtNs = 0;
        this.resolved = false;
        this.resolvedAtNs = 0;
        this.acknowledgedBy = '';
        this.assignedProvider = '';
    }
    acknowledge(userId, timestampNs) {
        this.acknowledged = true;
        this.acknowledgedAtNs = timestampNs;
        this.acknowledgedBy = userId;
    }
    resolve(timestampNs) {
        this.resolved = true;
        this.resolvedAtNs = timestampNs;
    }
    isActive() {
        return !this.resolved && !this.acknowledged;
    }
}

class CareProvider {
    constructor(providerId, providerType, name, specialty, organization) {
        this.providerId = providerId;
        this.providerType = providerType;
        this.name = name;
        this.specialty = specialty;
        this.organization = organization;
        this.contactInfo = { phone: '', email: '', address: '' };
        this.authorizationLevel = 1;
        this.consentGranted = false;
        this.consentExpiresNs = 0;
        this.lastVisitNs = 0;
        this.totalVisits = 0;
        this.rating = 0.0;
        this.active = true;
    }
    isAuthorized(nowNs) {
        return this.active && this.consentGranted && nowNs < this.consentExpiresNs;
    }
}

class CitizenHealthDashboard {
    constructor(citizenDid, dashboardId, initTimestampNs) {
        this.citizenDid = citizenDid;
        this.dashboardId = dashboardId;
        this.metrics = {};
        this.metricCount = 0;
        this.alerts = [];
        this.alertCount = 0;
        this.careProviders = {};
        this.providerCount = 0;
        this.consentRecords = [];
        this.healthScore = 100.0;
        this.wellnessIndex = 0.0;
        this.lastDataRefreshNs = initTimestampNs;
        this.lastSyncNs = initTimestampNs;
        this.dataExportRequests = [];
        this.privacySettings = {
            shareWithProviders: true,
            shareForResearch: false,
            shareForCommercial: false,
            neurorightsProtected: true,
            geneticDataProtected: true,
            mentalHealthDataProtected: true,
            autoDeleteAfterDays: 365
        };
        this.emergencyContacts = [];
        this.advancedDirectives = {
            hasLivingWill: false,
            hasHealthcareProxy: false,
            hasDNR: false,
            organDonor: false,
            lastUpdatedNs: initTimestampNs
        };
    }
    addHealthMetric(metric) {
        if (this.metricCount >= MAX_HEALTH_METRICS) return false;
        this.metrics[metric.metricId] = metric;
        this.metricCount++;
        this.computeHealthScore();
        return true;
    }
    updateMetricValue(metricId, value, timestampNs) {
        const metric = this.metrics[metricId];
        if (!metric) return false;
        metric.addDataPoint(value, timestampNs);
        if (metric.requiresAlert()) {
            this.createHealthAlert('THRESHOLD_EXCEEDED', 2, metricId, 
                `${metric.metricType} outside target range: ${value} ${metric.unit}`, timestampNs);
        }
        this.computeHealthScore();
        return true;
    }
    createHealthAlert(alertType, severity, metricId, message, timestampNs) {
        if (this.alertCount >= MAX_HEALTH_ALERTS) return false;
        const alert = new HealthAlert(
            this.alertCount + 1,
            alertType,
            severity,
            metricId,
            message,
            timestampNs
        );
        this.alerts[this.alertCount] = alert;
        this.alertCount++;
        return true;
    }
    acknowledgeAlert(alertId, userId, timestampNs) {
        for (let i = 0; i < this.alertCount; i++) {
            if (this.alerts[i].alertId === alertId) {
                this.alerts[i].acknowledge(userId, timestampNs);
                return true;
            }
        }
        return false;
    }
    resolveAlert(alertId, timestampNs) {
        for (let i = 0; i < this.alertCount; i++) {
            if (this.alerts[i].alertId === alertId) {
                this.alerts[i].resolve(timestampNs);
                return true;
            }
        }
        return false;
    }
    addCareProvider(provider) {
        if (this.providerCount >= MAX_CARE_PROVIDERS) return false;
        this.careProviders[provider.providerId] = provider;
        this.providerCount++;
        return true;
    }
    grantProviderConsent(providerId, expiresDays, nowNs) {
        const provider = this.careProviders[providerId];
        if (!provider) return false;
        provider.consentGranted = true;
        provider.consentExpiresNs = nowNs + (expiresDays * 86400000000000);
        return true;
    }
    revokeProviderConsent(providerId, nowNs) {
        const provider = this.careProviders[providerId];
        if (!provider) return false;
        provider.consentGranted = false;
        return true;
    }
    computeHealthScore() {
        let score = 100.0;
        let metricCount = 0;
        for (const metricId in this.metrics) {
            const metric = this.metrics[metricId];
            if (!metric.isWithinTargetRange()) {
                score -= 10;
            }
            if (metric.trend === 'increasing' && metric.value > metric.baselineValue * 1.2) {
                score -= 5;
            }
            if (metric.trend === 'decreasing' && metric.value < metric.baselineValue * 0.8) {
                score -= 5;
            }
            metricCount++;
        }
        const activeAlerts = this.alerts.filter(a => a.isActive()).length;
        score -= activeAlerts * 5;
        this.healthScore = Math.max(0.0, Math.min(100.0, score));
        this.wellnessIndex = this.healthScore / 100.0;
        return this.healthScore;
    }
    getActiveAlerts() {
        return this.alerts.filter(a => a.isActive());
    }
    getUnacknowledgedAlerts() {
        return this.alerts.filter(a => !a.acknowledged && !a.resolved);
    }
    getAuthorizedProviders(nowNs) {
        const authorized = [];
        for (const providerId in this.careProviders) {
            if (this.careProviders[providerId].isAuthorized(nowNs)) {
                authorized.push(this.careProviders[providerId]);
            }
        }
        return authorized;
    }
    requestDataExport(requestType, recipient, nowNs) {
        this.dataExportRequests.push({
            requestId: this.dataExportRequests.length + 1,
            requestType,
            recipient,
            requestedAtNs: nowNs,
            completed: false,
            completedAtNs: 0
        });
    }
    computePrivacyScore() {
        let score = 1.0;
        if (!this.privacySettings.neurorightsProtected) score -= 0.3;
        if (!this.privacySettings.geneticDataProtected) score -= 0.2;
        if (!this.privacySettings.mentalHealthDataProtected) score -= 0.2;
        if (this.privacySettings.shareForCommercial) score -= 0.2;
        const authorizedProviders = Object.values(this.careProviders).filter(p => p.consentGranted).length;
        if (authorizedProviders > 10) score -= 0.1;
        return Math.max(0.0, score);
    }
    getDashboardStatus(nowNs) {
        const activeAlerts = this.getActiveAlerts().length;
        const unacknowledgedAlerts = this.getUnacknowledgedAlerts().length;
        const authorizedProviders = this.getAuthorizedProviders(nowNs).length;
        const metricsOutOfTarget = Object.values(this.metrics).filter(m => !m.isWithinTargetRange()).length;
        return {
            citizenDid: this.citizenDid,
            dashboardId: this.dashboardId,
            healthScore: this.healthScore,
            wellnessIndex: this.wellnessIndex,
            privacyScore: this.computePrivacyScore(),
            totalMetrics: this.metricCount,
            metricsOutOfTarget,
            totalAlerts: this.alertCount,
            activeAlerts,
            unacknowledgedAlerts,
            totalProviders: this.providerCount,
            authorizedProviders,
            lastDataRefreshNs: this.lastDataRefreshNs,
            lastSyncNs: this.lastSyncNs,
            neurorightsProtected: this.privacySettings.neurorightsProtected,
            hasAdvancedDirectives: this.advancedDirectives.hasLivingWill || 
                                   this.advancedDirectives.hasHealthcareProxy
        };
    }
    computeReadinessForEmergency() {
        let readiness = 1.0;
        if (!this.advancedDirectives.hasLivingWill) readiness -= 0.2;
        if (!this.advancedDirectives.hasHealthcareProxy) readiness -= 0.2;
        if (this.emergencyContacts.length === 0) readiness -= 0.2;
        const activeAlerts = this.getActiveAlerts().length;
        if (activeAlerts > 0) readiness -= 0.1 * activeAlerts;
        return Math.max(0.0, readiness);
    }
}

class PopulationHealthAnalytics {
    constructor(cityCode, analyticsId) {
        this.cityCode = cityCode;
        this.analyticsId = analyticsId;
        this.aggregatedMetrics = {};
        this.healthTrends = {};
        this.outbreakAlerts = [];
        this.resourceUtilization = {};
    }
    aggregateDashboardData(dashboards) {
        const metrics = {};
        for (const dashboard of dashboards) {
            for (const metricId in dashboard.metrics) {
                const metric = dashboard.metrics[metricId];
                if (!metrics[metric.metricType]) {
                    metrics[metric.metricType] = { values: [], count: 0 };
                }
                metrics[metric.metricType].values.push(metric.value);
                metrics[metric.metricType].count++;
            }
        }
        for (const metricType in metrics) {
            const values = metrics[metricType].values;
            this.aggregatedMetrics[metricType] = {
                mean: values.reduce((a, b) => a + b, 0) / values.length,
                median: values.sort((a, b) => a - b)[Math.floor(values.length / 2)],
                stdDev: this.computeStandardDeviation(values),
                count: metrics[metricType].count
            };
        }
    }
    computeStandardDeviation(values) {
        const mean = values.reduce((a, b) => a + b, 0) / values.length;
        const squareDiffs = values.map(value => Math.pow(value - mean, 2));
        return Math.sqrt(squareDiffs.reduce((a, b) => a + b, 0) / values.length);
    }
    detectOutbreak(metricType, thresholdStdDev) {
        const metric = this.aggregatedMetrics[metricType];
        if (!metric) return false;
        if (metric.stdDev > thresholdStdDev * metric.mean) {
            this.outbreakAlerts.push({
                metricType,
                detectedAtNs: Date.now() * 1000000,
                severity: metric.stdDev / metric.mean,
                affectedPopulation: metric.count
            });
            return true;
        }
        return false;
    }
    computePopulationWellnessIndex() {
        const healthScores = Object.values(this.aggregatedMetrics).map(m => m.mean);
        if (healthScores.length === 0) return 0.0;
        return healthScores.reduce((a, b) => a + b, 0) / healthScores.length;
    }
}

module.exports = {
    HealthMetric,
    HealthAlert,
    CareProvider,
    CitizenHealthDashboard,
    PopulationHealthAnalytics,
    VERSION: HEALTH_DASHBOARD_VERSION,
};
