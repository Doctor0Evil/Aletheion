# Aletheion API Reference
**File:** 97/100  
**Version:** 1.0.0  
**Compliance:** Shared Data Contracts, ALN-Blockchain Interfaces  

## 1. Identity API (aletheion-core/identity)
### `verify_birth_sign(sign: BirthSignId) -> bool`
- **Description:** Validates citizen identity on ALN-Blockchain.
- **Compliance:** DID-Bound, Privacy-Preserving.
- **Endpoint:** `/identity/v1/verify`

### `get_consent_token(subject: BirthSignId, scope: string) -> FpicToken`
- **Description:** Retrieves consent token for specific data scope.
- **Compliance:** FPIC, Neurorights.
- **Endpoint:** `/identity/v1/consent`

## 2. Health API (aletheion-health/biosignal)
### `ingest_biosignal(packet: BiosignalPacket) -> Result`
- **Description:** Privacy-preserving biosignal aggregation.
- **Compliance:** Homomorphic Encryption, HIPAA-Equivalent.
- **Endpoint:** `/health/v1/signal`

### `get_heat_stress_alert(location: GeoCoord) -> Alert`
- **Description:** Returns heat stress warnings for location.
- **Compliance:** Phoenix Heat Protocol.
- **Endpoint:** `/health/v1/alerts/heat`

## 3. Safety API (aletheion-safety/emergency)
### `dispatch_emergency(incident: EmergencyIncident) -> UnitId`
- **Description:** Coordinates multi-agency response.
- **Compliance:** De-escalation Priority, ALN Audit.
- **Endpoint:** `/safety/v1/dispatch`

### `verify_airspace_fpic(location: GeoCoord) -> bool`
- **Description:** Checks Indigenous airspace rights for drones.
- **Compliance:** FPIC, Sovereignty.
- **Endpoint:** `/safety/v1/airspace`

## 4. Infrastructure API (aletheion-infra/mesh)
### `get_mesh_status() -> NetworkMetrics`
- **Description:** Returns offline capacity and resilience score.
- **Compliance:** 72-Hour Offline Minimum.
- **Endpoint:** `/infra/v1/mesh/status`

### `forward_packet(packet: MeshPacket) -> Result`
- **Description:** Store-and-forward packet transmission.
- **Compliance:** Haboob Resilience.
- **Endpoint:** `/infra/v1/mesh/forward`

## 5. Data Contracts
- **BirthSignId:** `aln:identity:birthsign:<hash>`
- **FpicToken:** `aln:treaty:fpic:<community>:<scope>`
- **GeoCoord:** `geo:az:phx:<lat>:<lon>`
- **AlnHash:** `aln:crypto:hash:<algorithm>:<value>`

## 6. Error Codes
- `ERR_FPIC_MISSING`: Consent not verified.
- `ERR_HEAT_PAUSE`: Operation paused due to extreme heat.
- `ERR_OFFLINE_LIMIT`: Offline buffer exceeded.
- `ERR_ROLLBACK_DENIED`: Downgrade attempt blocked.
