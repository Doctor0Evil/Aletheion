// aletheion/governance/birthsigns/ALE-GOV-BIRTHSIGNS-BINDING-001.rs
// Runtime binding between Birth-Signs and canonical workflows, enforcing jurisdictional preconditions. [file:2]

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BirthSignId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WorkflowId(pub String);

#[derive(Clone, Debug)]
pub struct TreatyBundleId(pub String);

#[derive(Clone, Debug)]
pub struct BirthSignBundle {
    pub id: BirthSignId,
    pub treaties: Vec<TreatyBundleId>,
    pub tile: String,
    pub indigenous_overlays: Vec<String>,
    pub environmental_overlays: Vec<String>,
    pub local_policies: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct WorkflowBinding {
    pub workflow_id: WorkflowId,
    pub birth_sign: BirthSignId,
    pub treaties: Vec<TreatyBundleId>,
}

#[derive(Clone, Debug)]
pub enum BindingError {
    MissingBirthSign,
    MissingTreaties,
    JurisdictionConflict(String),
}

pub struct BirthSignRegistry {
    bundles: HashMap<BirthSignId, BirthSignBundle>,
}

impl BirthSignRegistry {
    pub fn new() -> Self {
        Self { bundles: HashMap::new() }
    }

    pub fn upsert(&mut self, bundle: BirthSignBundle) {
        self.bundles.insert(bundle.id.clone(), bundle);
    }

    pub fn bind_workflow(
        &self,
        wf_id: WorkflowId,
        bs_id: &BirthSignId,
    ) -> Result<WorkflowBinding, BindingError> {
        let bundle = self
            .bundles
            .get(bs_id)
            .ok_or(BindingError::MissingBirthSign)?;

        if bundle.treaties.is_empty() {
            return Err(BindingError::MissingTreaties);
        }

        // Placeholder for deeper conflict logic (e.g., overlapping Indigenous territories). [file:2]
        if bundle.indigenous_overlays.is_empty() && bundle.environmental_overlays.is_empty() {
            return Err(BindingError::JurisdictionConflict(
                "Birth-Sign has no overlays; unsafe to bind sensitive workflows".into(),
            ));
        }

        Ok(WorkflowBinding {
            workflow_id: wf_id,
            birth_sign: bs_id.clone(),
            treaties: bundle.treaties.clone(),
        })
    }
}
