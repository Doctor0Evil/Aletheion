local router = {}

local ACTIONS = {
    POS_SALE = "POS_SALE",
    INVENTORY_SYNC = "INVENTORY_SYNC",
    DEV_TUNNEL_OPEN = "DEV_TUNNEL_OPEN",
    SEARCHTRACE_AUDIT = "SEARCHTRACE_AUDIT",
}

local function map_event(event)
    return {
        event_id = event.event_id or "",
        sku = event.sku or "",
        quantity = event.quantity or 0,
        actor_did = event.actor_did or "",
        device_id = event.device_id or "",
        timestamp = event.timestamp or 0,
        policy_profile = event.policy_profile or "",
    }
end

function router.route_pos_request(core, tok, manifest, action, event)
    local payload = map_event(event)
    return core:handle_pos_request(tok, manifest, action, payload)
end

function router.allowed_actions()
    return ACTIONS
end

return router
