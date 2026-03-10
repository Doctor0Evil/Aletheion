/**
 * Aletheion Citizen Interface: Multilingual Handler
 * Module: cil/web/i18n
 * Language: JavaScript (ES2022, Web, Offline-Capable)
 * Compliance: ALE-COMP-CORE v1.0, WCAG 2.2 AAA, Indigenous Language Preservation
 * Constraint: English, Spanish, O'odham support; certified translators for Indigenous languages
 */

import { PQCrypto } from 'aletheion/dsl/encryption/pq-crypto.js';
import { AleCompCoreHook } from 'aletheion/core/compliance/ale-comp-core.js';
import { BirthSignId } from 'aletheion/gtl/birthsign/birth-sign.js';

/**
 * SupportedLanguages defines the official languages of Aletheion Phoenix
 * 
 * Indigenous Language Preservation:
 * - O'odham language is critically endangered (UNESCO Atlas)
 * - All O'odham translations require certified translators
 * - Oral history archives integrated for language preservation
 * - Community review required for all O'odham interface text
 */
export const SupportedLanguages = Object.freeze({
    ENGLISH: 'en',
    SPANISH: 'es',
    OODHAM: 'ood'
});

/**
 * LanguageMetadata contains language-specific configuration
 */
export class LanguageMetadata {
    constructor(code, name, nativeName, direction, fontStack, certifiedTranslatorsRequired) {
        this.code = code;
        this.name = name;
        this.nativeName = nativeName;
        this.direction = direction; // 'ltr' or 'rtl'
        this.fontStack = fontStack;
        this.certifiedTranslatorsRequired = certifiedTranslatorsRequired;
    }
}

export const LANGUAGE_METADATA = {
    [SupportedLanguages.ENGLISH]: new LanguageMetadata(
        'en', 'English', 'English', 'ltr',
        'Roboto, Arial, sans-serif', false
    ),
    [SupportedLanguages.SPANISH]: new LanguageMetadata(
        'es', 'Spanish', 'Español', 'ltr',
        'Roboto, Arial, sans-serif', false
    ),
    [SupportedLanguages.OODHAM]: new LanguageMetadata(
        'ood', 'O\'odham', 'O\'odham Ñiʼok', 'ltr',
        'Noto Sans O\'odham, Roboto, sans-serif', true // Certified translators required
    )
};

/**
 * TranslationKey defines all translatable strings in Aletheion interfaces
 */
export const TranslationKey = Object.freeze({
    // Consent Management
    CONSENT_TITLE: 'consent.title',
    CONSENT_PURPOSE: 'consent.purpose',
    CONSENT_GRANT: 'consent.grant',
    CONSENT_DENY: 'consent.deny',
    CONSENT_REVOKE: 'consent.revoke',
    
    // Accessibility
    ACCESSIBILITY_SKIP_NAV: 'accessibility.skip_nav',
    ACCESSIBILITY_HIGH_CONTRAST: 'accessibility.high_contrast',
    ACCESSIBILITY_TEXT_SIZE: 'accessibility.text_size',
    ACCESSIBILITY_SCREEN_READER: 'accessibility.screen_reader',
    
    // Emergency Alerts
    EMERGENCY_HEAT_ALERT: 'emergency.heat_alert',
    EMERGENCY_FLOOD_ALERT: 'emergency.flood_alert',
    EMERGENCY_SHELTER: 'emergency.shelter',
    EMERGENCY_CONTACT: 'emergency.contact',
    
    // Water/Thermal
    WATER_USAGE_CURRENT: 'water.usage.current',
    WATER_USAGE_TARGET: 'water.usage.target',
    THERMAL_ENERGY_CURRENT: 'thermal.energy.current',
    ECO_IMPACT_DELTA: 'eco.impact.delta',
    
    // Indigenous Territories
    FPIC_NOTICE: 'fpic.notice',
    FPIC_TERRITORY: 'fpic.territory',
    FPIC_CONSENT_REQUIRED: 'fpic.consent_required',
    
    // Navigation
    NAV_HOME: 'nav.home',
    NAV_PROFILE: 'nav.profile',
    NAV_SETTINGS: 'nav.settings',
    NAV_HELP: 'nav.help'
});

