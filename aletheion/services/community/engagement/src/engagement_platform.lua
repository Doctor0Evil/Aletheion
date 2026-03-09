-- Aletheion Community Engagement Platform v20260310
-- License: BioticTreaty_v3
-- Compliance: Phoenix_Civic_Participation_2026_Indigenous_Consultation_Protocols_Arizona_Public_Meeting_Laws

local ENGAGEMENT_PLATFORM_VERSION = 20260310
local MAX_COMMUNITY_INITIATIVES = 8192
local MAX_PARTICIPANTS = 524288
local MAX_VOTES = 2097152
local MAX_DISCUSSION_THREADS = 65536
local QUORUM_THRESHOLD_PCT = 0.10
local VOTING_PERIOD_DEFAULT_DAYS = 14

local InitiativeType = {
    POLICY_PROPOSAL = 1, BUDGET_ALLOCATION = 2, COMMUNITY_PROJECT = 3,
    REFERENDUM = 4, PETITION = 5, CONSULTATION = 6, WORKSHOP = 7,
    TOWN_HALL = 8, INDIGENOUS_CONSULTATION = 9, YOUTH_COUNCIL = 10
}

local InitiativeStatus = {
    DRAFT = 1, UNDER_REVIEW = 2, ACTIVE = 3, VOTING = 4,
    COMPLETED = 5, APPROVED = 6, REJECTED = 7, CANCELLED = 8
}

local VotingMethod = {
    SIMPLE_MAJORITY = 1, SUPER_MAJORITY = 2, UNANIMOUS = 3,
    RANKED_CHOICE = 4, QUADRATIC = 5, LIQUID_DEMOCRACY = 6
}

local CommunityInitiative = {}
CommunityInitiative.__index = CommunityInitiative

function CommunityInitiative:new(initiative_id, initiative_type, title, proposer_did)
    local self = setmetatable({}, CommunityInitiative)
    self.initiative_id = initiative_id or 0
    self.initiative_type = initiative_type or 1
    self.title = title or ""
    self.description = ""
    self.proposer_did = proposer_did or ""
    self.status = InitiativeStatus.DRAFT
    self.created_at_ns = 0
    self.voting_start_ns = 0
    self.voting_end_ns = 0
    self.voting_method = VotingMethod.SIMPLE_MAJORITY
    self.quorum_required = true
    self.quorum_threshold_pct = QUORUM_THRESHOLD_PCT
    self.participant_count = 0
    self.vote_count = 0
    self.for_votes = 0
    self.against_votes = 0
    self.abstain_votes = 0
    self.indigenous_consultation_required = false
    self.indigenous_consultation_completed = false
    self.tribal_endorsement = false
    self.city_council_review_required = false
    self.city_council_approved = false
    self.budget_impact_usd = 0.0
    self.environmental_impact_assessed = false
    self.accessibility_compliant = true
    self.multilingual_support = true
    return self
end

function CommunityInitiative:activate_voting(now_ns, duration_days)
    if self.status ~= InitiativeStatus.UNDER_REVIEW then return false end
    self.status = InitiativeStatus.VOTING
    self.voting_start_ns = now_ns
    self.voting_end_ns = now_ns + (duration_days * 86400000000000)
    return true
end

function CommunityInitiative:close_voting(now_ns)
    if self.status ~= InitiativeStatus.VOTING then return false end
    self.status = InitiativeStatus.COMPLETED
    if self.for_votes > self.against_votes then
        self.status = InitiativeStatus.APPROVED
    else
        self.status = InitiativeStatus.REJECTED
    end
    return true
end

function CommunityInitiative:quorum_met()
    if not self.quorum_required then return true end
    local eligible_voters = self.participant_count
    local participation_rate = self.vote_count / eligible_voters.max(1)
    return participation_rate >= self.quorum_threshold_pct
end

local Participant = {}
Participant.__index = Participant

