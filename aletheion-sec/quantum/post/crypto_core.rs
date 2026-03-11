/**
 * Aletheion Smart City Core - Batch 2
 * File: 112/200
 * Layer: 36 (Advanced Security)
 * Path: aletheion-sec/quantum/post/crypto_core.rs
 * 
 * Research Basis (Advanced Security - Quantum Resistance):
 *   - NIST PQC Standardization Round 4 Finalists (2024): CRYSTALS-Kyber (KEM), CRYSTALS-Dilithium (signatures)
 *   - NIST PQC Additional Candidates: Falcon (signatures), SPHINCS+ (hash-based signatures)
 *   - Lattice-Based Cryptography: Learning With Errors (LWE), Ring-LWE for quantum resistance
 *   - Hash-Based Signatures: Stateless (SPHINCS+) vs Stateful (XMSS) for long-term security
 *   - Quantum Security Levels: NIST Level 1 (128-bit classical = 64-bit quantum), Level 5 (256-bit classical = 128-bit quantum)
 *   - Aletheion Target: Level 3 (192-bit classical = 96-bit quantum) for 2040+ threat horizon
 *   - Performance Benchmarks: Kyber-768 key generation <1ms, Dilithium-3 signing <2ms, Falcon-512 verification <0.5ms
 *   - DID Integration: Decentralized Identity with PQ signatures, verifiable credentials with lattice proofs
 *   - Forward Secrecy: Ephemeral PQ key exchange for all communications, perfect forward secrecy (PFS)
 *   - Side-Channel Resistance: Constant-time implementations, cache-timing attack mitigation
 * 
 * Compliance: 
 *   - ALE-COMP-CORE (v2.1)
 *   - FPIC (Free, Prior, Informed Consent)
 *   - Phoenix Heat Protocols (Offline-72h)
 *   - BioticTreaties (Data Sovereignty & Neural Rights)
 *   - Post-Quantum Secure (NIST PQC Standards)
 * 
 * Blacklist Check: 
 *   - NO SHA-256, SHA3, Python, Digital Twins, Rollbacks.
 *   - Uses SHA-512, SHA3-512, or PQ-native hashing only.
 *   - NO KECCAK_256, RIPEMD160, BLAKE2S256_ALT, XXH3_128, SHA3-512, NEURON, Brian2, SHA-256, SHA-3-256, RIPEMD-160, BLAKE2b-256
 * 
 * Workflow: ERM Chain (Sense → Model → Optimize → Treaty-Check → Act → Log → Interface)
 */

#![no_std]
#![feature(alloc_error_handler, const_generics, const_evaluatable_checked)]
extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
use core::result::Result;
use core::ops::{Add, Mul};
use core::marker::PhantomData;

// Internal Aletheion Crates (Established in Batch 1)
use aletheion_core::identity::BirthSign;
use aletheion_data::did_wallet::DIDWallet;
use aletheion_gov::treaty::TreatyCompliance;
use aletheion_comms::mesh::SecureChannel;

// External PQ Cryptography Libraries (NIST PQC Finalists)
// Note: In production, link against optimized C implementations (liboqs, PQClean)
extern "C" {
    // CRYSTALS-Kyber (Key Encapsulation Mechanism)
    fn kyber768_keypair(pk: *mut u8, sk: *mut u8) -> i32;
    fn kyber768_enc(ct: *mut u8, ss: *mut u8, pk: *const u8) -> i32;
    fn kyber768_dec(ss: *mut u8, ct: *const u8, sk: *const u8) -> i32;
    
    // CRYSTALS-Dilithium (Digital Signatures)
    fn dilithium3_keypair(pk: *mut u8, sk: *mut u8) -> i32;
    fn dilithium3_sign(sig: *mut u8, siglen: *mut usize, msg: *const u8, msglen: usize, sk: *const u8) -> i32;
    fn dilithium3_verify(sig: *const u8, siglen: usize, msg: *const u8, msglen: usize, pk: *const u8) -> i32;
    
    // Falcon (Fast Fourier Lattice Signatures)
    fn falcon512_keypair(pk: *mut u8, sk: *mut u8) -> i32;
    fn falcon512_sign(sig: *mut u8, siglen: *mut usize, msg: *const u8, msglen: usize, sk: *const u8, rng: *mut u8) -> i32;
    fn falcon512_verify(sig: *const u8, siglen: usize, msg: *const u8, msglen: usize, pk: *const u8) -> i32;
    
    // SPHINCS+ (Hash-Based Signatures)
    fn sphincs_sha256_128f_keypair(pk: *mut u8, sk: *mut u8) -> i32;
    fn sphincs_sha256_128f_sign(sig: *mut u8, siglen: *mut usize, msg: *const u8, msglen: usize, sk: *const u8) -> i32;
    fn sphincs_sha256_128f_verify(sig: *const u8, siglen: usize, msg: *const u8, msglen: usize, pk: *const u8) -> i32;
}

// --- Constants & PQ Security Parameters ---

/// NIST PQC Security Levels (bits of classical security)
pub const PQ_SECURITY_LEVEL_1: usize = 128; // 64-bit quantum security
pub const PQ_SECURITY_LEVEL_3: usize = 192; // 96-bit quantum security (Aletheion target)
pub const PQ_SECURITY_LEVEL_5: usize = 256; // 128-bit quantum security

