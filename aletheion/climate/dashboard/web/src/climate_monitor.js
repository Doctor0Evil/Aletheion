// aletheion/climate/dashboard/web/src/climate_monitor.js
// Copyright (c) 2026 Aletheion City-OS. All Rights Reserved.
// License: BioticTreaty-Compliant AGPL-3.0-or-later with Indigenous-Rights-Clause
// Purpose: Dashboard & Observability for Phoenix Desert Grid (Interface → Log → Audit)
// Constraints: No Blacklisted Crypto (SHA/Blake/Argon), No Rollbacks, Offline-First, Indigenous-Rights-Hardened
// Status: ACTIVE | VERSION: 1.0.0-E-PHX | TERRITORY: Akimel O'odham & Piipaash Traditional Lands
// Identity: Augmented-Citizen Organically-Integrated (BI-Bound)

// ============================================================================
// MODULE IMPORTS & ABSTRACTIONS (Cross-Language Interop)
// ============================================================================
// Per Rule (L): Supported-language set: ALN, Lua, Rust, Javascript, Kotlin/Android, C++
// Per Rule (R): No blacklisted tech. Post-Quantum Secure Abstraction.

import { PQSecure } from 'aletheion.core.security.pq'; // Abstracted PQ Crypto
import { SensorBridge } from 'aletheion.climate.edge.sensor'; // Rust WASM Bridge
import { AuditChain } from 'aletheion.climate.audit.chain'; // Rust/ALN Bridge
import { TreatyValidator } from 'aletheion.climate.rights.treaty'; // Indigenous Rights Bridge

// ============================================================================
// CONSTANTS & THRESHOLDS (Phoenix Desert Grid Specifics - ALN Aligned)
// ============================================================================
// Per Rule (E): Desert-climate optimization, Monsoon resilience, Air quality.
// Per Rule (P): Node-Placement opportunities where civil-disturbance will-not create unrest.

const CONFIG = Object.freeze({
    THRESHOLDS: {
        MAX_TEMP_F: 120.0,
        COOL_PAVEMENT_TRIGGER_F: 105.0,
        MONSOON_CAPTURE_IN: 2.71,
        DUST_ALERT_PM10: 150.0,
        DUST_ALERT_PM25: 55.0,
        WATER_RECLAIM_PCT: 99.0,
        COGNITIVE_LOAD_MAX: 0.7
    },
    TERRITORY: {
        NATION_PRIMARY: "Akimel O'odham",
        NATION_SECONDARY: "Piipaash",
        ACKNOWLEDGMENT_REQUIRED: true
    },
    UI: {
        REFRESH_RATE_MS: 1000,
        AUDIT_LOG_MAX: 100,
        OFFLINE_BUFFER_MAX: 500,
        COLOR_CRITICAL: '#FF4444',
        COLOR_SAFE: '#00C851',
        COLOR_WARNING: '#FFBB33',
        COLOR_OFFLINE: '#AA66CC'
    },
    RIGHTS: {
        NEURORIGHTS_ENABLED: true,
        BIOTIC_TREATY_ENABLED: true,
        CONSENT_VISIBLE: true
    }
});

// ============================================================================
// STATE MANAGEMENT (Immutable, Forward-Only)
// ============================================================================
// Per Rule (R): No rollbacks, no digital twins, no fictional content.
// Per Rule (L): High-density codes, syntax_ladders.

class ImmutableState {
    constructor(initial) {
        this._state = Object.freeze(initial);
    }
    update(updates) {
        return new ImmutableState(Object.freeze({ ...this._state, ...updates }));
    }
    get() { return this._state; }
}

const initialState = {
    sensors: { temp: 0, pm10: 0, rain: 0, humidity: 0, aquifer: 0 },
    machinery: { pumps: 'OFF', valves: 'CLOSED', pavement: 'INACTIVE' },
    audit: [],
    consent: { pending: 0, granted: 0, denied: 0, cognitive_load: 0.0 },
    territory: { acknowledged: false, status: 'PENDING' },
    network: { online: true, last_sync: 0 },
    alerts: []
};

let state = new ImmutableState(initialState);

// ============================================================================
// CORE DASHBOARD CONTROLLER (ERM Chain: Interface → Log → Audit)
// ============================================================================
// Per Rule (E): Monsoon resilience: flash-flood management systems, stormwater harvesting.
// Per Rule (I): DID-Bound brain-identity (BI) and biosignal-collector respect.

class ClimateDashboard {
    constructor() {
        this.sensorBridge = new SensorBridge();
        this.auditChain = new AuditChain();
        this.treatyValidator = new TreatyValidator();
        this.pqSecure = new PQSecure();
        this.container = null;
        this.interval = null;
    }

    init(containerId) {
        this.container = document.getElementById(containerId);
        if (!this.container) throw new Error("Dashboard Container Missing");
        this.renderTerritoryBanner(); // Hard Constraint: Render First
        this.startPolling();
        this.setupOfflineListener();
        this.logSystemEvent("Dashboard_Init", this.pqSecure.hash("JS_v1"));
    }

