// Aletheion/cluster/src/aletheion_artemis_brainid_bridge.rs
pub struct BrainIpContext {
    pub did: String,
    pub chipset: String,
    pub sovereignty_level: u8,
}

pub struct NeuromorphicSessionMetrics {
    pub global_trust: f64,
    pub global_uncertainty: f64,
}

pub struct Ai0SovereigntyContext {
    pub brainip: BrainIpContext,
    pub neuromorphic: NeuromorphicSessionMetrics,
}

pub fn open_ai0_session(token: &BrainIPToken, trust_net: &NeuromorphicTrustNetwork)
    -> Result<Ai0SovereigntyContext, AuthError>
{
    let validator = MT6883Validator::default();
    let ctx = validator.validate_token(token, current_utc_string())?;
    validator.verify_sovereignty(&ctx, 5)?; // at least standard sovereignty

    let neuromorphic = trust_net.compute_global_metrics();
    Ok(Ai0SovereigntyContext {
        brainip: BrainIpContext {
            did: token.citizen_did.clone(),
            chipset: token.chipset_id.clone(),
            sovereignty_level: token.sovereignty_level,
        },
        neuromorphic: NeuromorphicSessionMetrics {
            global_trust: trust_net.global_trust,
            global_uncertainty: trust_net.global_uncertainty,
        },
    })
}

pub fn channel_for_turn(&self, state: &Ai0State, sov: &Ai0SovereigntyContext) -> Ai0Channel {
    let mut s = state.clone();
    s.neuromorphic_trust = sov.neuromorphic.global_trust;
    s.sovereignty_level = sov.brainip.sovereignty_level;
    if can_escalate_to_qpu(&s, &self.qpu_contract) {
        Ai0Channel::Qpu
    } else {
        Ai0Channel::Local
    }
}