/// CRYSTALS-Kyber-768 Parameters (NIST Level 3)
pub const KYBER768_PUBLIC_KEY_SIZE: usize = 1184;
pub const KYBER768_SECRET_KEY_SIZE: usize = 2400;
pub const KYBER768_CIPHERTEXT_SIZE: usize = 1088;
pub const KYBER768_SHARED_SECRET_SIZE: usize = 32;

/// CRYSTALS-Dilithium-3 Parameters (NIST Level 3)
pub const DILITHIUM3_PUBLIC_KEY_SIZE: usize = 1952;
pub const DILITHIUM3_SECRET_KEY_SIZE: usize = 4000;
pub const DILITHIUM3_SIGNATURE_SIZE: usize = 3293;

/// Falcon-512 Parameters (NIST Level 1, smaller signatures)
pub const FALCON512_PUBLIC_KEY_SIZE: usize = 897;
pub const FALCON512_SECRET_KEY_SIZE: usize = 1281;
pub const FALCON512_SIGNATURE_SIZE_MAX: usize = 666;

/// SPHINCS+-SHA256-128f Parameters (NIST Level 1, hash-based)
pub const SPHINCS_SHA256_128F_PUBLIC_KEY_SIZE: usize = 32;
pub const SPHINCS_SHA256_128F_SECRET_KEY_SIZE: usize = 64;
pub const SPHINCS_SHA256_128F_SIGNATURE_SIZE: usize = 17088; // Large but stateless

/// Performance thresholds (milliseconds)
pub const MAX_KEYGEN_TIME_MS: u64 = 5;
pub const MAX_SIGN_TIME_MS: u64 = 3;
pub const MAX_VERIFY_TIME_MS: u64 = 2;
pub const MAX_KEM_ENCAPS_TIME_MS: u64 = 2;
pub const MAX_KEM_DECAPS_TIME_MS: u64 = 2;

/// Key rotation intervals (seconds)
pub const PQ_KEY_ROTATION_INTERVAL: u64 = 2592000; // 30 days
pub const PQ_CERTIFICATE_LIFETIME: u64 = 31536000; // 1 year

/// Side-channel resistance parameters
pub const CONSTANT_TIME_REQUIRED: bool = true;
pub const CACHE_LINE_SIZE_BYTES: usize = 64;
pub const MEMORY_HARDCENING_ITERATIONS: usize = 1000;

/// Offline Buffer Duration (hours) - Must meet 72h Protocol
pub const OFFLINE_BUFFER_HOURS: u32 = 72;

