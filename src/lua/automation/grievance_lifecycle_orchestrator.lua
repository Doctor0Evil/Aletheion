-- Function: Manage Grievance State Machine with Deadline Enforcement
-- Constraints: Offline-Capable, Deterministic Timing, Immutable Audit

local GRIEVANCE_STATES = {
    FILED = 1,
    TRIAGED = 2,
    ASSIGNED = 3,
    UNDER_REVIEW = 4,
    ESCALATED = 5,
    REMEDIED = 6,
    DENIED = 7,
    APPEALED = 8,
    CLOSED = 9
}

local DEADLINE_HOURS = {
    [GRIEVANCE_STATES.FILED] = 24,
    [GRIEVANCE_STATES.TRIAGED] = 48,
    [GRIEVANCE_STATES.ASSIGNED] = 72,
    [GRIEVANCE_STATES.UNDER_REVIEW] = 168,
    [GRIEVANCE_STATES.ESCALATED] = 72,
    [GRIEVANCE_STATES.REMEDIED] = 24,
    [GRIEVANCE_STATES.DENIED] = 24,
    [GRIEVANCE_STATES.APPEALED] = 168,
    [GRIEVANCE_STATES.CLOSED] = 0
}

local ESCALATION_CHAIN = {
    [1] = "Department_Director",
    [2] = "City_Manager",
    [3] = "Citizen_Jury",
    [4] = "Community_Referendum"
}

function process_grievance_lifecycle(grievance_id, current_state, timestamp)
    local deadline_hours = DEADLINE_HOURS[current_state]
    local deadline_timestamp = timestamp + (deadline_hours * 3600)
    
    -- Log state entry to immutable ledger
    aletheion.ledger.append("GRIEVANCE_STATE_CHANGE", {
        id = grievance_id,
        state = current_state,
        entered_at = timestamp,
        deadline = deadline_timestamp
    })
    
    -- Monitor for deadline breach
    while true do
        local current_time = os.time()
        if current_time > deadline_timestamp then
            -- Deadline breached: auto-escalate
            local new_state = GRIEVANCE_STATES.ESCALATED
            local escalation_level = get_escalation_level(grievance_id)
            trigger_escalation(grievance_id, ESCALATION_CHAIN[escalation_level])
            break
        end
        
        -- Check for state change signal
        local signal = aletheion.io.receive("GRIEVANCE_ACTION", grievance_id)
        if signal then
            if signal.action == "RESOLVE" then
                transition_to(grievance_id, GRIEVANCE_STATES.REMEDIED)
            elseif signal.action == "DENY" then
                transition_to(grievance_id, GRIEVANCE_STATES.DENIED)
            elseif signal.action == "APPEAL" then
                transition_to(grievance_id, GRIEVANCE_STATES.APPEALED)
            end
            break
        end
        
        coroutine.yield() -- Yield for offline resilience
    end
end

function trigger_escalation(grievance_id, authority)
    aletheion.notify.send(authority, "GRIEVANCE_ESCALATED", grievance_id)
    aletheion.ledger.append("GRIEVANCE_ESCALATION", {
        id = grievance_id,
        escalated_to = authority,
        timestamp = os.time()
    })
    
    -- If escalation level 3+, trigger public disclosure
    local level = get_escalation_level(grievance_id)
    if level >= 3 then
        aletheion.public_disclosure.publish(grievance_id)
    end
end

function transition_to(grievance_id, new_state)
    -- Validate transition rules (no backward transitions except APPEAL)
    -- Update state in database
    -- Log to immutable ledger
end