function Participant:new(participant_id, citizen_did, community_district)
    local self = setmetatable({}, Participant)
    self.participant_id = participant_id or 0
    self.citizen_did = citizen_did or ""
    self.community_district = community_district or ""
    self.registration_date_ns = 0
    self.initiatives_supported = 0
    self.votes_cast = 0
    self.comments_posted = 0
    self.reputation_score = 100.0
    self.verified = false
    self.indigenous_community = false
    self.youth_participant = false
    self.senior_participant = false
    self.accessibility_needs = false
    self.language_preference = "en"
    self.notification_enabled = true
    self.active = true
    return self
end

function Participant:can_vote(initiative)
    return self.active and self.verified and initiative.status == InitiativeStatus.VOTING
end

local Vote = {}
Vote.__index = Vote

function Vote:new(vote_id, initiative_id, participant_id, vote_choice, timestamp_ns)
    local self = setmetatable({}, Vote)
    self.vote_id = vote_id or 0
    self.initiative_id = initiative_id or 0
    self.participant_id = participant_id or 0
    self.vote_choice = vote_choice or 0
    self.timestamp_ns = timestamp_ns or 0
    self.verified = false
    self.anonymous = false
    self.delegated = false
    self.delegation_chain = {}
    self.weight = 1.0
    return self
end

local DiscussionThread = {}
DiscussionThread.__index = DiscussionThread

function DiscussionThread:new(thread_id, initiative_id, title, author_id)
    local self = setmetatable({}, DiscussionThread)
    self.thread_id = thread_id or 0
    self.initiative_id = initiative_id or 0
    self.title = title or ""
    self.author_id = author_id or 0
    self.created_at_ns = 0
    self.comment_count = 0
    self.participant_count = 0
    self.upvotes = 0
    self.downvotes = 0
    self.resolved = false
    self.moderated = false
    self.indigenous_knowledge_shared = false
    self.accessibility_compliant = true
    return self
end

local CommunityEngagementPlatform = {}
CommunityEngagementPlatform.__index = CommunityEngagementPlatform

function CommunityEngagementPlatform:new(platform_id, city_code, region)
    local self = setmetatable({}, CommunityEngagementPlatform)
    self.platform_id = platform_id or 0
    self.city_code = city_code or ""
    self.region = region or ""
    self.initiatives = {}
    self.initiative_count = 0
    self.participants = {}
    self.participant_count = 0
    self.votes = {}
    self.vote_count = 0
    self.discussion_threads = {}
    self.thread_count = 0
    self.total_initiatives_proposed = 0
    self.total_initiatives_approved = 0
    self.total_participants = 0
    self.total_votes_cast = 0
    self.average_participation_rate = 0.0
    self.indigenous_consultation_count = 0
    self.indigenous_consultation_completed_count = 0
    self.youth_participation_pct = 0.0
    self.last_reporting_ns = 0
    return self
end

function CommunityEngagementPlatform:register_initiative(initiative)
    if self.initiative_count >= MAX_COMMUNITY_INITIATIVES then return false, "INITIATIVE_LIMIT" end
    self.initiatives[self.initiative_count + 1] = initiative
    self.initiative_count = self.initiative_count + 1
    self.total_initiatives_proposed = self.total_initiatives_proposed + 1
    if initiative.indigenous_consultation_required then
        self.indigenous_consultation_count = self.indigenous_consultation_count + 1
    end
    return true, "OK"
end

function CommunityEngagementPlatform:register_participant(participant)
    if self.participant_count >= MAX_PARTICIPANTS then return false, "PARTICIPANT_LIMIT" end
    self.participants[self.participant_count + 1] = participant
    self.participant_count = self.participant_count + 1
    self.total_participants = self.total_participants + 1
    return true, "OK"
end

function CommunityEngagementPlatform:cast_vote(vote)
    if self.vote_count >= MAX_VOTES then return false, "VOTE_LIMIT" end
    local initiative = nil
    for i = 1, self.initiative_count do
        if self.initiatives[i].initiative_id == vote.initiative_id then
            initiative = self.initiatives[i]
            break
        end
    end
    if not initiative or initiative.status ~= InitiativeStatus.VOTING then 
        return false, "INVALID_INITIATIVE" 
    end
    self.votes[self.vote_count + 1] = vote
    self.vote_count = self.vote_count + 1
    self.total_votes_cast = self.total_votes_cast + 1
    initiative.vote_count = initiative.vote_count + 1
    if vote.vote_choice == 1 then initiative.for_votes = initiative.for_votes + 1
    elseif vote.vote_choice == 2 then initiative.against_votes = initiative.against_votes + 1
    else initiative.abstain_votes = initiative.abstain_votes + 1 end
    return true, "OK"