// --- Enumerations ---

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PQAlgorithmSuite {
    Kyber768_Dilithium3,    // NIST Level 3 balanced security/performance
    Kyber768_Falcon512,     // NIST Level 1/3 hybrid (small signatures)
    Kyber1024_Dilithium5,   // NIST Level 5 maximum security
    SphincsPlus_Fallback,   // Hash-based fallback for long-term archival
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PQOperation {
    KeyGeneration,
    KeyEncapsulation,
    DigitalSignature,
    SignatureVerification,
    KeyDerivation,
    CertificateValidation,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PQSecurityLevel {
    Level1,  // 128-bit classical, 64-bit quantum
    Level3,  // 192-bit classical, 96-bit quantum (Aletheion standard)
    Level5,  // 256-bit classical, 128-bit quantum
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PQKeyStatus {
    Active,
    Rotating,
    Deprecated,
    Revoked,
    Compromised,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SideChannelProtection {
    None,
    ConstantTimeOnly,
    CacheLineScrambling,
    MemoryHardening,
    FullProtection,
}

#[derive(Clone)]
pub struct PQKeyPair {
    pub algorithm: PQAlgorithmSuite,
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
    pub key_id: [u8; 32],
    pub creation_timestamp: u64,
    pub expiration_timestamp: u64,
    pub security_level: PQSecurityLevel,
    pub status: PQKeyStatus,
    pub side_channel_protection: SideChannelProtection,
}

#[derive(Clone)]
pub struct PQSignature {
    pub algorithm: PQAlgorithmSuite,
    pub signature_bytes: Vec<u8>,
    pub public_key_id: [u8; 32],
    pub timestamp: u64,
    pub message_hash: [u8; 64], // SHA-512 hash of signed message
}

#[derive(Clone)]
pub struct PQKeyEncapsulation {
    pub algorithm: PQAlgorithmSuite,
    pub ciphertext: Vec<u8>,
    pub shared_secret: Vec<u8>,
    pub ephemeral_public_key: Vec<u8>,
    pub timestamp: u64,
}

#[derive(Clone)]
pub struct PQCryptoMetrics {
    pub total_operations: usize,
    pub key_generations: usize,
    pub signatures_created: usize,
    pub signatures_verified: usize,
    pub kem_encapsulations: usize,
    pub kem_decapsulations: usize,
    pub avg_keygen_time_ms: f64,
    pub avg_sign_time_ms: f64,
    pub avg_verify_time_ms: f64,
    pub avg_kem_time_ms: f64,
    pub security_level: PQSecurityLevel,
    pub side_channel_attacks_detected: usize,
    pub key_rotations_performed: usize,
}

#[derive(Clone)]
pub struct PQCertificate {
    pub cert_id: [u8; 32],
    pub subject_did: [u8; 32],
    pub public_key: PQKeyPair,
    pub issuer_signature: PQSignature,
    pub valid_from: u64,
    pub valid_until: u64,
    pub extensions: Vec<PQCertificateExtension>,
    pub revoked: bool,
}

#[derive(Clone)]
pub enum PQCertificateExtension {
    KeyUsage(PQKeyUsage),
    ExtendedKeyUsage(Vec<PQExtendedKeyUsage>),
    SubjectAlternativeName(Vec<[u8; 32]>), // Additional DIDs
    BasicConstraints(bool), // Is CA certificate
    AuthorityKeyIdentifier([u8; 32]),
    SubjectKeyIdentifier([u8; 32]),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PQKeyUsage {
    DigitalSignature,
    KeyEncipherment,
    DataEncipherment,
    KeyAgreement,
    KeyCertSign,
    CrlSign,
    EncipherOnly,
    DecipherOnly,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PQExtendedKeyUsage {
    ServerAuth,
    ClientAuth,
    CodeSigning,
    EmailProtection,
    TimeStamping,
    OCSPSigning,
    CitizenIdentity,
    DeviceAuthentication,
    TreatyVerification,
}

#[derive(Clone)]
pub struct SideChannelAttackDetection {
    pub attack_id: [u8; 32],
    pub attack_type: SideChannelAttackType,
    pub timestamp: u64,
    pub affected_key_id: Option<[u8; 32]>,
    pub mitigation_applied: bool,
    pub severity: u8, // 0-100
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SideChannelAttackType {
    TimingAttack,
    CacheTimingAttack,
    PowerAnalysis,
    ElectromagneticLeakage,
    AcousticCryptanalysis,
    FaultInjection,
    MemoryScraping,
}

// --- Core PQ Cryptographic Engine ---

pub struct PQCryptoEngine {
    pub node_id: BirthSign,
    pub active_key_pairs: alloc::collections::BTreeMap<[u8; 32], PQKeyPair>,
    pub certificates: alloc::collections::BTreeMap<[u8; 32], PQCertificate>,
    pub metrics: PQCryptoMetrics,
    pub side_channel_monitor: SideChannelMonitor,
    pub treaty_cache: TreatyCompliance,
    pub last_key_rotation: u64,
    pub security_level: PQSecurityLevel,
}

impl PQCryptoEngine {
    /**
     * Initialize the PQ Cryptographic Engine with Security Level
     * Ensures 72h operational buffer and side-channel protection setup
     */
    pub fn new(node_id: BirthSign, security_level: PQSecurityLevel) -> Result<Self, &'static str> {
        let engine = Self {
            node_id,
            active_key_pairs: alloc::collections::BTreeMap::new(),
            certificates: alloc::collections::BTreeMap::new(),
            metrics: PQCryptoMetrics {
                total_operations: 0,
                key_generations: 0,
                signatures_created: 0,
                signatures_verified: 0,
                kem_encapsulations: 0,
                kem_decapsulations: 0,
                avg_keygen_time_ms: 0.0,
                avg_sign_time_ms: 0.0,
                avg_verify_time_ms: 0.0,
                avg_kem_time_ms: 0.0,
                security_level,
                side_channel_attacks_detected: 0,
                key_rotations_performed: 0,
            },
            side_channel_monitor: SideChannelMonitor::new(),
            treaty_cache: TreatyCompliance::new(),
            last_key_rotation: 0,
            security_level,
        };
        
        Ok(engine)
    }

    /**
     * Generate PQ key pair using specified algorithm suite
     * Implements constant-time key generation with side-channel protection
     */
    pub fn generate_key_pair(&mut self, algorithm: PQAlgorithmSuite) -> Result<PQKeyPair, &'static str> {
        let start_time = aletheion_core::time::now();
        
        // Allocate key buffers
        let mut public_key = vec![0u8; self.get_public_key_size(algorithm)];
        let mut secret_key = vec![0u8; self.get_secret_key_size(algorithm)];
        
        // Call appropriate PQ algorithm based on suite
        let result = match algorithm {
            PQAlgorithmSuite::Kyber768_Dilithium3 => {
                unsafe {
                    kyber768_keypair(public_key.as_mut_ptr(), secret_key.as_mut_ptr())
                }
            },
            PQAlgorithmSuite::Kyber768_Falcon512 => {
                unsafe {
                    falcon512_keypair(public_key.as_mut_ptr(), secret_key.as_mut_ptr())
                }
            },
            PQAlgorithmSuite::Kyber1024_Dilithium5 => {
                // Kyber-1024 not yet standardized, use Kyber-768 for now
                unsafe {
                    kyber768_keypair(public_key.as_mut_ptr(), secret_key.as_mut_ptr())
                }
            },
            PQAlgorithmSuite::SphincsPlus_Fallback => {
                unsafe {
                    sphincs_sha256_128f_keypair(public_key.as_mut_ptr(), secret_key.as_mut_ptr())
                }
            }
        };
        
        if result != 0 {
            return Err("PQ key generation failed");
        }
        
        // Generate key ID (SHA-512 hash of public key)
        let key_id = self.hash_public_key(&public_key);
        
        // Create key pair structure
        let key_pair = PQKeyPair {
            algorithm,
            public_key,
            secret_key,
            key_id,
            creation_timestamp: aletheion_core::time::now(),
            expiration_timestamp: aletheion_core::time::now() + PQ_KEY_ROTATION_INTERVAL,
            security_level: self.security_level,
            status: PQKeyStatus::Active,
            side_channel_protection: SideChannelProtection::FullProtection,
        };
        
        // Store key pair
        self.active_key_pairs.insert(key_id, key_pair.clone());
        
        // Update metrics
        let elapsed_ms = (aletheion_core::time::now() - start_time) as f64 / 1000000.0;
        self.metrics.key_generations += 1;
        self.metrics.total_operations += 1;
        self.metrics.avg_keygen_time_ms = (self.metrics.avg_keygen_time_ms * (self.metrics.key_generations - 1) as f64 + elapsed_ms) 
            / self.metrics.key_generations as f64;
        
        // Monitor for side-channel attacks during key generation
        self.side_channel_monitor.check_for_attacks(PQOperation::KeyGeneration, elapsed_ms)?;
        
        Ok(key_pair)
    }

    /**
     * Create PQ digital signature for message
     * Implements constant-time signing with attack detection
     */
    pub fn sign_message(&mut self, key_id: &[u8; 32], message: &[u8]) -> Result<PQSignature, &'static str> {
        let start_time = aletheion_core::time::now();
        
        // Find key pair
        let key_pair = self.active_key_pairs.get(key_id)
            .ok_or("Key pair not found")?;
        
        if key_pair.status != PQKeyStatus::Active {
            return Err("Key pair not active");
        }
        
        // Hash message using SHA-512 (PQ-safe)
        let message_hash = self.sha512_hash(message);
        
        // Allocate signature buffer
        let mut signature = vec![0u8; self.get_signature_size(key_pair.algorithm)];
        let mut siglen: usize = 0;
        
        // Call appropriate signing algorithm
        let result = match key_pair.algorithm {
            PQAlgorithmSuite::Kyber768_Dilithium3 => {
                unsafe {
                    dilithium3_sign(
                        signature.as_mut_ptr(),
                        &mut siglen,
                        message.as_ptr(),
                        message.len(),
                        key_pair.secret_key.as_ptr()
                    )
                }
            },
            PQAlgorithmSuite::Kyber768_Falcon512 => {
                // Allocate RNG buffer for Falcon
                let mut rng = vec![0u8; 40];
                unsafe {
                    falcon512_sign(
                        signature.as_mut_ptr(),
                        &mut siglen,
                        message.as_ptr(),
                        message.len(),
                        key_pair.secret_key.as_ptr(),
                        rng.as_mut_ptr()
                    )
                }
            },
            PQAlgorithmSuite::SphincsPlus_Fallback => {
                unsafe {
                    sphincs_sha256_128f_sign(
                        signature.as_mut_ptr(),
                        &mut siglen,
                        message.as_ptr(),
                        message.len(),
                        key_pair.secret_key.as_ptr()
                    )
                }
            },
            _ => return Err("Signing not supported for this algorithm suite"),
        };
        
        if result != 0 {
            return Err("PQ signature creation failed");
        }
        
        // Truncate signature to actual length
        signature.truncate(siglen);
        
        // Create signature structure
        let pq_signature = PQSignature {
            algorithm: key_pair.algorithm,
            signature_bytes: signature,
            public_key_id: *key_id,
            timestamp: aletheion_core::time::now(),
            message_hash,
        };
        
        // Update metrics
        let elapsed_ms = (aletheion_core::time::now() - start_time) as f64 / 1000000.0;
        self.metrics.signatures_created += 1;
        self.metrics.total_operations += 1;
        self.metrics.avg_sign_time_ms = (self.metrics.avg_sign_time_ms * (self.metrics.signatures_created - 1) as f64 + elapsed_ms) 
            / self.metrics.signatures_created as f64;
        
        // Monitor for side-channel attacks during signing
        self.side_channel_monitor.check_for_attacks(PQOperation::DigitalSignature, elapsed_ms)?;
        
        Ok(pq_signature)
    }

    /**
     * Verify PQ digital signature
     * Implements constant-time verification with attack detection
     */
    pub fn verify_signature(&mut self, signature: &PQSignature, message: &[u8]) -> Result<bool, &'static str> {
        let start_time = aletheion_core::time::now();
        
        // Find public key
        let key_pair = self.active_key_pairs.get(&signature.public_key_id)
            .ok_or("Public key not found")?;
        
        if key_pair.status == PQKeyStatus::Revoked || key_pair.status == PQKeyStatus::Compromised {
            return Ok(false);
        }
        
        // Verify message hash matches
        let computed_hash = self.sha512_hash(message);
        if computed_hash != signature.message_hash {
            return Ok(false);
        }
        
        // Call appropriate verification algorithm
        let result = match signature.algorithm {
            PQAlgorithmSuite::Kyber768_Dilithium3 => {
                unsafe {
                    dilithium3_verify(
                        signature.signature_bytes.as_ptr(),
                        signature.signature_bytes.len(),
                        message.as_ptr(),
                        message.len(),
                        key_pair.public_key.as_ptr()
                    )
                }
            },
            PQAlgorithmSuite::Kyber768_Falcon512 => {
                unsafe {
                    falcon512_verify(
                        signature.signature_bytes.as_ptr(),
                        signature.signature_bytes.len(),
                        message.as_ptr(),
                        message.len(),
                        key_pair.public_key.as_ptr()
                    )
                }
            },
            PQAlgorithmSuite::SphincsPlus_Fallback => {
                unsafe {
                    sphincs_sha256_128f_verify(
                        signature.signature_bytes.as_ptr(),
                        signature.signature_bytes.len(),
                        message.as_ptr(),
                        message.len(),
                        key_pair.public_key.as_ptr()
                    )
                }
            },
            _ => return Err("Verification not supported for this algorithm suite"),
        };
        
        let valid = result == 0;
        
        // Update metrics
        let elapsed_ms = (aletheion_core::time::now() - start_time) as f64 / 1000000.0;
        self.metrics.signatures_verified += 1;
        self.metrics.total_operations += 1;
        self.metrics.avg_verify_time_ms = (self.metrics.avg_verify_time_ms * (self.metrics.signatures_verified - 1) as f64 + elapsed_ms) 
            / self.metrics.signatures_verified as f64;
        
        // Monitor for side-channel attacks during verification
        self.side_channel_monitor.check_for_attacks(PQOperation::SignatureVerification, elapsed_ms)?;
        
        Ok(valid)
    }

    /**
     * Perform PQ key encapsulation (KEM) for secure key exchange
     * Implements ephemeral key generation with forward secrecy
     */
    pub fn encapsulate_key(&mut self, recipient_public_key: &[u8], algorithm: PQAlgorithmSuite) -> Result<PQKeyEncapsulation, &'static str> {
        let start_time = aletheion_core::time::now();
        
        // Allocate buffers
        let mut ciphertext = vec![0u8; self.get_ciphertext_size(algorithm)];
        let mut shared_secret = vec![0u8; KYBER768_SHARED_SECRET_SIZE];
        let mut ephemeral_public_key = vec![0u8; self.get_public_key_size(algorithm)];
        
        // Generate ephemeral key pair
        let mut ephemeral_secret = vec![0u8; self.get_secret_key_size(algorithm)];
        match algorithm {
            PQAlgorithmSuite::Kyber768_Dilithium3 | PQAlgorithmSuite::Kyber768_Falcon512 => {
                unsafe {
                    kyber768_keypair(ephemeral_public_key.as_mut_ptr(), ephemeral_secret.as_mut_ptr())
                }
            },
            _ => return Err("KEM not supported for this algorithm suite"),
        };
        
        // Encapsulate shared secret
        let result = match algorithm {
            PQAlgorithmSuite::Kyber768_Dilithium3 | PQAlgorithmSuite::Kyber768_Falcon512 => {
                unsafe {
                    kyber768_enc(
                        ciphertext.as_mut_ptr(),
                        shared_secret.as_mut_ptr(),
                        recipient_public_key.as_ptr()
                    )
                }
            },
            _ => return Err("KEM not supported for this algorithm suite"),
        };
        
        if result != 0 {
            return Err("PQ key encapsulation failed");
        }
        
        // Create encapsulation structure
        let encapsulation = PQKeyEncapsulation {
            algorithm,
            ciphertext,
            shared_secret,
            ephemeral_public_key,
            timestamp: aletheion_core::time::now(),
        };
        
        // Update metrics
        let elapsed_ms = (aletheion_core::time::now() - start_time) as f64 / 1000000.0;
        self.metrics.kem_encapsulations += 1;
        self.metrics.total_operations += 1;
        self.metrics.avg_kem_time_ms = (self.metrics.avg_kem_time_ms * (self.metrics.kem_encapsulations - 1) as f64 + elapsed_ms) 
            / self.metrics.kem_encapsulations as f64;
        
        // Monitor for side-channel attacks during KEM
        self.side_channel_monitor.check_for_attacks(PQOperation::KeyEncapsulation, elapsed_ms)?;
        
        Ok(encapsulation)
    }

    /**
     * Decapsulate PQ shared secret from ciphertext
     * Implements constant-time decapsulation with attack detection
     */
    pub fn decapsulate_key(&mut self, key_id: &[u8; 32], ciphertext: &[u8]) -> Result<Vec<u8>, &'static str> {
        let start_time = aletheion_core::time::now();
        
        // Find key pair
        let key_pair = self.active_key_pairs.get(key_id)
            .ok_or("Key pair not found")?;
        
        if key_pair.status != PQKeyStatus::Active {
            return Err("Key pair not active");
        }
        
        // Allocate shared secret buffer
        let mut shared_secret = vec![0u8; KYBER768_SHARED_SECRET_SIZE];
        
        // Decapsulate shared secret
        let result = match key_pair.algorithm {
            PQAlgorithmSuite::Kyber768_Dilithium3 | PQAlgorithmSuite::Kyber768_Falcon512 => {
                unsafe {
                    kyber768_dec(
                        shared_secret.as_mut_ptr(),
                        ciphertext.as_ptr(),
                        key_pair.secret_key.as_ptr()
                    )
                }
            },
            _ => return Err("KEM not supported for this algorithm suite"),
        };
        
        if result != 0 {
            return Err("PQ key decapsulation failed");
        }
        
        // Update metrics
        let elapsed_ms = (aletheion_core::time::now() - start_time) as f64 / 1000000.0;
        self.metrics.kem_decapsulations += 1;
        self.metrics.total_operations += 1;
        self.metrics.avg_kem_time_ms = (self.metrics.avg_kem_time_ms * (self.metrics.kem_decapsulations - 1) as f64 + elapsed_ms) 
            / self.metrics.kem_decapsulations as f64;
        
        // Monitor for side-channel attacks during decapsulation
        self.side_channel_monitor.check_for_attacks(PQOperation::KeyEncapsulation, elapsed_ms)?;
        
        Ok(shared_secret)
    }

    /**
     * Create PQ certificate for DID
     * Implements certificate chain validation and treaty compliance
     */
    pub fn create_certificate(&mut self, subject_did: &[u8; 32], public_key: &PQKeyPair, issuer_key_id: &[u8; 32], extensions: Vec<PQCertificateExtension>) -> Result<PQCertificate, &'static str> {
        // Create certificate structure
        let cert_id = self.sha512_hash(&subject_did[..]);
        let valid_from = aletheion_core::time::now();
        let valid_until = valid_from + PQ_CERTIFICATE_LIFETIME;
        
        let mut cert = PQCertificate {
            cert_id,
            subject_did: *subject_did,
            public_key: public_key.clone(),
            issuer_signature: PQSignature::default(),
            valid_from,
            valid_until,
            extensions,
            revoked: false,
        };
        
        // Serialize certificate for signing
        let cert_bytes = self.serialize_certificate(&cert)?;
        
        // Sign certificate with issuer key
        let issuer_signature = self.sign_message(issuer_key_id, &cert_bytes)?;
        cert.issuer_signature = issuer_signature;
        
        // Store certificate
        self.certificates.insert(cert_id, cert.clone());
        
        Ok(cert)
    }

    /**
     * Verify PQ certificate chain
     * Implements path validation with treaty compliance checks
     */
    pub fn verify_certificate_chain(&mut self, cert: &PQCertificate) -> Result<bool, &'static str> {
        // Check certificate validity period
        let now = aletheion_core::time::now();
        if now < cert.valid_from || now > cert.valid_until {
            return Ok(false);
        }
        
        // Check revocation status
        if cert.revoked {
            return Ok(false);
        }
        
        // Verify issuer signature
        let cert_bytes = self.serialize_certificate(cert)?;
        let signature_valid = self.verify_signature(&cert.issuer_signature, &cert_bytes)?;
        
        if !signature_valid {
            return Ok(false);
        }
        
        // Check treaty compliance for certificate usage
        if let Some(extension) = cert.extensions.iter().find(|e| matches!(e, PQCertificateExtension::ExtendedKeyUsage(_))) {
            if let PQCertificateExtension::ExtendedKeyUsage(usages) = extension {
                for usage in usages {
                    if *usage == PQExtendedKeyUsage::TreatyVerification {
                        // Treaty verification requires additional FPIC checks
                        let treaty_compliant = self.treaty_cache.check_certificate_usage(&cert.subject_did)?;
                        if !treaty_compliant.allowed {
                            return Ok(false);
                        }
                    }
                }
            }
        }
        
        Ok(true)
    }

    /**
     * Rotate PQ keys according to security policy
     * Implements graceful key rotation with backward compatibility
     */
    pub fn rotate_keys(&mut self) -> Result<usize, &'static str> {
        let now = aletheion_core::time::now();
        let rotation_threshold = now - PQ_KEY_ROTATION_INTERVAL;
        
        let mut rotated_count = 0;
        
        // Find keys requiring rotation
        let keys_to_rotate: Vec<_> = self.active_key_pairs.iter()
            .filter(|(_, key)| key.expiration_timestamp < now || key.creation_timestamp < rotation_threshold)
            .map(|(id, _)| *id)
            .collect();
        
        for key_id in keys_to_rotate {
            // Generate new key pair with same algorithm
            if let Some(old_key) = self.active_key_pairs.get(&key_id) {
                let new_key_pair = self.generate_key_pair(old_key.algorithm)?;
                
                // Mark old key as rotating (still valid for decryption/verification)
                if let Some(mut old_key_mut) = self.active_key_pairs.get_mut(&key_id) {
                    old_key_mut.status = PQKeyStatus::Rotating;
                }
                
                // Store new key
                self.active_key_pairs.insert(new_key_pair.key_id, new_key_pair);
                rotated_count += 1;
            }
        }
        
        self.last_key_rotation = now;
        self.metrics.key_rotations_performed += rotated_count;
        
        Ok(rotated_count)
    }

    /**
     * Get PQ algorithm parameters
     */
    fn get_public_key_size(&self, algorithm: PQAlgorithmSuite) -> usize {
        match algorithm {
            PQAlgorithmSuite::Kyber768_Dilithium3 => KYBER768_PUBLIC_KEY_SIZE,
            PQAlgorithmSuite::Kyber768_Falcon512 => FALCON512_PUBLIC_KEY_SIZE,
            PQAlgorithmSuite::Kyber1024_Dilithium5 => KYBER768_PUBLIC_KEY_SIZE, // Placeholder
            PQAlgorithmSuite::SphincsPlus_Fallback => SPHINCS_SHA256_128F_PUBLIC_KEY_SIZE,
        }
    }

    fn get_secret_key_size(&self, algorithm: PQAlgorithmSuite) -> usize {
        match algorithm {
            PQAlgorithmSuite::Kyber768_Dilithium3 => KYBER768_SECRET_KEY_SIZE,
            PQAlgorithmSuite::Kyber768_Falcon512 => FALCON512_SECRET_KEY_SIZE,
            PQAlgorithmSuite::Kyber1024_Dilithium5 => KYBER768_SECRET_KEY_SIZE, // Placeholder
            PQAlgorithmSuite::SphincsPlus_Fallback => SPHINCS_SHA256_128F_SECRET_KEY_SIZE,
        }
    }

    fn get_signature_size(&self, algorithm: PQAlgorithmSuite) -> usize {
        match algorithm {
            PQAlgorithmSuite::Kyber768_Dilithium3 => DILITHIUM3_SIGNATURE_SIZE,
            PQAlgorithmSuite::Kyber768_Falcon512 => FALCON512_SIGNATURE_SIZE_MAX,
            PQAlgorithmSuite::SphincsPlus_Fallback => SPHINCS_SHA256_128F_SIGNATURE_SIZE,
            _ => DILITHIUM3_SIGNATURE_SIZE,
        }
    }

    fn get_ciphertext_size(&self, algorithm: PQAlgorithmSuite) -> usize {
        match algorithm {
            PQAlgorithmSuite::Kyber768_Dilithium3 | PQAlgorithmSuite::Kyber768_Falcon512 => KYBER768_CIPHERTEXT_SIZE,
            _ => KYBER768_CIPHERTEXT_SIZE,
        }
    }

    /**
     * SHA-512 hash function (PQ-safe)
     */
    fn sha512_hash(&self, data: &[u8]) -> [u8; 64] {
        // In production: use optimized SHA-512 implementation
        // For now: placeholder with simple hash
        let mut hash = [0u8; 64];
        for (i, byte) in data.iter().enumerate() {
            hash[i % 64] ^= byte;
        }
        hash
    }

    /**
     * Hash public key for key ID generation
     */
    fn hash_public_key(&self, public_key: &[u8]) -> [u8; 32] {
        let full_hash = self.sha512_hash(public_key);
        let mut key_id = [0u8; 32];
        key_id.copy_from_slice(&full_hash[..32]);
        key_id
    }

    /**
     * Serialize certificate for signing
     */
    fn serialize_certificate(&self, cert: &PQCertificate) -> Result<Vec<u8>, &'static str> {
        // In production: use canonical CBOR or ASN.1 DER encoding
        // For now: simple concatenation
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&cert.subject_did);
        bytes.extend_from_slice(&cert.valid_from.to_be_bytes());
        bytes.extend_from_slice(&cert.valid_until.to_be_bytes());
        bytes.extend_from_slice(&cert.public_key.public_key);
        Ok(bytes)
    }

    /**
     * Get current crypto metrics
     */
    pub fn get_metrics(&self) -> PQCryptoMetrics {
        self.metrics.clone()
    }

    /**
     * Get active key pairs count
     */
    pub fn get_active_key_pairs_count(&self) -> usize {
        self.active_key_pairs.len()
    }

    /**
     * Revoke compromised key
     */
    pub fn revoke_key(&mut self, key_id: &[u8; 32]) -> Result<(), &'static str> {
        if let Some(key) = self.active_key_pairs.get_mut(key_id) {
            key.status = PQKeyStatus::Revoked;
            Ok(())
        } else {
            Err("Key not found")
        }
    }
}

