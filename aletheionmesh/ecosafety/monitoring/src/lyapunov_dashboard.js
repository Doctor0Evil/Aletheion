// FILE: aletheionmesh/ecosafety/monitoring/src/lyapunov_dashboard.js
// DESTINATION: https://github.com/Doctor0Evil/Aletheion/blob/main/aletheionmesh/ecosafety/monitoring/src/lyapunov_dashboard.js
// LANGUAGE: JavaScript (ES2024+, Offline-Capable, No External Dependencies)
// LICENSE: Aletheion Public License (APL-1.0) + BioticTreaty Clause 7
// STATUS: Production-Ready, Offline-Capable, Treaty-Bound
// CONTEXT: Environmental & Climate Integration (E) - Real-Time Lyapunov Visualization
// PROGRESS: File 4 of 47 (Ecosafety Spine Phase) | 8.51% Complete
// BINDING: Integrates with city_object_guard.rs, environmental_risk_coordinates.aln, cyboquatic_controller.lua

// ============================================================================
// MODULE: Aletheion Lyapunov Stability Dashboard
// PURPOSE: Real-time visualization of V_t stability metrics across all city objects
// CONSTRAINTS: No rollbacks, Indigenous treaty overlay, Offline-first architecture
// DEPLOYMENT: Citizen-facing web interface, municipal operations center, mobile apps
// ============================================================================