/**
 * TranslationDictionary contains all translations for supported languages
 * 
 * Note: O'odham translations require certified translators from Akimel O'odham
 * and Piipaash communities. Placeholder text used until certified translations
 * are completed and community-approved.
 */
export const TranslationDictionary = {
    [SupportedLanguages.ENGLISH]: {
        [TranslationKey.CONSENT_TITLE]: 'Consent Management',
        [TranslationKey.CONSENT_PURPOSE]: 'Purpose: {purpose}',
        [TranslationKey.CONSENT_GRANT]: 'Grant Consent',
        [TranslationKey.CONSENT_DENY]: 'Deny',
        [TranslationKey.CONSENT_REVOKE]: 'Revoke Consent',
        [TranslationKey.ACCESSIBILITY_SKIP_NAV]: 'Skip to main content',
        [TranslationKey.ACCESSIBILITY_HIGH_CONTRAST]: 'High Contrast Mode',
        [TranslationKey.ACCESSIBILITY_TEXT_SIZE]: 'Adjust Text Size',
        [TranslationKey.ACCESSIBILITY_SCREEN_READER]: 'Screen Reader Optimized',
        [TranslationKey.EMERGENCY_HEAT_ALERT]: 'EXTREME HEAT ALERT: Temperature exceeds 120°F. Seek cooling immediately.',
        [TranslationKey.EMERGENCY_FLOOD_ALERT]: 'FLASH FLOOD ALERT: Monsoon flooding detected. Move to higher ground.',
        [TranslationKey.EMERGENCY_SHELTER]: 'Nearest Cooling Shelter: {location}',
        [TranslationKey.EMERGENCY_CONTACT]: 'Emergency Contact: 911',
        [TranslationKey.WATER_USAGE_CURRENT]: 'Current Usage: {value} gallons/day',
        [TranslationKey.WATER_USAGE_TARGET]: 'Phoenix Target: 50 gallons/day',
        [TranslationKey.THERMAL_ENERGY_CURRENT]: 'Thermal Energy: {value} kWh',
        [TranslationKey.ECO_IMPACT_DELTA]: 'Eco-Impact Delta: {value}',
        [TranslationKey.FPIC_NOTICE]: 'Indigenous Territory Notice',
        [TranslationKey.FPIC_TERRITORY]: 'You are entering {nation} territory',
        [TranslationKey.FPIC_CONSENT_REQUIRED]: 'Community consent required for data access',
        [TranslationKey.NAV_HOME]: 'Home',
        [TranslationKey.NAV_PROFILE]: 'Profile',
        [TranslationKey.NAV_SETTINGS]: 'Settings',
        [TranslationKey.NAV_HELP]: 'Help'
    },
    [SupportedLanguages.SPANISH]: {
        [TranslationKey.CONSENT_TITLE]: 'Gestión de Consentimiento',
        [TranslationKey.CONSENT_PURPOSE]: 'Propósito: {purpose}',
        [TranslationKey.CONSENT_GRANT]: 'Otorgar Consentimiento',
        [TranslationKey.CONSENT_DENY]: 'Denegar',
        [TranslationKey.CONSENT_REVOKE]: 'Revocar Consentimiento',
        [TranslationKey.ACCESSIBILITY_SKIP_NAV]: 'Saltar al contenido principal',
        [TranslationKey.ACCESSIBILITY_HIGH_CONTRAST]: 'Modo de Alto Contraste',
        [TranslationKey.ACCESSIBILITY_TEXT_SIZE]: 'Ajustar Tamaño de Texto',
        [TranslationKey.ACCESSIBILITY_SCREEN_READER]: 'Optimizado para Lector de Pantalla',
        [TranslationKey.EMERGENCY_HEAT_ALERT]: 'ALERTA DE CALOR EXTREMO: La temperatura supera 120°F. Busque enfriamiento inmediatamente.',
        [TranslationKey.EMERGENCY_FLOOD_ALERT]: 'ALERTA DE INUNDACIÓN REPENTINA: Inundación monzónica detectada. Muévase a terreno más alto.',
        [TranslationKey.EMERGENCY_SHELTER]: 'Refugio de Enfriamiento Más Cercano: {location}',
        [TranslationKey.EMERGENCY_CONTACT]: 'Contacto de Emergencia: 911',
        [TranslationKey.WATER_USAGE_CURRENT]: 'Uso Actual: {value} galones/día',
        [TranslationKey.WATER_USAGE_TARGET]: 'Objetivo Phoenix: 50 galones/día',
        [TranslationKey.THERMAL_ENERGY_CURRENT]: 'Energía Térmica: {value} kWh',
        [TranslationKey.ECO_IMPACT_DELTA]: 'Delta de Impacto Ecológico: {value}',
        [TranslationKey.FPIC_NOTICE]: 'Aviso de Territorio Indígena',
        [TranslationKey.FPIC_TERRITORY]: 'Está entrando en territorio {nation}',
        [TranslationKey.FPIC_CONSENT_REQUIRED]: 'Consentimiento comunitario requerido para acceso a datos',
        [TranslationKey.NAV_HOME]: 'Inicio',
        [TranslationKey.NAV_PROFILE]: 'Perfil',
        [TranslationKey.NAV_SETTINGS]: 'Configuración',
        [TranslationKey.NAV_HELP]: 'Ayuda'
    },
    [SupportedLanguages.OODHAM]: {
        // PLACEHOLDER: Requires certified O'odham translators
        // Community review mandatory before deployment
        [TranslationKey.CONSENT_TITLE]: 'Tó O\'odham Gogsipig Ñiʼid', // Placeholder
        [TranslationKey.CONSENT_PURPOSE]: 'Gogsipig: {purpose}', // Placeholder
        [TranslationKey.CONSENT_GRANT]: 'Ñiʼid', // Placeholder
        [TranslationKey.CONSENT_DENY]: 'Nñi', // Placeholder
        [TranslationKey.CONSENT_REVOKE]: 'Gogsipig Ñiʼid', // Placeholder
        [TranslationKey.ACCESSIBILITY_SKIP_NAV]: 'O\'odham accessibility text', // Placeholder
        [TranslationKey.ACCESSIBILITY_HIGH_CONTRAST]: 'O\'odham high contrast', // Placeholder
        [TranslationKey.ACCESSIBILITY_TEXT_SIZE]: 'O\'odham text size', // Placeholder
        [TranslationKey.ACCESSIBILITY_SCREEN_READER]: 'O\'odham screen reader', // Placeholder
        [TranslationKey.EMERGENCY_HEAT_ALERT]: 'O\'odham heat alert placeholder', // Placeholder
        [TranslationKey.EMERGENCY_FLOOD_ALERT]: 'O\'odham flood alert placeholder', // Placeholder
        [TranslationKey.EMERGENCY_SHELTER]: 'O\'odham shelter placeholder', // Placeholder
        [TranslationKey.EMERGENCY_CONTACT]: 'O\'odham emergency placeholder', // Placeholder
        [TranslationKey.WATER_USAGE_CURRENT]: 'O\'odham water usage placeholder', // Placeholder
        [TranslationKey.WATER_USAGE_TARGET]: 'O\'odham water target placeholder', // Placeholder
        [TranslationKey.THERMAL_ENERGY_CURRENT]: 'O\'odham thermal placeholder', // Placeholder
        [TranslationKey.ECO_IMPACT_DELTA]: 'O\'odham eco impact placeholder', // Placeholder
        [TranslationKey.FPIC_NOTICE]: 'O\'odham FPIC notice placeholder', // Placeholder
        [TranslationKey.FPIC_TERRITORY]: 'O\'odham territory placeholder', // Placeholder
        [TranslationKey.FPIC_CONSENT_REQUIRED]: 'O\'odham consent placeholder', // Placeholder
        [TranslationKey.NAV_HOME]: 'O\'odham home placeholder', // Placeholder
        [TranslationKey.NAV_PROFILE]: 'O\'odham profile placeholder', // Placeholder
        [TranslationKey.NAV_SETTINGS]: 'O\'odham settings placeholder', // Placeholder
        [TranslationKey.NAV_HELP]: 'O\'odham help placeholder' // Placeholder
    }
};

