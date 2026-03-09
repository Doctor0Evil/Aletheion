-- Aletheion City-Wide Dashboard Aggregator v20260310
-- License: BioticTreaty_v3
-- Compliance: Phoenix_Municipal_Code_2026_BioticTreaty_v3_Open_Data_Standards

local DASHBOARD_AGGREGATOR_VERSION = 20260310
local MAX_DATA_SOURCES = 512
local MAX_DASHBOARD_WIDGETS = 2048
local MAX_CITIZEN_VIEWS = 1048576
local REFRESH_INTERVAL_S = 300

local DataSourceType = {
    SENSOR = 1, DATABASE = 2, API = 3, STREAM = 4,
    FILE = 5, MANUAL = 6, CALCULATED = 7, AGGREGATED = 8
}

local WidgetType = {
    METRIC = 1, CHART = 2, MAP = 3, TABLE = 4,
    ALERT = 5, STATUS = 6, TREND = 7, COMPARISON = 8
}

local DataSource = {}
DataSource.__index = DataSource

function DataSource:new(source_id, source_type, name, endpoint)
    local self = setmetatable({}, DataSource)
    self.source_id = source_id or 0
    self.source_type = source_type or 1
    self.name = name or ""
    self.endpoint = endpoint or ""
    self.subsystem = ""
    self.data_format = ""
    self.refresh_interval_s = REFRESH_INTERVAL_S
    self.last_refresh_ns = 0
    self.next_refresh_ns = 0
    self.operational = true
    self.error_count = 0
    self.total_requests = 0
    self.successful_requests = 0
    self.average_latency_ms = 0.0
    self.data_quality_score = 1.0
    return self
end

function DataSource:refresh(now_ns)
    if now_ns < self.next_refresh_ns then return false end
    self.last_refresh_ns = now_ns
    self.next_refresh_ns = now_ns + (self.refresh_interval_s * 1000000000)
    self.total_requests = self.total_requests + 1
    if self.operational then
        self.successful_requests = self.successful_requests + 1
    else
        self.error_count = self.error_count + 1
    end
    if self.total_requests > 0 then
        self.data_quality_score = self.successful_requests / self.total_requests
    end
    return self.operational
end

function DataSource:health_score()
    local availability = self.data_quality_score
    local latency_score = 1.0
    if self.average_latency_ms > 1000 then latency_score = 0.5
    elseif self.average_latency_ms > 500 then latency_score = 0.7
    elseif self.average_latency_ms > 100 then latency_score = 0.9 end
    local error_penalty = math.min(self.error_count * 0.02, 0.3)
    return math.max(0.0, availability * 0.5 + latency_score * 0.3 + (1.0 - error_penalty) * 0.2)
end

local DashboardWidget = {}
DashboardWidget.__index = DashboardWidget

function DashboardWidget:new(widget_id, widget_type, title, data_source_ids)
    local self = setmetatable({}, DashboardWidget)
    self.widget_id = widget_id or 0
    self.widget_type = widget_type or 1
    self.title = title or ""
    self.data_source_ids = data_source_ids or {}
    self.description = ""
    self.refresh_interval_s = REFRESH_INTERVAL_S
    self.last_update_ns = 0
    self.current_value = 0.0
    self.previous_value = 0.0
    self.target_value = 0.0
    self.unit = ""
    self.min_value = 0.0
    self.max_value = 100.0
    self.threshold_warning = 0.0
    self.threshold_critical = 0.0
    self.status = "NORMAL"
    self.visible = true
    self.public = true
    self.accessibility_compliant = true
    return self
end

function DashboardWidget:update_value(value, now_ns)
    self.previous_value = self.current_value
    self.current_value = value
    self.last_update_ns = now_ns
    if value >= self.threshold_critical then self.status = "CRITICAL"
    elseif value >= self.threshold_warning then self.status = "WARNING"
    else self.status = "NORMAL" end
end

function DashboardWidget:compute_trend()
    if self.previous_value == 0 then return 0.0 end
    return (self.current_value - self.previous_value) / self.previous_value * 100.0
end

local CitizenView = {}
CitizenView.__index = CitizenView

function CitizenView:new(view_id, citizen_did, view_name)
    local self = setmetatable({}, CitizenView)
    self.view_id = view_id or 0
    self.citizen_did = citizen_did or ""
    self.view_name = view_name or ""
    self.widget_ids = {}
    self.widget_count = 0
    self.layout = "GRID"
    self.theme = "LIGHT"
    self.created_at_ns = 0
    self.last_accessed_ns = 0
    self.access_count = 0
    self.shared = false
    self.favorite = false
    return self
end

function CitizenView:add_widget(widget_id)
    if self.widget_count >= 50 then return false end
    self.widget_ids[self.widget_count + 1] = widget_id
    self.widget_count = self.widget_count + 1
    return true
end

local CityDashboardAggregator = {}
CityDashboardAggregator.__index = CityDashboardAggregator

function CityDashboardAggregator:new(aggregator_id, city_code, region)
    local self = setmetatable({}, CityDashboardAggregator)
    self.aggregator_id = aggregator_id or 0
    self.city_code = city_code or ""
    self.region = region or ""
    self.data_sources = {}
    self.source_count = 0
    self.widgets = {}
    self.widget_count = 0
    self.citizen_views = {}
    self.view_count = 0
    self.total_data_requests = 0
    self.successful_data_requests = 0
    self.average_refresh_time_ms = 0.0
    self.system_health_score = 1.0
    self.citizen_satisfaction_score = 0.0
    self.last_full_refresh_ns = 0
    return self