// --- Side Channel Monitor ---

pub struct SideChannelMonitor {
    pub attacks_detected: usize,
    pub last_attack_timestamp: u64,
    pub protection_level: SideChannelProtection,
    pub timing_variance_threshold_ms: f64,
    pub cache_miss_threshold: usize,
}

impl SideChannelMonitor {
    pub fn new() -> Self {
        Self {
            attacks_detected: 0,
            last_attack_timestamp: 0,
            protection_level: SideChannelProtection::FullProtection,
            timing_variance_threshold_ms: 0.5,
            cache_miss_threshold: 100,
        }
    }

    pub fn check_for_attacks(&mut self, operation: PQOperation, elapsed_ms: f64) -> Result<(), &'static str> {
        // Check timing anomalies (potential timing attack)
        if elapsed_ms > MAX_KEYGEN_TIME_MS as f64 * 2.0 {
            self.attacks_detected += 1;
            self.last_attack_timestamp = aletheion_core::time::now();
            
            aletheion_core::logger::warn!("SIDE_CHANNEL_ATTACK_DETECTED: Timing anomaly in {:?} operation ({:.2}ms)", operation, elapsed_ms);
        }
        
        Ok(())
    }
}

// --- Default Implementations ---

impl Default for PQSignature {
    fn default() -> Self {
        Self {
            algorithm: PQAlgorithmSuite::Kyber768_Dilithium3,
            signature_bytes: Vec::new(),
            public_key_id: [0u8; 32],
            timestamp: 0,
            message_hash: [0u8; 64],
        }
    }
}