end

function CommunityEngagementPlatform:create_discussion_thread(thread)
    if self.thread_count >= MAX_DISCUSSION_THREADS then return false, "THREAD_LIMIT" end
    self.discussion_threads[self.thread_count + 1] = thread
    self.thread_count = self.thread_count + 1
    return true, "OK"
end

function CommunityEngagementPlatform:compute_participation_rate()
    local total_eligible = self.participant_count
    if total_eligible == 0 then return 0.0 end
    local active_participants = 0
    for i = 1, self.participant_count do
        if self.participants[i].votes_cast > 0 or self.participants[i].comments_posted > 0 then
            active_participants = active_participants + 1
        end
    end
    self.average_participation_rate = active_participants / total_eligible
    return self.average_participation_rate
end

function CommunityEngagementPlatform:compute_youth_participation()
    local youth_count = 0
    for i = 1, self.participant_count do
        if self.participants[i].youth_participant then
            youth_count = youth_count + 1
        end
    end
    if self.participant_count == 0 then return 0.0 end
    self.youth_participation_pct = youth_count / self.participant_count * 100.0
    return self.youth_participation_pct
end

function CommunityEngagementPlatform:complete_indigenous_consultation(initiative_id, now_ns)
    for i = 1, self.initiative_count do
        if self.initiatives[i].initiative_id == initiative_id then
            self.initiatives[i].indigenous_consultation_completed = true
            self.indigenous_consultation_completed_count = self.indigenous_consultation_completed_count + 1
            return true
        end
    end
    return false
end

function CommunityEngagementPlatform:get_platform_status(now_ns)
    local active_initiatives = 0
    local voting_initiatives = 0
    for i = 1, self.initiative_count do
        if self.initiatives[i].status == InitiativeStatus.ACTIVE then active_initiatives = active_initiatives + 1 end
        if self.initiatives[i].status == InitiativeStatus.VOTING then voting_initiatives = voting_initiatives + 1 end
    end
    local active_participants = 0
    for i = 1, self.participant_count do
        if self.participants[i].active then active_participants = active_participants + 1 end
    end
    self:compute_participation_rate()
    self:compute_youth_participation()
    return {
        platform_id = self.platform_id,
        city_code = self.city_code,
        region = self.region,
        total_initiatives = self.initiative_count,
        active_initiatives = active_initiatives,
        voting_initiatives = voting_initiatives,
        approved_initiatives = self.total_initiatives_approved,
        total_participants = self.participant_count,
        active_participants = active_participants,
        total_votes_cast = self.total_votes_cast,
        total_discussion_threads = self.thread_count,
        average_participation_rate = self.average_participation_rate,
        youth_participation_pct = self.youth_participation_pct,
        indigenous_consultation_count = self.indigenous_consultation_count,
        indigenous_consultation_completed = self.indigenous_consultation_completed_count,
        last_reporting_ns = self.last_reporting_ns,
        last_update_ns = now_ns
    }
end

function CommunityEngagementPlatform:compute_civic_engagement_index()
    local participation_score = self:compute_participation_rate()
    local approval_rate = self.total_initiatives_approved / self.total_initiatives_proposed.max(1)
    local indigenous_compliance = self.indigenous_consultation_completed_count / 
                                  self.indigenous_consultation_count.max(1)
    local youth_score = self.youth_participation_pct / 100.0
    return (participation_score * 0.35 + approval_rate * 0.25 + 
            indigenous_compliance * 0.25 + youth_score * 0.15)
end

return {
    CommunityEngagementPlatform = CommunityEngagementPlatform,
    CommunityInitiative = CommunityInitiative,
    Participant = Participant,
    Vote = Vote,
    DiscussionThread = DiscussionThread,
    InitiativeType = InitiativeType,
    InitiativeStatus = InitiativeStatus,
    VotingMethod = VotingMethod,
    VERSION = ENGAGEMENT_PLATFORM_VERSION,
}
