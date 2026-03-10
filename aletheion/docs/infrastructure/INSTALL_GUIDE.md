# Aletheion Installation Guide

* **Version:** 1.0.0
* **Target:** Phoenix, Arizona (Sonoran Desert)
* **Installer:** `deployment/installer/src/city_installer.lua`
* **Offline Capable:** Yes (≥72 hours)

## Prerequisites
1. **Hardware:** x86_64 or ARM64 server (4GB RAM min, 100GB Storage).
2. **OS:** Linux (Ubuntu 22.04+, Alpine, or NixOS).
3. **Runtime:** Lua 5.4+ (for installer), Rust Toolchain (for compilation).
4. **Network:** Initial download required; offline operation thereafter.

## Step 1: Clone Repository
```bash
git clone https://github.com/Doctor0Evil/Aletheion.git
cd aletheion
```

## Step 2: Verify Integrity
```bash
# Verify file hashes (no blacklisted algorithms)
lua deployment/installer/src/verify_hashes.lua
```

## Step 3: Configure Profile
Edit `deployment/config/phoenix_profile.json`:
```json
{
  "location": { "lat": 33.4484, "lon": -112.0740 },
  "indigenous_territory": true,
  "offline_mode": true,
  "subsystems": ["environmental", "energy", "security"]
}
```

## Step 4: Run Installer
```bash
lua deployment/installer/src/city_installer.lua
```
**Installer Actions:**
1. **Territory Check:** Verifies Indigenous land sovereignty (Akimel O'odham).
2. **Backup:** Creates system snapshot (`/var/backup/aletheion`).
3. **Deploy:** Installs subsystems in parallel (max 4 concurrent).
4. **Validate:** Runs health checks on each module.
5. **Rollback:** Auto-rolls back failed subsystems (Forward-Compatible).

## Step 5: Post-Install Validation
```bash
# Check subsystem status
lua deployment/installer/src/validate_install.lua

# View logs
cat /var/log/aletheion/install.log
```

## Step 6: Initialize Security
```bash
# Generate post-quantum keys (Lattice-based)
rust run core/security/zero_knowledge/src/keygen.rs

# Rotate keys (Daily)
rust run core/security/zero_knowledge/src/rotate_keys.rs
```

## Step 7: Citizen Interface
1. **Android:** Install `interface/citizen/android/app` via F-Droid or sideload.
2. **Web:** Serve `interface/governance/web` via local server (Caddy/Nginx).
3. **Consent:** Users must opt-in via NeuroConsent module.

## Troubleshooting
- **Offline Failure:** Check `offline_cache_ttl` in config (default 72h).
- **Sovereignty Error:** Verify Indigenous territory coordinates.
- **Hash Mismatch:** Re-clone repository; do not modify core files.

## Uninstallation
```bash
# Purge all data (Neurorights Compliant)
lua deployment/installer/src/purge_all_data.lua
```

---
*Installation complete. Aletheion is now operational.*