(function(global) {
    'use strict';

    // ========================================================================
    // SECTION 1: CORE DASHBOARD CLASS DEFINITION
    // ========================================================================

    class LyapunovDashboard {
        constructor(config = {}) {
            this.config = {
                refreshIntervalMs: config.refreshIntervalMs || 1000,
                maxHistoryPoints: config.maxHistoryPoints || 300,
                stabilityThreshold: config.stabilityThreshold || 0.0001,
                violationAlertThreshold: config.violationAlertThreshold || 3,
                treatyZoneOverlay: config.treatyZoneOverlay || true,
                offlineMode: config.offlineMode || true,
                auditLogging: config.auditLogging || true,
                ...config
            };

            this.state = {
                objects: new Map(),
                violations: [],
                auditTrail: [],
                lastUpdate: 0,
                connectionStatus: 'offline',
                treatyZones: [],
                emergencyOverrides: []
            };

            this.canvas = null;
            this.ctx = null;
            this.animationFrame = null;
            this.eventListeners = new Map();
            this.webWorkers = [];
            this.indexedDB = null;
            this.cacheVersion = 'v1.0.0';

            this.initialize();
        }

        // ====================================================================
        // SECTION 2: INITIALIZATION & OFFLINE STORAGE
        // ====================================================================

        async initialize() {
            await this.setupIndexedDB();
            await this.loadCachedData();
            this.setupEventListeners();
            this.startRenderLoop();
            this.startDataPolling();
            this.logAudit('DASHBOARD_INITIALIZED', {
                timestamp: Date.now(),
                config: this.config,
                cacheVersion: this.cacheVersion
            });
        }

        async setupIndexedDB() {
            return new Promise((resolve, reject) => {
                const request = indexedDB.open('AletheionLyapunovDB', 1);

                request.onerror = () => reject(request.error);
                request.onsuccess = () => {
                    this.indexedDB = request.result;
                    resolve(this.indexedDB);
                };

                request.onupgradeneeded = (event) => {
                    const db = event.target.result;

                    if (!db.objectStoreNames.contains('objectStates')) {
                        const objectStore = db.createObjectStore('objectStates', { keyPath: 'objectId' });
                        objectStore.createIndex('objectClass', 'objectClass', { unique: false });
                        objectStore.createIndex('treatyZone', 'treatyZoneId', { unique: false });
                        objectStore.createIndex('lastUpdate', 'timestamp', { unique: false });
                    }

                    if (!db.objectStoreNames.contains('violations')) {
                        const violationStore = db.createObjectStore('violations', { keyPath: 'id', autoIncrement: true });
                        violationStore.createIndex('timestamp', 'timestamp', { unique: false });
                        violationStore.createIndex('violationType', 'violationType', { unique: false });
                        violationStore.createIndex('objectId', 'objectId', { unique: false });
                    }

                    if (!db.objectStoreNames.contains('auditTrail')) {
                        const auditStore = db.createObjectStore('auditTrail', { keyPath: 'id', autoIncrement: true });
                        auditStore.createIndex('timestamp', 'timestamp', { unique: false });
                        auditStore.createIndex('eventType', 'eventType', { unique: false });
                    }

                    if (!db.objectStoreNames.contains('treatyZones')) {
                        const treatyStore = db.createObjectStore('treatyZones', { keyPath: 'zoneId' });
                        treatyStore.createIndex('name', 'name', { unique: false });
                    }
                };
            });
        }

        async loadCachedData() {
            if (!this.indexedDB) return;

            try {
                const objectStates = await this.getAllFromStore('objectStates');
                const violations = await this.getAllFromStore('violations');
                const treatyZones = await this.getAllFromStore('treatyZones');

                objectStates.forEach(obj => this.state.objects.set(obj.objectId, obj));
                this.state.violations = violations.slice(-100);
                this.state.treatyZones = treatyZones;
                this.state.lastUpdate = Date.now();
            } catch (error) {
                this.logAudit('CACHE_LOAD_ERROR', { error: error.message, timestamp: Date.now() });
            }
        }

        async getAllFromStore(storeName) {
            return new Promise((resolve, reject) => {
                const transaction = this.indexedDB.transaction([storeName], 'readonly');
                const store = transaction.objectStore(storeName);
                const request = store.getAll();

                request.onsuccess = () => resolve(request.result);
                request.onerror = () => reject(request.error);
            });
        }

        async saveToStore(storeName, data) {
            return new Promise((resolve, reject) => {
                const transaction = this.indexedDB.transaction([storeName], 'readwrite');
                const store = transaction.objectStore(storeName);
                const request = store.put(data);

                request.onsuccess = () => resolve(request.result);
                request.onerror = () => reject(request.error);
            });
        }

        // ====================================================================
        // SECTION 3: EVENT LISTENER SETUP
        // ====================================================================

        setupEventListeners() {
            window.addEventListener('resize', () => this.handleResize());
            window.addEventListener('offline', () => this.handleConnectionChange('offline'));
            window.addEventListener('online', () => this.handleConnectionChange('online'));

            document.addEventListener('keydown', (e) => this.handleKeyboardInput(e));

            if (typeof navigator !== 'undefined' && navigator.serviceWorker) {
                navigator.serviceWorker.register('/aletheion-sw.js').then(registration => {
                    this.logAudit('SERVICE_WORKER_REGISTERED', {
                        scope: registration.scope,
                        timestamp: Date.now()
                    });
                }).catch(error => {
                    this.logAudit('SERVICE_WORKER_FAILED', {
                        error: error.message,
                        timestamp: Date.now()
                    });
                });
            }
        }

        handleResize() {
            if (this.canvas) {
                this.canvas.width = window.innerWidth;
                this.canvas.height = window.innerHeight;
                this.render();
            }
        }

        handleConnectionChange(status) {
            this.state.connectionStatus = status;
            this.logAudit('CONNECTION_STATUS_CHANGE', {
                status: status,
                timestamp: Date.now()
            });

            if (status === 'online') {
                this.syncWithServer();
            }
        }

        handleKeyboardInput(e) {
            switch(e.key) {
                case 'r':
                case 'R':
                    this.toggleRealTimeView();
                    break;
                case 't':
                case 'T':
                    this.toggleTreatyOverlay();
                    break;
                case 'v':
                case 'V':
                    this.toggleViolationLog();
                    break;
                case 'e':
                case 'E':
                    this.toggleEmergencyOverrides();
                    break;
                case 'Escape':
                    this.resetView();
                    break;
            }
        }

        // ====================================================================
        // SECTION 4: DATA POLLING & SERVER SYNC
        // ====================================================================

        startDataPolling() {
            setInterval(() => {
                this.pollObjectStates();
                this.pollViolations();
                this.pollTreatyZones();
            }, this.config.refreshIntervalMs);
        }

        async pollObjectStates() {
            try {
                const response = await fetch('/api/v1/ecosafety/objects/states', {
                    method: 'GET',
                    headers: {
                        'Content-Type': 'application/json',
                        'X-Aletheion-Client': 'LyapunovDashboard/' + this.cacheVersion,
                        'X-Offline-Capable': 'true'
                    },
                    cache: 'no-cache'
                });

                if (response.ok) {
                    const data = await response.json();
                    await this.processObjectStates(data);
                    this.state.connectionStatus = 'online';
                } else if (this.config.offlineMode) {
                    this.state.connectionStatus = 'offline';
                }
            } catch (error) {
                if (this.config.offlineMode) {
                    this.state.connectionStatus = 'offline';
                } else {
                    this.logAudit('POLL_ERROR', {
                        error: error.message,
                        endpoint: '/api/v1/ecosafety/objects/states',
                        timestamp: Date.now()
                    });
                }
            }
        }

        async processObjectStates(data) {
            if (!data || !Array.isArray(data)) return;

            for (const obj of data) {
                const existing = this.state.objects.get(obj.objectId);

                if (existing) {
                    existing.v_t = obj.v_t;
                    existing.v_t_delta = obj.v_t - existing.v_t;
                    existing.risk_scalar = obj.risk_scalar;
                    existing.swarm_coverage = obj.swarm_coverage;
                    existing.agent_density = obj.agent_density;
                    existing.timestamp = obj.timestamp;
                    existing.status = this.calculateObjectStatus(obj);
                } else {
                    this.state.objects.set(obj.objectId, {
                        objectId: obj.objectId,
                        objectClass: obj.objectClass,
                        v_t: obj.v_t,
                        v_t_delta: 0,
                        risk_scalar: obj.risk_scalar,
                        swarm_coverage: obj.swarm_coverage,
                        agent_density: obj.agent_density,
                        timestamp: obj.timestamp,
                        status: this.calculateObjectStatus(obj),
                        geoZone: obj.geoZone
                    });
                }

                await this.saveToStore('objectStates', this.state.objects.get(obj.objectId));
            }

            this.state.lastUpdate = Date.now();
        }

        calculateObjectStatus(obj) {
            if (obj.v_t_delta > this.config.stabilityThreshold) {
                return 'unstable';
            } else if (obj.v_t_delta < -this.config.stabilityThreshold) {
                return 'improving';
            } else {
                return 'stable';
            }
        }

        async pollViolations() {
            try {
                const response = await fetch('/api/v1/ecosafety/violations/recent?limit=50', {
                    method: 'GET',
                    headers: {
                        'Content-Type': 'application/json',
                        'X-Aletheion-Client': 'LyapunovDashboard/' + this.cacheVersion
                    }
                });

                if (response.ok) {
                    const data = await response.json();
                    this.state.violations = data;
                    await this.clearOldViolations();
                }
            } catch (error) {
                if (!this.config.offlineMode) {
                    this.logAudit('VIOLATION_POLL_ERROR', {
                        error: error.message,
                        timestamp: Date.now()
                    });
                }
            }
        }

        async clearOldViolations() {
            const cutoff = Date.now() - (7 * 24 * 60 * 60 * 1000);
            this.state.violations = this.state.violations.filter(v => v.timestamp > cutoff);
        }

        async pollTreatyZones() {
            try {
                const response = await fetch('/api/v1/governance/treaty/zones', {
                    method: 'GET',
                    headers: {
                        'Content-Type': 'application/json',
                        'X-Aletheion-Client': 'LyapunovDashboard/' + this.cacheVersion
                    }
                });

                if (response.ok) {
                    const data = await response.json();
                    this.state.treatyZones = data;

                    for (const zone of data) {
                        await this.saveToStore('treatyZones', zone);
                    }
                }
            } catch (error) {
                if (!this.config.offlineMode) {
                    this.logAudit('TREATY_ZONE_POLL_ERROR', {
                        error: error.message,
                        timestamp: Date.now()
                    });
                }
            }
        }

        async syncWithServer() {
            const pendingViolations = await this.getAllFromStore('violations');
            const pendingAudits = await this.getAllFromStore('auditTrail');

            if (pendingViolations.length > 0 || pendingAudits.length > 0) {
                try {
                    await fetch('/api/v1/ecosafety/sync', {
                        method: 'POST',
                        headers: {
                            'Content-Type': 'application/json',
                            'X-Aletheion-Client': 'LyapunovDashboard/' + this.cacheVersion
                        },
                        body: JSON.stringify({
                            violations: pendingViolations,
                            audits: pendingAudits,
                            syncTimestamp: Date.now()
                        })
                    });
                } catch (error) {
                    this.logAudit('SYNC_ERROR', {
                        error: error.message,
                        timestamp: Date.now()
                    });
                }
            }
        }

        // ====================================================================
        // SECTION 5: CANVAS RENDERING & VISUALIZATION
        // ====================================================================

        startRenderLoop() {
            this.canvas = document.getElementById('lyapunov-canvas');
            if (!this.canvas) {
                this.canvas = document.createElement('canvas');
                this.canvas.id = 'lyapunov-canvas';
                this.canvas.style.width = '100%';
                this.canvas.style.height = '100%';
                document.body.appendChild(this.canvas);
            }

            this.canvas.width = window.innerWidth;
            this.canvas.height = window.innerHeight;
            this.ctx = this.canvas.getContext('2d');

            const render = () => {
                this.render();
                this.animationFrame = requestAnimationFrame(render);
            };

            render();
        }

        render() {
            if (!this.ctx) return;

            this.ctx.fillStyle = '#0a0e1a';
            this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);

            this.renderGrid();
            this.renderTreatyZones();
            this.renderObjects();
            this.renderViolations();
            this.renderHUD();
            this.renderLegend();
        }

        renderGrid() {
            const gridSize = 50;
            this.ctx.strokeStyle = '#1a2332';
            this.ctx.lineWidth = 1;

            for (let x = 0; x < this.canvas.width; x += gridSize) {
                this.ctx.beginPath();
                this.ctx.moveTo(x, 0);
                this.ctx.lineTo(x, this.canvas.height);
                this.ctx.stroke();
            }

            for (let y = 0; y < this.canvas.height; y += gridSize) {
                this.ctx.beginPath();
                this.ctx.moveTo(0, y);
                this.ctx.lineTo(this.canvas.width, y);
                this.ctx.stroke();
            }
        }

        renderTreatyZones() {
            if (!this.config.treatyZoneOverlay) return;

            this.state.treatyZones.forEach(zone => {
                const x = this.geoToScreen(zone.geo_polygon[0]);
                const y = this.geoToScreen(zone.geo_polygon[1]);
                const width = this.geoToScreen(zone.geo_polygon[2]) - x;
                const height = this.geoToScreen(zone.geo_polygon[3]) - y;

                this.ctx.fillStyle = zone.biotic_treaty_level >= 4 ?
                    'rgba(139, 69, 19, 0.3)' : 'rgba(34, 139, 34, 0.2)';
                this.ctx.fillRect(x, y, width, height);

                this.ctx.strokeStyle = zone.biotic_treaty_level >= 4 ?
                    '#8B4513' : '#228B22';
                this.ctx.lineWidth = 2;
                this.ctx.strokeRect(x, y, width, height);

                this.ctx.fillStyle = '#ffffff';
                this.ctx.font = '12px monospace';
                this.ctx.fillText(zone.name, x + 5, y + 15);
            });
        }

        renderObjects() {
            const objects = Array.from(this.state.objects.values());
