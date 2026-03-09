-- Aletheion Community Learning Hub Coordinator v20260310
-- License: BioticTreaty_v3
-- Compliance: Arizona_Education_Laws_2026_Indigenous_Education_Agreements_Accessibility_Standards_WCAG_2.2_AAA

local HUB_COORDINATOR_VERSION = 20260310
local MAX_LEARNING_HUBS = 1024
local MAX_PROGRAMS = 8192
local MAX_PARTICIPANTS = 262144
local MAX_VOLUNTEERS = 65536
local TARGET_UTILIZATION_PCT = 0.75
local MIN_ACCESSIBILITY_SCORE = 0.95

local HubType = {
    LIBRARY = 1, COMMUNITY_CENTER = 2, SCHOOL = 3, MAKER_SPACE = 4,
    SENIOR_CENTER = 5, YOUTH_CENTER = 6, TRIBAL_CENTER = 7, MOBILE = 8
}

local ProgramType = {
    LITERACY = 1, NUMERACY = 2, DIGITAL_LITERACY = 3, LANGUAGE = 4,
    VOCATIONAL = 5, ARTS = 6, HEALTH = 7, FINANCIAL = 8,
    PARENTING = 9, ELDER_CARE = 10, INDIGENOUS_KNOWLEDGE = 11, SUSTAINABILITY = 12
}

local ParticipantStatus = {
    ENROLLED = 1, ACTIVE = 2, COMPLETED = 3, DROPPED = 4,
    WAITLISTED = 5, TRANSFERRED = 6, GRADUATED = 7
}

local LearningHub = {}
LearningHub.__index = LearningHub

function LearningHub:new(hub_id, hub_name, hub_type, latitude, longitude)
    local self = setmetatable({}, LearningHub)
    self.hub_id = hub_id or 0
    self.hub_name = hub_name or ""
    self.hub_type = hub_type or 1
    self.latitude = latitude or 0.0
    self.longitude = longitude or 0.0
    self.capacity = 0
    self.current_occupancy = 0
    self.operational = true
    self.accessibility_compliant = false
    self.multilingual_support = true
    self.indigenous_cultural_competency = false
    self.free_wifi = false
    self.computer_access = false
    self.childcare_available = false
    self.meal_services = false
    self.transportation_access = false
    self.hours_of_operation = ""
    self.staff_count = 0
    self.volunteer_count = 0
    self.program_count = 0
    self.last_inspection_ns = 0
    self.created_at_ns = 0
    return self
end

function LearningHub:utilization_rate()
    if self.capacity == 0 then return 0.0 end
    return self.current_occupancy / self.capacity
end

function LearningHub:can_accept_participants()
    return self.operational and self:utilization_rate() < TARGET_UTILIZATION_PCT
end

function LearningHub:accessibility_score()
    local score = 0.0
    if self.accessibility_compliant then score = score + 0.3 end
    if self.multilingual_support then score = score + 0.2 end
    if self.indigenous_cultural_competency then score = score + 0.2 end
    if self.transportation_access then score = score + 0.15 end
    if self.childcare_available then score = score + 0.15 end
    return math.min(1.0, score)
end

local LearningProgram = {}
LearningProgram.__index = LearningProgram

function LearningProgram:new(program_id, program_name, program_type, hub_id)
    local self = setmetatable({}, LearningProgram)
    self.program_id = program_id or 0
    self.program_name = program_name or ""
    self.program_type = program_type or 1
    self.hub_id = hub_id or 0
    self.instructor_name = ""
    self.max_participants = 0
    self.current_participants = 0
    self.schedule = ""
    self.duration_weeks = 0
    self.cost_usd = 0.0
    self.scholarship_available = false
    self.accessibility_accommodations = true
    self.indigenous_knowledge_integrated = false
    self.certification_offered = false
    self.job_placement_support = false
    self.enrollment_open = true
    self.created_at_ns = 0
    self.started_at_ns = 0
    self.completed_at_ns = 0
    return self
end

function LearningProgram:enrollment_rate()
    if self.max_participants == 0 then return 0.0 end
    return self.current_participants / self.max_participants
end

function LearningProgram:is_full()
    return self.current_participants >= self.max_participants
end

function LearningProgram:completion_rate(completed_count, enrolled_count)
    if enrolled_count == 0 then return 0.0 end
    return completed_count / enrolled_count
end

local Participant = {}
Participant.__index = Participant

function Participant:new(participant_id, citizen_did, age, hub_id)
    local self = setmetatable({}, Participant)
    self.participant_id = participant_id or 0
    self.citizen_did = citizen_did or ""
    self.age = age or 0
    self.hub_id = hub_id or 0
    self.enrolled_programs = {}
    self.enrolled_program_count = 0
    self.completed_programs = {}
    self.completed_program_count = 0
    self.status = ParticipantStatus.ENROLLED
    self.accessibility_needs = {}
    self.language_preference = "en"
    self.indigenous_community = false
    self.low_income_status = false
    self.disability_status = false
    self.veteran_status = false
    self.scholarship_recipient = false
    self.childcare_needed = false
    self.transportation_needed = false
    self.enrolled_at_ns = 0
    self.last_activity_ns = 0
    return self