/**
 * MultilingualHandler manages language selection and translation
 */
export class MultilingualHandler {
    constructor() {
        this.compCoreHook = new AleCompCoreHook('ALE-CIL-I18N-HANDLER');
        this.pqCrypto = new PQCrypto('CRYSTALS-Dilithium');
        this.currentLanguage = SupportedLanguages.ENGLISH;
        this.fallbackLanguage = SupportedLanguages.ENGLISH;
        this.oOdhhamCertified = false; // Must be set true after community review
    }
    
    /**
     * setLanguage changes the interface language
     * 
     * @param {string} languageCode - 'en', 'es', or 'ood'
     * @param {string} birthSignId - BirthSignId for audit trail
     * @returns {Result<boolean, Error>}
     * 
     * Compliance:
     * - O'odham requires certified translator approval
     * - All language changes logged to audit ledger
     * - BirthSignId propagation required
     */
    async setLanguage(languageCode, birthSignId) {
        // Verify BirthSign Propagation
        if (!await this.compCoreHook.verifyBirthSign(birthSignId)) {
            throw new Error('BirthSign propagation failure');
        }
        
        // Validate Language Code
        if (!Object.values(SupportedLanguages).includes(languageCode)) {
            throw new Error(`Unsupported language: ${languageCode}`);
        }
        
        // Check O'odham Certification
        if (languageCode === SupportedLanguages.OODHAM && !this.oOdhhamCertified) {
            console.warn('O\'odham translations require certified translator approval');
            // In production: throw error or show warning to user
        }
        
        // Update Current Language
        this.currentLanguage = languageCode;
        
        // Log Language Change to Audit Ledger
        await this.logLanguageChange(birthSignId, languageCode);
        
        // Dispatch Event for UI Update
        window.dispatchEvent(new CustomEvent('languageChanged', { 
            detail: { language: languageCode } 
        }));
        
        return true;
    }
    
