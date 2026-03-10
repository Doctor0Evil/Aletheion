-- Function: Manage FPIC Lifecycle for Land/Water Projects
-- Constraints: Offline-Capable, Deterministic Timing

local FPIC_STATES = {
    PROPOSED = 1,
    CONSULTATION = 2,
    CONSENT_GRANTED = 3,
    CONSENT_DENIED = 4,
    VETO_EXECUTED = 5
}

local MAX_CONSULTATION_DAYS = 90
local ESCALATION_PATH = { "Tribal_Council", "Joint_Committee", "Community_Referendum" }

function initiate_fpic(project_id, tribal_authority_did)
    local state = FPIC_STATES.PROPOSED
    local start_time = os.time()
    local log_entry = {
        project = project_id,
        authority = tribal_authority_did,
        status = "INITIATED",
        timestamp = start_time
    }
    
    -- Log to Immutable Ledger (Aletheion Chain)
    aletheion.ledger.append("FPIC_LOG", log_entry)
    
    -- Wait for Response (Event Driven)
    while state == FPIC_STATES.PROPOSED or state == FPIC_STATES.CONSULTATION do
        local current_time = os.time()
        if (current_time - start_time) > (MAX_CONSULTATION_DAYS * 86400) then
            state = FPIC_STATES.CONSENT_DENIED -- Timeout equals Denial (Safety Default)
            trigger_veto(project_id)
            break
        end
        
        -- Check for External Signal (Tribal Authority Input)
        local signal = aletheion.io.receive("TRIBAL_CONSENT_SIGNAL", project_id)
        if signal then
            if signal.vote == "YES" then
                state = FPIC_STATES.CONSENT_GRANTED
            elseif signal.vote == "NO" then
                state = FPIC_STATES.CONSENT_DENIED
                trigger_veto(project_id)
            end
        end
        
        -- Yield to prevent blocking (Offline Resilience)
        coroutine.yield()
    end
    
    return state
end

function trigger_veto(project_id)
    -- Hard Stop on all related workflows (Water, Energy, Construction)
    aletheion.workflow.halt(project_id, "FPIC_VETO")
    aletheion.notify.send(ESCALATION_PATH[1], "VETO_ALERT", project_id)
end