    startPolling() {
        this.interval = setInterval(async () => {
            try {
                const sensorData = await this.sensorBridge.getFusedContext(); // Rust FFI
                const auditData = await this.auditChain.getRecentLogs(CONFIG.UI.AUDIT_LOG_MAX);
                const consentData = await this.sensorBridge.getConsentMetrics(); // Kotlin Bridge
                const networkStatus = navigator.onLine;
                
                state = state.update({
                    sensors: { ...sensorData },
                    audit: auditData,
                    consent: { ...consentData },
                    network: { online: networkStatus, last_sync: Date.now() }
                });
                this.checkThresholds();
                this.render();
            } catch (err) {
                this.handleOffline();
            }
        }, CONFIG.UI.REFRESH_RATE_MS);
    }

    checkThresholds() {
        const s = state.get().sensors;
        const alerts = [];
        if (s.temp >= CONFIG.THRESHOLDS.MAX_TEMP_F) alerts.push({ type: 'CRITICAL', msg: `Heat Critical: ${s.temp}°F` });
        if (s.pm10 >= CONFIG.THRESHOLDS.DUST_ALERT_PM10) alerts.push({ type: 'WARNING', msg: `Dust Alert: ${s.pm10} PM10` });
        if (s.rain >= CONFIG.THRESHOLDS.MONSOON_CAPTURE_IN) alerts.push({ type: 'INFO', msg: `Monsoon Capture Active: ${s.rain}"` });
        if (state.get().consent.cognitive_load > CONFIG.THRESHOLDS.COGNITIVE_LOAD_MAX) alerts.push({ type: 'NEURORIGHTS', msg: `High Cognitive Load Detected` });
        state = state.update({ alerts });
    }

    handleOffline() {
        state = state.update({ network: { ...state.get().network, online: false } });
        console.warn("[OFFLINE] Dashboard operating on local cache");
        // Offline-First: Continue rendering last known good state
    }

    setupOfflineListener() {
        window.addEventListener('online', () => state = state.update({ network: { ...state.get().network, online: true } }));
        window.addEventListener('offline', () => this.handleOffline());
    }

    logSystemEvent(event, hash) {
        // Local log before pushing to chain
        console.log(`[AUDIT] ${event} | Hash: ${hash}`);
    }

    // ============================================================================
    // RENDERING ENGINE (High-Density, Accessibility-First)
    // ============================================================================
    // Per Rule (L): Compatibility: Github, and adjustable to any city-builder, or deployment-guide.
    // Per Rule (P): Avoids: War, Civil-Unrest, Industrial-Route Conflict.

    render() {
        if (!this.container) return;
        const s = state.get();
        this.container.innerHTML = ''; // Clear (In production, use Virtual DOM for efficiency)
        
        // 1. Network Status
        this.container.appendChild(this.createStatusBar(s.network));
        
        // 2. Environmental Metrics
        this.container.appendChild(this.createMetricsGrid(s.sensors));
        
        // 3. Machinery Status
        this.container.appendChild(this.createMachineryPanel(s.machinery));
        
        // 4. Audit Log (Immutable)
        this.container.appendChild(this.createAuditLog(s.audit));
        
        // 5. Consent & Neurorights
        this.container.appendChild(this.createConsentPanel(s.consent));
        
        // 6. Alerts
        if (s.alerts.length > 0) this.container.appendChild(this.createAlertPanel(s.alerts));
    }

    createStatusBar(network) {
        const div = document.createElement('div');
        div.style.cssText = "background:#333;color:#fff;padding:10px;display:flex;justify-content:space-between;";
        div.innerHTML = `<span>ALETHEION CLIMATE GRID | PHOENIX</span><span style="color:${network.online ? CONFIG.UI.COLOR_SAFE : CONFIG.UI.COLOR_OFFLINE}">${network.online ? 'ONLINE' : 'OFFLINE'}</span>`;
        return div;
    }

    createMetricsGrid(sensors) {
        const div = document.createElement('div');
        div.style.cssText = "display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:10px;margin:10px 0;";
        const metrics = [
            { label: 'Air Temp', value: `${sensors.temp}°F`, color: sensors.temp >= CONFIG.THRESHOLDS.MAX_TEMP_F ? CONFIG.UI.COLOR_CRITICAL : CONFIG.UI.COLOR_SAFE },
            { label: 'PM10', value: `${sensors.pm10} µg/m³`, color: sensors.pm10 >= CONFIG.THRESHOLDS.DUST_ALERT_PM10 ? CONFIG.UI.COLOR_WARNING : CONFIG.UI.COLOR_SAFE },
            { label: 'Rainfall', value: `${sensors.rain}"`, color: CONFIG.UI.COLOR_SAFE },
            { label: 'Aquifer', value: `${sensors.aquifer}ft`, color: CONFIG.UI.COLOR_SAFE }
        ];
        metrics.forEach(m => {
            const card = document.createElement('div');
            card.style.cssText = `background:#222;color:${m.color};padding:15px;border-radius:4px;text-align:center;`;
            card.innerHTML = `<div style="font-size:0.8em">${m.label}</div><div style="font-size:1.5em;font-weight:bold">${m.value}</div>`;
            div.appendChild(card);
        });
        return div;
    }