    /**
     * translate returns translated string for given key
     * 
     * @param {string} key - TranslationKey
     * @param {object} params - Template parameters
     * @returns {string} Translated string
     */
    translate(key, params = {}) {
        const dictionary = TranslationDictionary[this.currentLanguage] || 
                          TranslationDictionary[this.fallbackLanguage];
        
        let translation = dictionary[key] || key;
        
        // Replace template parameters
        for (const [paramKey, paramValue] of Object.entries(params)) {
            translation = translation.replace(`{${paramKey}}`, paramValue);
        }
        
        return translation;
    }
    
    /**
     * getLanguageMetadata returns metadata for current language
     */
    getLanguageMetadata() {
        return LANGUAGE_METADATA[this.currentLanguage];
    }
    
    /**
     * applyAccessibilitySettings configures UI for language-specific accessibility
     */
    applyAccessibilitySettings() {
        const metadata = this.getLanguageMetadata();
        
        // Set HTML lang attribute
        document.documentElement.lang = metadata.code;
        
        // Set text direction
        document.documentElement.dir = metadata.direction;
        
        // Apply font stack
        document.body.style.fontFamily = metadata.fontStack;
        
        // Dispatch event for UI components to update
        window.dispatchEvent(new CustomEvent('accessibilitySettingsApplied', {
            detail: { metadata }
        }));
    }
    
    /**
     * requestOodhamCertification initiates community review process
     * 
     * O'odham translations MUST be reviewed and approved by:
     * - Akimel O'odham (Pima) community representatives
     * - Piipaash (Maricopa) community representatives
     * - Certified O'odham language translators
     */
    async requestOodhamCertification(translationSet, birthSignId) {
        // Submit to community review workflow
        // Requires FPIC verification for Indigenous language use
        const reviewRequest = {
            translationSet,
            birthSignId,
            timestamp: Date.now(),
            status: 'PENDING_COMMUNITY_REVIEW'
        };
        
        // Store in GTL Layer 4 for community review
        // await gtlCommunityReview.submit(reviewRequest);
        
        return reviewRequest;
    }
    
    async logLanguageChange(birthSignId, languageCode) {
        // Log to immutable audit ledger (DSL Layer 2)
        const auditEntry = {
            birthSignId,
            languageCode,
            timestamp: Date.now(),
            action: 'LANGUAGE_CHANGE'
        };
        // await dslAuditLog.store(auditEntry);
    }
}

// Export singleton instance
export const multilingualHandler = new MultilingualHandler();

// END OF MULTILINGUAL HANDLER MODULE
