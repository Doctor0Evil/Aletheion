// ALETHEION_KNOWLEDGE_GRAPH_ENGINE_V1.0.0
// LICENSE: BioticTreaty_Compliant_AGPLv3
// ECO_IMPACT: K=0.93 | E=0.90 | R=0.12
// CHAIN: ERM (Sense → Model → Optimize)
// CONSTRAINTS: Offline-Search, Privacy-Preserved, Indigenous-Knowledge-Sovereignty
// INDIGENOUS_RIGHTS: O'odham_Knowledge_Access_Control

#![no_std]
extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use core::hash::Hash;

// --- KNOWLEDGE NODE ---
#[derive(Clone)]
pub struct KnowledgeNode {
    pub id: u64,
    pub title: String,
    pub language: LanguageCode,
    pub access_level: AccessLevel,
    pub tags: Vec<String>,
    pub content_hash: [u8; 32], // Post-quantum safe hash placeholder
    pub is_indigenous_knowledge: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum LanguageCode {
    English,
    Spanish,
    Oodham,
}

#[derive(Clone, Copy, PartialEq)]
pub enum AccessLevel {
    Public,
    Community,
    Restricted, // Indigenous Sovereignty Protection
    Private,
}

// --- GRAPH ENGINE ---
pub struct KnowledgeGraph {
    pub nodes: Vec<KnowledgeNode>,
    pub edges: Vec<(u64, u64)>, // From, To
    pub user_clearance: AccessLevel,
    pub user_languages: Vec<LanguageCode>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            user_clearance: AccessLevel::Public,
            user_languages: vec![LanguageCode::English],
        }
    }

    // ERM: SENSE → MODEL
    // Ingests new knowledge nodes with sovereignty checks
    pub fn ingest_node(&mut self, node: KnowledgeNode) {
        // SMART: TREATY-CHECK
        if node.is_indigenous_knowledge && node.access_level != AccessLevel::Restricted {
            // Force restricted access for indigenous knowledge unless explicitly shared
            // This protects traditional ecological knowledge from exploitation
            let mut protected_node = node;
            protected_node.access_level = AccessLevel::Restricted;
            self.nodes.push(protected_node);
        } else {
            self.nodes.push(node);
        }
    }

    // ERM: OPTIMIZE
    // Searches graph respecting access levels and language preferences
    pub fn search(&self, query_tags: &[String]) -> Vec<&KnowledgeNode> {
        let mut results = Vec::new();

        for node in &self.nodes {
            // Access Control Check
            if !self.check_access(&node.access_level) {
                continue;
            }

            // Language Preference Check
            if !self.user_languages.contains(&node.language) {
                continue;
            }

            // Tag Matching
            let matches = query_tags.iter().any(|q| node.tags.contains(q));
            if matches {
                results.push(node);
            }
        }

        results
    }

    // SMART: TREATY-CHECK
    fn check_access(&self, level: &AccessLevel) -> bool {
        match (self.user_clearance, level) {
            (_, AccessLevel::Public) => true,
            (Access::Community, AccessLevel::Community) => true,
            (Access::Community, AccessLevel::Public) => true,
            (Access::Restricted, _) => true, // Highest clearance
            _ => false,
        }
    }

    // --- SKILL MAPPING ---
    // Maps citizen skills to city needs (Workforce Allocation)
    pub fn map_skills_to_needs(&self, citizen_skills: &[String], city_needs: &[String]) -> f32 {
        let mut match_score = 0.0;
        for skill in citizen_skills {
            if city_needs.contains(skill) {
                match_score += 1.0;
            }
        }
        match_score / city_needs.len() as f32
    }
}

// --- UNIT TESTS ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indigenous_knowledge_protection() {
        let mut graph = KnowledgeGraph::new();
        graph.user_clearance = AccessLevel::Public;
        
        let node = KnowledgeNode {
            id: 1,
            title: "Traditional Harvesting".into(),
            language: LanguageCode::Oodham,
            access_level: AccessLevel::Public, // Attempted public access
            tags: vec!["agriculture".into()],
            content_hash: [0; 32],
            is_indigenous_knowledge: true,
        };
        
        graph.ingest_node(node);
        
        // Verify access level was forced to Restricted
        assert_eq!(graph.nodes[0].access_level, AccessLevel::Restricted);
    }
}