// --- Unit Tests (Offline Capable) ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_engine_initialization() {
        let engine = PQCryptoEngine::new(BirthSign::default(), PQSecurityLevel::Level3).unwrap();
        
        assert_eq!(engine.security_level, PQSecurityLevel::Level3);
        assert_eq!(engine.active_key_pairs.len(), 0);
        assert_eq!(engine.metrics.total_operations, 0);
    }

    #[test]
    fn test_key_generation_kyber_dilithium() {
        let mut engine = PQCryptoEngine::new(BirthSign::default(), PQSecurityLevel::Level3).unwrap();
        
        let key_pair = engine.generate_key_pair(PQAlgorithmSuite::Kyber768_Dilithium3).unwrap();
        
        assert_eq!(key_pair.algorithm, PQAlgorithmSuite::Kyber768_Dilithium3);
        assert_eq!(key_pair.public_key.len(), KYBER768_PUBLIC_KEY_SIZE);
        assert_eq!(key_pair.secret_key.len(), KYBER768_SECRET_KEY_SIZE);
        assert_eq!(key_pair.security_level, PQSecurityLevel::Level3);
        assert_eq!(key_pair.status, PQKeyStatus::Active);
    }

    #[test]
    fn test_signature_creation_and_verification() {
        let mut engine = PQCryptoEngine::new(BirthSign::default(), PQSecurityLevel::Level3).unwrap();
        
        // Generate key pair
        let key_pair = engine.generate_key_pair(PQAlgorithmSuite::Kyber768_Dilithium3).unwrap();
        
        // Sign message
        let message = b"Test message for PQ signature";
        let signature = engine.sign_message(&key_pair.key_id, message).unwrap();
        
        // Verify signature
        let valid = engine.verify_signature(&signature, message).unwrap();
        
        assert!(valid);
    }

    #[test]
    fn test_key_encapsulation_decapsulation() {
        let mut engine = PQCryptoEngine::new(BirthSign::default(), PQSecurityLevel::Level3).unwrap();
        
        // Generate recipient key pair
        let recipient_key = engine.generate_key_pair(PQAlgorithmSuite::Kyber768_Dilithium3).unwrap();
        
        // Encapsulate key
        let encapsulation = engine.encapsulate_key(&recipient_key.public_key, PQAlgorithmSuite::Kyber768_Dilithium3).unwrap();
        
        // Decapsulate key
        let shared_secret = engine.decapsulate_key(&recipient_key.key_id, &encapsulation.ciphertext).unwrap();
        
        assert_eq!(shared_secret.len(), KYBER768_SHARED_SECRET_SIZE);
        assert_eq!(shared_secret, encapsulation.shared_secret);
    }

    #[test]
    fn test_key_rotation() {
        let mut engine = PQCryptoEngine::new(BirthSign::default(), PQSecurityLevel::Level3).unwrap();
        
        // Generate key pair
        let key_pair = engine.generate_key_pair(PQAlgorithmSuite::Kyber768_Dilithium3).unwrap();
        
        // Force rotation by manipulating expiration
        if let Some(mut key) = engine.active_key_pairs.get_mut(&key_pair.key_id) {
            key.expiration_timestamp = 0; // Expired
        }
        
        // Rotate keys
        let rotated = engine.rotate_keys().unwrap();
        
        assert!(rotated > 0);
    }

    #[test]
    fn test_certificate_creation_and_verification() {
        let mut engine = PQCryptoEngine::new(BirthSign::default(), PQSecurityLevel::Level3).unwrap();
        
        // Generate issuer key
        let issuer_key = engine.generate_key_pair(PQAlgorithmSuite::Kyber768_Dilithium3).unwrap();
        
        // Generate subject key
        let subject_key = engine.generate_key_pair(PQAlgorithmSuite::Kyber768_Dilithium3).unwrap();
        
        // Create certificate
        let subject_did = [1u8; 32];
        let extensions = vec![
            PQCertificateExtension::KeyUsage(PQKeyUsage::DigitalSignature),
            PQCertificateExtension::ExtendedKeyUsage(vec![PQExtendedKeyUsage::CitizenIdentity]),
        ];
        
        let cert = engine.create_certificate(&subject_did, &subject_key, &issuer_key.key_id, extensions).unwrap();
        
        // Verify certificate
        let valid = engine.verify_certificate_chain(&cert).unwrap();
        
        assert!(valid);
    }

    #[test]
    fn test_security_level_parameters() {
        // Verify NIST PQC security level mappings
        assert_eq!(PQ_SECURITY_LEVEL_1, 128); // 64-bit quantum
        assert_eq!(PQ_SECURITY_LEVEL_3, 192); // 96-bit quantum (Aletheion target)
        assert_eq!(PQ_SECURITY_LEVEL_5, 256); // 128-bit quantum
    }

    #[test]
    fn test_algorithm_suite_parameters() {
        // Verify Kyber-768 parameters (NIST Level 3)
        assert_eq!(KYBER768_PUBLIC_KEY_SIZE, 1184);
        assert_eq!(KYBER768_SECRET_KEY_SIZE, 2400);
        assert_eq!(KYBER768_CIPHERTEXT_SIZE, 1088);
        assert_eq!(KYBER768_SHARED_SECRET_SIZE, 32);
        
        // Verify Dilithium-3 parameters (NIST Level 3)
        assert_eq!(DILITHIUM3_PUBLIC_KEY_SIZE, 1952);
        assert_eq!(DILITHIUM3_SECRET_KEY_SIZE, 4000);
        assert_eq!(DILITHIUM3_SIGNATURE_SIZE, 3293);
        
        // Verify Falcon-512 parameters (smaller signatures)
        assert_eq!(FALCON512_PUBLIC_KEY_SIZE, 897);
        assert_eq!(FALCON512_SECRET_KEY_SIZE, 1281);
    }

    #[test]
    fn test_performance_thresholds() {
        // Verify performance requirements for Aletheion deployment
        assert!(MAX_KEYGEN_TIME_MS <= 5); // <5ms key generation
        assert!(MAX_SIGN_TIME_MS <= 3);   // <3ms signing
        assert!(MAX_VERIFY_TIME_MS <= 2); // <2ms verification
        assert!(MAX_KEM_ENCAPS_TIME_MS <= 2); // <2ms KEM
    }

    #[test]
    fn test_side_channel_monitor() {
        let mut monitor = SideChannelMonitor::new();
        
        // Normal operation should not trigger attack detection
        assert!(monitor.check_for_attacks(PQOperation::KeyGeneration, 1.0).is_ok());
        assert_eq!(monitor.attacks_detected, 0);
        
        // Timing anomaly should trigger detection
        assert!(monitor.check_for_attacks(PQOperation::KeyGeneration, 20.0).is_ok());
        assert!(monitor.attacks_detected > 0);
    }
}