end

function Participant:enroll_in_program(program_id)
    if self.enrolled_program_count >= 10 then return false end
    self.enrolled_programs[self.enrolled_program_count + 1] = program_id
    self.enrolled_program_count = self.enrolled_program_count + 1
    self.status = ParticipantStatus.ACTIVE
    return true
end

function Participant:complete_program(program_id)
    for i = 1, self.enrolled_program_count do
        if self.enrolled_programs[i] == program_id then
            self.completed_programs[self.completed_program_count + 1] = program_id
            self.completed_program_count = self.completed_program_count + 1
            return true
        end
    end
    return false
end

local Volunteer = {}
Volunteer.__index = Volunteer

function Volunteer:new(volunteer_id, citizen_did, hub_id)
    local self = setmetatable({}, Volunteer)
    self.volunteer_id = volunteer_id or 0
    self.citizen_did = citizen_did or ""
    self.hub_id = hub_id or 0
    self.skills = {}
    self.skill_count = 0
    self.hours_contributed = 0
    self.programs_supported = 0
    self.participants_mentored = 0
    self.background_check_completed = false
    self.training_completed = false
    self.accessibility_trained = false
    self.indigenous_cultural_trained = false
    self.active = true
    self.joined_at_ns = 0
    self.last_activity_ns = 0
    return self
end

function Volunteer:log_hours(hours, now_ns)
    self.hours_contributed = self.hours_contributed + hours
    self.last_activity_ns = now_ns
end

local CommunityLearningHubCoordinator = {}
CommunityLearningHubCoordinator.__index = CommunityLearningHubCoordinator

function CommunityLearningHubCoordinator:new(coordinator_id, city_code, region)
    local self = setmetatable({}, CommunityLearningHubCoordinator)
    self.coordinator_id = coordinator_id or 0
    self.city_code = city_code or ""
    self.region = region or ""
    self.learning_hubs = {}
    self.hub_count = 0
    self.programs = {}
    self.program_count = 0
    self.participants = {}
    self.participant_count = 0
    self.volunteers = {}
    self.volunteer_count = 0
    self.total_enrollments = 0
    self.total_completions = 0
    self.total_volunteer_hours = 0
    self.average_completion_rate = 0.0
    self.hub_utilization_rate = 0.0
    self.indigenous_participation_rate = 0.0
    self.accessibility_compliance_rate = 0.0
    self.last_reporting_ns = 0
    return self
end

function CommunityLearningHubCoordinator:register_learning_hub(hub)
    if self.hub_count >= MAX_LEARNING_HUBS then return false, "HUB_LIMIT" end
    if not hub.accessibility_compliant then return false, "ACCESSIBILITY_REQUIRED" end
    self.learning_hubs[self.hub_count + 1] = hub
    self.hub_count = self.hub_count + 1
    return true, "OK"
end

function CommunityLearningHubCoordinator:register_program(program)
    if self.program_count >= MAX_PROGRAMS then return false, "PROGRAM_LIMIT" end
    self.programs[self.program_count + 1] = program
    self.program_count = self.program_count + 1
    return true, "OK"
end

function CommunityLearningHubCoordinator:register_participant(participant)
    if self.participant_count >= MAX_PARTICIPANTS then return false, "PARTICIPANT_LIMIT" end
    self.participants[self.participant_count + 1] = participant
    self.participant_count = self.participant_count + 1
    self.total_enrollments = self.total_enrollments + 1
    return true, "OK"
end

function CommunityLearningHubCoordinator:register_volunteer(volunteer)
    if self.volunteer_count >= MAX_VOLUNTEERS then return false, "VOLUNTEER_LIMIT" end
    self.volunteers[self.volunteer_count + 1] = volunteer
    self.volunteer_count = self.volunteer_count + 1
    return true, "OK"
end

function CommunityLearningHubCoordinator:enroll_participant_in_program(participant_id, program_id, now_ns)
    local participant = nil
    local program = nil
    for i = 1, self.participant_count do
        if self.participants[i].participant_id == participant_id then
            participant = self.participants[i]
            break
        end
    end
    for i = 1, self.program_count do
        if self.programs[i].program_id == program_id then
            program = self.programs[i]
            break
        end
    end
    if not participant or not program then return false end
    if program:is_full() then return false end
    if participant:enroll_in_program(program_id) then
        program.current_participants = program.current_participants + 1
        for i = 1, self.hub_count do
            if self.learning_hubs[i].hub_id == program.hub_id then
                self.learning_hubs[i].current_occupancy = self.learning_hubs[i].current_occupancy + 1
                break
            end
        end
        return true
    end
    return false
end