    createMachineryPanel(machinery) {
        const div = document.createElement('div');
        div.style.cssText = "background:#222;padding:10px;margin:10px 0;border-radius:4px;";
        div.innerHTML = `<h3 style="margin:0 0 10px 0;color:#fff;">Machinery Status</h3><div>${Object.entries(machinery).map(([k,v]) => `<span style="margin-right:15px">${k}: <strong>${v}</strong></span>`).join('')}</div>`;
        return div;
    }

    createAuditLog(audit) {
        const div = document.createElement('div');
        div.style.cssText = "background:#111;padding:10px;margin:10px 0;border-radius:4px;max-height:200px;overflow-y:auto;";
        div.innerHTML = `<h3 style="margin:0 0 10px 0;color:#fff;">Immutable Audit Chain</h3>`;
        const ul = document.createElement('ul');
        ul.style.cssText = "list-style:none;padding:0;margin:0;font-family:monospace;font-size:0.8em;color:#aaa;";
        audit.forEach(entry => {
            const li = document.createElement('li');
            li.style.cssText = "border-bottom:1px solid #333;padding:5px 0;";
            li.textContent = `[${new Date(entry.ts).toISOString()}] ${entry.action} | ${entry.result}`;
            ul.appendChild(li);
        });
        div.appendChild(ul);
        return div;
    }

    createConsentPanel(consent) {
        const div = document.createElement('div');
        div.style.cssText = "background:#222;padding:10px;margin:10px 0;border-radius:4px;";
        const loadColor = consent.cognitive_load > CONFIG.THRESHOLDS.COGNITIVE_LOAD_MAX ? CONFIG.UI.COLOR_CRITICAL : CONFIG.UI.COLOR_SAFE;
        div.innerHTML = `
            <h3 style="margin:0 0 10px 0;color:#fff;">Citizen Consent & Neurorights</h3>
            <div style="display:flex;justify-content:space-between;margin-bottom:5px;">
                <span>Granted: ${consent.granted}</span><span>Denied: ${consent.denied}</span><span>Pending: ${consent.pending}</span>
            </div>
            <div style="background:#333;height:10px;border-radius:5px;overflow:hidden;">
                <div style="background:${loadColor};width:${consent.cognitive_load * 100}%;height:100%;"></div>
            </div>
            <div style="font-size:0.8em;color:#aaa;margin-top:5px;">Aggregate Cognitive Load: ${(consent.cognitive_load * 100).toFixed(1)}%</div>
        `;
        return div;
    }

    createAlertPanel(alerts) {
        const div = document.createElement('div');
        div.style.cssText = "background:#330000;color:#ffcccc;padding:10px;margin:10px 0;border-radius:4px;border:1px solid #ff0000;";
        div.innerHTML = `<h3 style="margin:0 0 10px 0;">Active Alerts</h3><ul>${alerts.map(a => `<li>${a.msg}</li>`).join('')}</ul>`;
        return div;
    }

    renderTerritoryBanner() {
        if (!this.container) return;
        const banner = document.createElement('div');
        banner.style.cssText = "background:#004d40;color:#fff;padding:15px;text-align:center;border-bottom:2px solid #00e5ff;";
        banner.innerHTML = `
            <div style="font-weight:bold;">Territory Acknowledgment</div>
            <div style="font-size:0.9em;margin-top:5px;">This grid operates on the traditional lands of the ${CONFIG.TERRITORY.NATION_PRIMARY} and ${CONFIG.TERRITORY.NATION_SECONDARY} nations.</div>
            <div style="font-size:0.8em;margin-top:5px;opacity:0.8;">Himad Dak Do'ag (We Are All Together)</div>
        `;
        this.container.insertBefore(banner, this.container.firstChild);
    }

    destroy() {
        if (this.interval) clearInterval(this.interval);
        this.container = null;
    }
}

// ============================================================================
// EXPORTS & INITIALIZATION (Global Scope for Embedding)
// ============================================================================
// Per Rule (R): Codes must-be in the supported-languages, contain a filename, and an exact-destination.
// Per Rule (L): Compatibility: Github, and adjustable to any city-builder, or deployment-guide.

window.AletheionClimateDashboard = ClimateDashboard;
window.AletheionConfig = CONFIG;

// Auto-Init if container exists
document.addEventListener('DOMContentLoaded', () => {
    const container = document.getElementById('aletheion-climate-dashboard');
    if (container) {
        const dashboard = new ClimateDashboard();
        dashboard.init('aletheion-climate-dashboard');
        window.aletheionDashboardInstance = dashboard;
    }
});
