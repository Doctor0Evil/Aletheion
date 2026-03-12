local router = require("qpu_datashard.pos.ideis_pos_smartcity_router")

local M = {}

local function to_session_token(session)
    return {
        session_id = session.session_id,
        subject_did = session.subject_did,
        device_id = session.device_id,
        issued_at = session.issued_at,
        expires_at = session.expires_at,
        jurisdiction = session.jurisdiction,
        policy_profile = session.policy_profile,
        auth_context = session.auth_context,
    }
end

local function to_manifest(manifest)
    return manifest
end

local function to_event(ev)
    return {
        event_id = ev.event_id,
        sku = ev.sku,
        quantity = ev.quantity,
        actor_did = ev.actor_did,
        device_id = ev.device_id,
        timestamp = ev.timestamp,
        policy_profile = ev.policy_profile,
    }
end

function M.vnode_pos_sale(core, vnode_path, session, manifest, ev)
    local tok = to_session_token(session)
    local m = to_manifest(manifest)
    local payload = to_event(ev)
    payload.policy_profile = vnode_path .. "::POS"
    return router.route_pos_request(core, tok, m, "POS_SALE", payload)
end

function M.vnode_inventory_sync(core, vnode_path, session, manifest, ev)
    local tok = to_session_token(session)
    local m = to_manifest(manifest)
    local payload = to_event(ev)
    payload.policy_profile = vnode_path .. "::INVENTORY"
    return router.route_pos_request(core, tok, m, "INVENTORY_SYNC", payload)
end

return M