function CommunityLearningHubCoordinator:complete_program(participant_id, program_id, now_ns)
    local participant = nil
    local program = nil
    for i = 1, self.participant_count do
        if self.participants[i].participant_id == participant_id then
            participant = self.participants[i]
            break
        end
    end
    for i = 1, self.program_count do
        if self.programs[i].program_id == program_id then
            program = self.programs[i]
            break
        end
    end
    if not participant or not program then return false end
    if participant:complete_program(program_id) then
        self.total_completions = self.total_completions + 1
        program.current_participants = program.current_participants - 1
        for i = 1, self.hub_count do
            if self.learning_hubs[i].hub_id == program.hub_id then
                self.learning_hubs[i].current_occupancy = self.learning_hubs[i].current_occupancy - 1
                break
            end
        end
        return true
    end
    return false
end

function CommunityLearningHubCoordinator:log_volunteer_hours(volunteer_id, hours, now_ns)
    for i = 1, self.volunteer_count do
        if self.volunteers[i].volunteer_id == volunteer_id then
            self.volunteers[i]:log_hours(hours, now_ns)
            self.total_volunteer_hours = self.total_volunteer_hours + hours
            return true
        end
    end
    return false
end

function CommunityLearningHubCoordinator:compute_hub_utilization()
    local total_capacity = 0
    local total_occupancy = 0
    for i = 1, self.hub_count do
        total_capacity = total_capacity + self.learning_hubs[i].capacity
        total_occupancy = total_occupancy + self.learning_hubs[i].current_occupancy
    end
    if total_capacity == 0 then return 0.0 end
    self.hub_utilization_rate = total_occupancy / total_capacity
    return self.hub_utilization_rate
end

function CommunityLearningHubCoordinator:compute_completion_rate()
    if self.total_enrollments == 0 then return 0.0 end
    self.average_completion_rate = self.total_completions / self.total_enrollments
    return self.average_completion_rate
end

function CommunityLearningHubCoordinator:compute_indigenous_participation()
    local indigenous_count = 0
    for i = 1, self.participant_count do
        if self.participants[i].indigenous_community then
            indigenous_count = indigenous_count + 1
        end
    end
    if self.participant_count == 0 then return 0.0 end
    self.indigenous_participation_rate = indigenous_count / self.participant_count
    return self.indigenous_participation_rate
end

function CommunityLearningHubCoordinator:compute_accessibility_compliance()
    local compliant_hubs = 0
    for i = 1, self.hub_count do
        if self.learning_hubs[i].accessibility_compliant then
            compliant_hubs = compliant_hubs + 1
        end
    end
    if self.hub_count == 0 then return 0.0 end
    self.accessibility_compliance_rate = compliant_hubs / self.hub_count
    return self.accessibility_compliance_rate
end

function CommunityLearningHubCoordinator:get_coordinator_status(now_ns)
    self:compute_hub_utilization()
    self:compute_completion_rate()
    self:compute_indigenous_participation()
    self:compute_accessibility_compliance()
    local operational_hubs = 0
    for i = 1, self.hub_count do
        if self.learning_hubs[i].operational then
            operational_hubs = operational_hubs + 1
        end
    end
    local active_volunteers = 0
    for i = 1, self.volunteer_count do
        if self.volunteers[i].active then
            active_volunteers = active_volunteers + 1
        end
    end
    return {
        coordinator_id = self.coordinator_id,
        city_code = self.city_code,
        region = self.region,
        total_learning_hubs = self.hub_count,
        operational_hubs = operational_hubs,
        total_programs = self.program_count,
        active_programs = 0,
        total_participants = self.participant_count,
        active_participants = 0,
        total_volunteers = self.volunteer_count,
        active_volunteers = active_volunteers,
        total_enrollments = self.total_enrollments,
        total_completions = self.total_completions,
        total_volunteer_hours = self.total_volunteer_hours,
        hub_utilization_rate = self.hub_utilization_rate,
        average_completion_rate = self.average_completion_rate,
        indigenous_participation_rate = self.indigenous_participation_rate,
        accessibility_compliance_rate = self.accessibility_compliance_rate,
        last_reporting_ns = self.last_reporting_ns,
        last_update_ns = now_ns
    }
end

function CommunityLearningHubCoordinator:compute_community_education_index()
    local utilization_score = self:compute_hub_utilization()
    local completion_score = self:compute_completion_rate()
    local indigenous_score = self:compute_indigenous_participation()
    local accessibility_score = self:compute_accessibility_compliance()
    local volunteer_engagement = self.total_volunteer_hours / (self.volunteer_count * 100).max(1)
    return (utilization_score * 0.25 + completion_score * 0.25 + 
            indigenous_score * 0.20 + accessibility_score * 0.15 + 
            volunteer_engagement.math.min(1.0) * 0.15)
end

return {
    CommunityLearningHubCoordinator = CommunityLearningHubCoordinator,
    LearningHub = LearningHub,
    LearningProgram = LearningProgram,
    Participant = Participant,
    Volunteer = Volunteer,
    HubType = HubType,
    ProgramType = ProgramType,
    ParticipantStatus = ParticipantStatus,
    VERSION = HUB_COORDINATOR_VERSION,
}
