-- Aletheion FOG-Aware Workload Router v20260310
-- License: BioticTreaty_v3
-- Compliance: Neurorights_v1, ISO14851, OECD301

local FOG_ROUTER_VERSION = 20260310
local MAX_NODES = 256
local MAX_WORKLOAD_QUEUE = 1024

local RiskCoordinate = {}
RiskCoordinate.__index = RiskCoordinate

function RiskCoordinate:new(id, rx, threshold, timestamp_ns)
    local self = setmetatable({}, RiskCoordinate)
    self.id = id or 0
    self.rx = rx or 0.0
    self.threshold = threshold or 1.0
    self.timestamp_ns = timestamp_ns or 0
    return self
end

function RiskCoordinate:violated()
    return self.rx > self.threshold
end

function RiskCoordinate:margin()
    return self.threshold - self.rx
end

local NodeShard = {}
NodeShard.__index = NodeShard

function NodeShard:new(node_id)
    local self = setmetatable({}, NodeShard)
    self.node_id = node_id or 0
    self.energy_surplus_kwh = 0.0
    self.hydraulic_headroom_m3h = 0.0
    self.biosurface_ok = false
    self.lyapunov_ok = false
    self.r_fog = 0.0
    self.r_solids = 0.0
    self.risk_coordinates = {}
    self.active = false
    return self
end

function NodeShard:update_risk_coordinates(ts)
    table.insert(self.risk_coordinates, RiskCoordinate:new(101, self.r_fog, 1.0, ts))
    table.insert(self.risk_coordinates, RiskCoordinate:new(102, self.r_solids, 1.0, ts))
end

function NodeShard:tailwind_valid()
    return self.energy_surplus_kwh > 5.0
end

function NodeShard:biosurface_ok()
    return self.biosurface_ok == true
end

function NodeShard:hydraulic_ok()
    return self.hydraulic_headroom_m3h > 20.0
end

function NodeShard:lyapunov_ok()
    return self.lyapunov_ok == true
end

function NodeShard:all_predicates_pass()
    return self:tailwind_valid() and 
           self:biosurface_ok() and 
           self:hydraulic_ok() and 
           self:lyapunov_ok() and
           self.r_fog < 0.8 and
           self.r_solids < 0.8
end

local FOGRouter = {}
FOGRouter.__index = FOGRouter

function FOGRouter:new(router_id)
    local self = setmetatable({}, FOGRouter)
    self.router_id = router_id or 0
    self.nodes = {}
    self.node_count = 0
    self.workload_queue = {}
    self.queue_size = 0
    self.routing_decisions = {}
    self.decision_count = 0
    return self
end

function FOGRouter:register_node(node)
    if self.node_count >= MAX_NODES then
        return false, "NODE_LIMIT_EXCEEDED"
    end
    self.nodes[self.node_count + 1] = node
    self.node_count = self.node_count + 1
    return true, "OK"
end

function FOGRouter:enqueue_workload(workload)
    if self.queue_size >= MAX_WORKLOAD_QUEUE then
        return false, "QUEUE_FULL"
    end
    self.workload_queue[self.queue_size + 1] = workload
    self.queue_size = self.queue_size + 1
    return true, "OK"
end

function FOGRouter:find_best_node(workload)
    local best_node = nil
    local best_score = -1.0
    for i = 1, self.node_count do
        local node = self.nodes[i]
        if node.active and node:all_predicates_pass() then
            local score = node.energy_surplus_kwh * 0.4 + 
                         node.hydraulic_headroom_m3h * 0.3 + 
                         (1.0 - node.r_fog) * 0.2 + 
                         (1.0 - node.r_solids) * 0.1
            if score > best_score then
                best_score = score
                best_node = node
            end
        end
    end
    return best_node, best_score
end

function FOGRouter:route_high_fog_workload(workload, ts)
    for i = 1, self.node_count do
        local node = self.nodes[i]
        if node.active and node.r_fog < 0.5 and node:biosurface_ok() then
            return node, "HIGH_FOG_ROUTED"
        end
    end
    return nil, "NO_SUITABLE_NODE"
end

function FOGRouter:route_high_solids_workload(workload, ts)
    for i = 1, self.node_count do
        local node = self.nodes[i]
        if node.active and node.r_solids < 0.5 and node:hydraulic_ok() then
            return node, "HIGH_SOLIDS_ROUTED"
        end
    end
    return nil, "NO_SUITABLE_NODE"
end

function FOGRouter:process_queue(ts)
    local routed = 0
    local failed = 0
    for i = 1, self.queue_size do
        local workload = self.workload_queue[i]
        local node, reason
        if workload.fog_load > 0.7 then
            node, reason = self:route_high_fog_workload(workload, ts)
        elseif workload.solids_load > 0.7 then
            node, reason = self:route_high_solids_workload(workload, ts)
        else
            node, reason = self:find_best_node(workload)
        end
        if node then
            self.decision_count = self.decision_count + 1
            self.routing_decisions[self.decision_count] = {
                workload_id = workload.id,
                node_id = node.node_id,
                reason = reason,
                timestamp_ns = ts
            }
            routed = routed + 1
        else
            failed = failed + 1
        end
    end
    self.workload_queue = {}
    self.queue_size = 0
    return routed, failed
end

function FOGRouter:get_routing_stats()
    local total = self.decision_count
    if total == 0 then return 0.0, 0, 0 end
    local fog_routed = 0
    local solids_routed = 0
    for i = 1, total do
        local dec = self.routing_decisions[i]
        if dec.reason == "HIGH_FOG_ROUTED" then fog_routed = fog_routed + 1 end
        if dec.reason == "HIGH_SOLIDS_ROUTED" then solids_routed = solids_routed + 1 end
    end
    return fog_routed / total, solids_routed / total, total
end

return {
    FOGRouter = FOGRouter,
    NodeShard = NodeShard,
    RiskCoordinate = RiskCoordinate,
    VERSION = FOG_ROUTER_VERSION,
}