end

function CityDashboardAggregator:register_data_source(source)
    if self.source_count >= MAX_DATA_SOURCES then return false, "SOURCE_LIMIT" end
    self.data_sources[self.source_count + 1] = source
    self.source_count = self.source_count + 1
    return true, "OK"
end

function CityDashboardAggregator:register_widget(widget)
    if self.widget_count >= MAX_DASHBOARD_WIDGETS then return false, "WIDGET_LIMIT" end
    self.widgets[self.widget_count + 1] = widget
    self.widget_count = self.widget_count + 1
    return true, "OK"
end

function CityDashboardAggregator:register_citizen_view(view)
    if self.view_count >= MAX_CITIZEN_VIEWS then return false, "VIEW_LIMIT" end
    self.citizen_views[self.view_count + 1] = view
    self.view_count = self.view_count + 1
    return true, "OK"
end

function CityDashboardAggregator:refresh_all_sources(now_ns)
    local refreshed = 0
    local failed = 0
    for i = 1, self.source_count do
        local source = self.data_sources[i]
        if source:refresh(now_ns) then
            refreshed = refreshed + 1
        else
            failed = failed + 1
        end
        self.total_data_requests = self.total_data_requests + 1
        if source.operational then
            self.successful_data_requests = self.successful_data_requests + 1
        end
    end
    if self.total_data_requests > 0 then
        self.average_refresh_time_ms = (self.average_refresh_time_ms * (self.total_data_requests - 1) + 
                                        refreshed * 100) / self.total_data_requests
    end
    self.last_full_refresh_ns = now_ns
    return refreshed, failed
end

function CityDashboardAggregator:update_widget_values(now_ns)
    for i = 1, self.widget_count do
        local widget = self.widgets[i]
        local source_values = {}
        for j, source_id in ipairs(widget.data_source_ids) do
            for k = 1, self.source_count do
                if self.data_sources[k].source_id == source_id then
                    table.insert(source_values, self.data_sources[k].data_quality_score)
                    break
                end
            end
        end
        if #source_values > 0 then
            local avg_value = 0
            for _, v in ipairs(source_values) do avg_value = avg_value + v end
            widget:update_value(avg_value / #source_values, now_ns)
        end
    end
end

function CityDashboardAggregator:compute_system_health_score()
    if self.source_count == 0 then return 0.0 end
    local total_health = 0.0
    for i = 1, self.source_count do
        total_health = total_health + self.data_sources[i]:health_score()
    end
    self.system_health_score = total_health / self.source_count
    local request_success_rate = self.successful_data_requests / self.total_data_requests.max(1)
    self.system_health_score = self.system_health_score * 0.7 + request_success_rate * 0.3
    return self.system_health_score
end

function CityDashboardAggregator:generate_dashboard_summary(now_ns)
    self:refresh_all_sources(now_ns)
    self:update_widget_values(now_ns)
    self:compute_system_health_score()
    local critical_widgets = 0
    local warning_widgets = 0
    for i = 1, self.widget_count do
        if self.widgets[i].status == "CRITICAL" then critical_widgets = critical_widgets + 1
        elseif self.widgets[i].status == "WARNING" then warning_widgets = warning_widgets + 1 end
    end
    local operational_sources = 0
    for i = 1, self.source_count do
        if self.data_sources[i].operational then operational_sources = operational_sources + 1 end
    end
    return {
        aggregator_id = self.aggregator_id,
        city_code = self.city_code,
        region = self.region,
        summary_timestamp_ns = now_ns,
        total_data_sources = self.source_count,
        operational_sources = operational_sources,
        total_widgets = self.widget_count,
        critical_widgets = critical_widgets,
        warning_widgets = warning_widgets,
        total_citizen_views = self.view_count,
        total_data_requests = self.total_data_requests,
        successful_data_requests = self.successful_data_requests,
        request_success_rate = self.successful_data_requests / self.total_data_requests.max(1),
        average_refresh_time_ms = self.average_refresh_time_ms,
        system_health_score = self.system_health_score,
        last_full_refresh_ns = self.last_full_refresh_ns
    }
end

function CityDashboardAggregator:identify_data_gaps()
    local gaps = {}
    local count = 0
    for i = 1, self.source_count do
        local source = self.data_sources[i]
        if source.data_quality_score < 0.8 or source.error_count > 10 then
            count = count + 1
            gaps[count] = {
                source_id = source.source_id,
                source_name = source.name,
                data_quality_score = source.data_quality_score,
                error_count = source.error_count,
                health_score = source:health_score(),
                recommended_action = "Investigate and remediate data source"
            }
        end
    end
    return gaps, count
end

function CityDashboardAggregator:compute_citizen_engagement_score()
    if self.view_count == 0 then return 0.0 end
    local total_accesses = 0
    for i = 1, self.view_count do
        total_accesses = total_accesses + self.citizen_views[i].access_count
    end
    local avg_accesses = total_accesses / self.view_count
    local engagement_score = math.min(avg_accesses / 100, 1.0)
    self.citizen_satisfaction_score = engagement_score
    return engagement_score
end

return {
    CityDashboardAggregator = CityDashboardAggregator,
    DataSource = DataSource,
    DashboardWidget = DashboardWidget,
    CitizenView = CitizenView,
    DataSourceType = DataSourceType,
    WidgetType = WidgetType,
    VERSION = DASHBOARD_AGGREGATOR_VERSION,
}
