package aletheion.education.adaptive.learning

const val LEARNING_PLATFORM_VERSION = 20260310L
const val MAX_LEARNERS = 524288
const val MAX_COURSES = 16384
const val MAX_LEARNING_PATHS = 65536
const val MAX_ASSESSMENTS = 1048576
const val TARGET_COMPLETION_RATE = 0.85
const val MIN_ACCESSIBILITY_SCORE = 0.95

enum class LearningLevel {
    BEGINNER(1), INTERMEDIATE(2), ADVANCED(3), EXPERT(4), MASTER(5)
    val level: Int
    constructor(level: Int) { this.level = level }
}

enum class LearningModality {
    SELF_PACED, INSTRUCTOR_LED, HYBRID, COMPETENCY_BASED,
    PROJECT_BASED, APPRENTICESHIP, VIRTUAL_REALITY, AUGMENTED_REALITY
}

enum class AssessmentType {
    DIAGNOSTIC, FORMATIVE, SUMMATIVE, PERFORMANCE, PORTFOLIO,
    PEER_REVIEW, SELF_ASSESSMENT, PRACTICAL_EXAM, WRITTEN_EXAM
}

data class LearningCourse(
    val courseId: ULong,
    val courseName: String,
    val category: String,
    val level: LearningLevel,
    val modality: LearningModality,
    val estimatedHours: UInt,
    val credits: UInt,
    val prerequisites: List<ULong>,
    val learningObjectives: List<String>,
    val accessibilityCompliant: Boolean,
    val indigenousKnowledgeIntegrated: Boolean,
    val multilingualSupport: List<String>,
    val instructorId: ULong?,
    val enrollmentCount: UInt,
    val completionRate: Double,
    val averageRating: Double,
    val createdAtNs: Long,
    val updatedAtNs: Long,
    val active: Boolean
) {
    fun isAccessible(): Boolean = accessibilityCompliant && averageRating >= 3.5
    fun hasCapacity(): Boolean = enrollmentCount < 1000U
    fun meetsCompletionTarget(): Boolean = completionRate >= TARGET_COMPLETION_RATE
}

data class LearningPath(
    val pathId: ULong,
    val pathName: String,
    val careerTrack: String,
    val targetRole: String,
    val courseIds: List<ULong>,
    val totalHours: UInt,
    val totalCredits: UInt,
    val estimatedCompletionMonths: UInt,
    val difficultyLevel: LearningLevel,
    val industryCertifications: List<String>,
    val employerPartners: List<String>,
    val indigenousSkillIntegration: Boolean,
    val accessibilityCompliant: Boolean,
    val enrollmentCount: UInt,
    val completionCount: UInt,
    val jobPlacementRate: Double,
    val createdAtNs: Long,
    val updatedAtNs: Long
) {
    fun completionRate(): Double = 
        if (enrollmentCount == 0U) 0.0 else completionCount.toDouble() / enrollmentCount.toDouble()
    fun isOnTrack(): Boolean = completionRate() >= TARGET_COMPLETION_RATE
}

data class LearnerProfile(
    val learnerId: ULong,
    val citizenDid: String,
    val age: UInt,
    val educationLevel: String,
    val currentSkillLevel: LearningLevel,
    val targetSkillLevel: LearningLevel,
    val learningStyle: String,
    val accessibilityNeeds: List<String>,
    val languagePreference: String,
    val indigenousCommunity: Boolean,
    val lowIncomeStatus: Boolean,
    val disabilityStatus: Boolean,
    val enrolledCourses: List<ULong>,
    val completedCourses: List<ULong>,
    val learningPathId: ULong?,
    val totalLearningHours: UInt,
    val skillGaps: List<String>,
    val careerGoals: List<String>,
    val lastActivityNs: Long,
    val createdAtNs: Long
) {
    fun progressPct(): Double = 
        if (enrolledCourses.isEmpty()) 0.0 else 
        completedCourses.size.toDouble() / (enrolledCourses.size + completedCourses.size).toDouble() * 100.0
    fun requiresSupport(): Boolean = 
        accessibilityNeeds.isNotEmpty() || lowIncomeStatus || disabilityStatus
}

data class Assessment(
    val assessmentId: ULong,
    val courseId: ULong,
    val learnerId: ULong,
    val assessmentType: AssessmentType,
    val title: String,
    val maxScore: Double,
    val achievedScore: Double?,
    val submittedAtNs: Long?,
    val gradedAtNs: Long?,
    val feedback: String?,
    val retakeCount: UInt,
    val maxRetakes: UInt,
    val proctored: Boolean,
    val accessibilityAccommodations: List<String>,
    val indigenousKnowledgeAssessed: Boolean,
    val passed: Boolean
) {
    fun isComplete(): Boolean = submittedAtNs != null && gradedAtNs != null
    fun isPassing(): Boolean = passed || (achievedScore ?: 0.0) >= (maxScore * 0.7)
    fun canRetake(): Boolean = retakeCount < maxRetakes && !passed
}

class AdaptiveLearningPlatform(
    private val platformId: ULong,
    private val cityCode: String,
    private val initTimestampNs: Long
) {
    private val courses = mutableMapOf<ULong, LearningCourse>()
    private val learningPaths = mutableMapOf<ULong, LearningPath>()
    private val learners = mutableMapOf<ULong, LearnerProfile>()
    private val assessments = mutableMapOf<ULong, Assessment>()
    private val auditLog = mutableListOf<LearningAuditEntry>()
    private var nextCourseId: ULong = 1UL
    private var nextPathId: ULong = 1UL
    private var nextLearnerId: ULong = 1UL
    private var nextAssessmentId: ULong = 1UL
    private var totalEnrollments: ULong = 0UL
    private var totalCompletions: ULong = 0UL
    private var averageCompletionRate: Double = 0.0
    private var platformAccessibilityScore: Double = 1.0
    private var indigenousParticipationRate: Double = 0.0
    
    data class LearningAuditEntry(
        val entryId: ULong,
        val action: String,
        val courseId: ULong?,
        val learnerId: ULong?,
        val timestampNs: Long,
        val success: Boolean,
        val details: String,
        val riskScore: Double
    )
    
    fun registerCourse(course: LearningCourse): Result<ULong> {
        if (courses.size >= MAX_COURSES.toULong()) {
            logAudit("COURSE_REGISTER", null, null, initTimestampNs, false, "Course limit exceeded", 0.3)
            return Result.failure(Error("COURSE_LIMIT_EXCEEDED"))
        }
        if (!course.accessibilityCompliant) {
            logAudit("COURSE_REGISTER", null, null, initTimestampNs, false, "Accessibility compliance required", 0.2)
            return Result.failure(Error("ACCESSIBILITY_COMPLIANCE_REQUIRED"))
        }
        courses[nextCourseId] = course
        logAudit("COURSE_REGISTER", nextCourseId, null, initTimestampNs, true, "Course registered: ${course.courseName}", 0.02)
        val courseId = nextCourseId
        nextCourseId++
        return Result.success(courseId)
    }
    
    fun createLearningPath(path: LearningPath): Result<ULong> {
        if (learningPaths.size >= MAX_LEARNING_PATHS.toULong()) {
            return Result.failure(Error("PATH_LIMIT_EXCEEDED"))
        }
        if (!path.accessibilityCompliant) {
            return Result.failure(Error("ACCESSIBILITY_COMPLIANCE_REQUIRED"))
        }
        learningPaths[nextPathId] = path
        val pathId = nextPathId
        nextPathId++
        logAudit("PATH_CREATE", null, null, initTimestampNs, true, "Learning path created: ${path.pathName}", 0.05)
        return Result.success(pathId)
    }
    
    fun enrollLearner(learner: LearnerProfile, courseId: ULong, nowNs: Long): Result<Unit> {
        if (learners.size >= MAX_LEARNERS.toULong()) {
            return Result.failure(Error("LEARNER_LIMIT_EXCEEDED"))
        }
        val course = courses[courseId] ?: return Result.failure(Error("COURSE_NOT_FOUND"))
        if (!course.hasCapacity()) {
            return Result.failure(Error("COURSE_FULL"))
        }
        if (!course.isAccessible() && learner.accessibilityNeeds.isNotEmpty()) {
            return Result.failure(Error("ACCESSIBILITY_MISMATCH"))
        }
        val updatedLearner = learner.copy(
            enrolledCourses = learner.enrolledCourses + courseId,
            lastActivityNs = nowNs
        )
        learners[nextLearnerId] = updatedLearner
        totalEnrollments++
        val learnerId = nextLearnerId
        nextLearnerId++
        logAudit("LEARNER_ENROLL", courseId, learnerId, nowNs, true, "Learner enrolled in course", 0.05)
        return Result.success(Unit)
    }
    
    fun completeAssessment(assessment: Assessment, nowNs: Long): Result<ULong> {
        if (assessments.size >= MAX_ASSESSMENTS.toULong()) {
            return Result.failure(Error("ASSESSMENT_LIMIT_EXCEEDED"))
        }
        val learner = learners[assessment.learnerId] ?: return Result.failure(Error("LEARNER_NOT_FOUND"))
        val course = courses[assessment.courseId] ?: return Result.failure(Error("COURSE_NOT_FOUND"))
        assessments[nextAssessmentId] = assessment
        if (assessment.passed) {
            val updatedLearner = learner.copy(
                completedCourses = learner.completedCourses + assessment.courseId,
                totalLearningHours = learner.totalLearningHours + course.estimatedHours,
                lastActivityNs = nowNs
            )
            learners[assessment.learnerId] = updatedLearner
            totalCompletions++
        }
        val assessmentId = nextAssessmentId
        nextAssessmentId++
        logAudit("ASSESSMENT_COMPLETE", assessment.courseId, assessment.learnerId, nowNs, 
                assessment.passed, "Assessment completed: ${if (assessment.passed) "PASSED" else "FAILED"}", 0.02)
        return Result.success(assessmentId)
    }
    
    fun recommendLearningPath(learnerId: ULong, nowNs: Long): ULong? {
        val learner = learners[learnerId] ?: return null
        val careerGoals = learner.careerGoals
        if (careerGoals.isEmpty()) return null
        return learningPaths.values.filter { 
            it.targetRole in careerGoals &&
            it.accessibilityCompliant &&
            it.isOnTrack()
        }.maxByOrNull { it.jobPlacementRate }?.pathId
    }
    
    fun identifySkillGaps(learnerId: ULong): List<String> {
        val learner = learners[learnerId] ?: return emptyList()
        val completedCourses = learner.completedCourses.mapNotNull { courses[it] }
        val allObjectives = completedCourses.flatMap { it.learningObjectives }.toSet()
        val targetPath = learner.learningPathId?.let { learningPaths[it] }
        val requiredSkills = targetPath?.courseIds?.mapNotNull { courses[it] }?.flatMap { it.learningObjectives }?.toSet() ?: emptySet()
        return (requiredSkills - allObjectives).toList()
    }
    
    fun computePlatformMetrics(nowNs: Long) {
        val totalLearners = learners.size
        val indigenousLearners = learners.count { it.value.indigenousCommunity }
        indigenousParticipationRate = if (totalLearners > 0) {
            indigenousLearners.toDouble() / totalLearners.toDouble()
        } else 0.0
        val accessibleCourses = courses.count { it.value.accessibilityCompliant }
        platformAccessibilityScore = if (courses.isNotEmpty()) {
            accessibleCourses.toDouble() / courses.size.toDouble()
        } else 1.0
        averageCompletionRate = if (totalEnrollments > 0UL) {
            totalCompletions.toDouble() / totalEnrollments.toDouble()
        } else 0.0
    }
    
    fun getPlatformStatus(nowNs: Long): PlatformStatus {
        computePlatformMetrics(nowNs)
        val activeLearners = learners.count { 
            nowNs - it.value.lastActivityNs < 77760000000000L 
        }
        val onTrackPaths = learningPaths.count { it.value.isOnTrack() }
        val highRatedCourses = courses.count { it.value.averageRating >= 4.0 }
        return PlatformStatus(
            platformId = platformId,
            cityCode = cityCode,
            totalCourses = courses.size,
            activeCourses = courses.count { it.value.active },
            highRatedCourses,
            totalLearningPaths = learningPaths.size,
            onTrackPaths,
            totalLearners = learners.size,
            activeLearners,
            indigenousLearners = learners.count { it.value.indigenousCommunity },
            learnersWithAccessibilityNeeds = learners.count { it.value.accessibilityNeeds.isNotEmpty() },
            totalAssessments = assessments.size,
            passedAssessments = assessments.count { it.value.passed },
            totalEnrollments = totalEnrollments,
            totalCompletions = totalCompletions,
            averageCompletionRate = averageCompletionRate,
            platformAccessibilityScore = platformAccessibilityScore,
            indigenousParticipationRate = indigenousParticipationRate,
            lastUpdateNs = nowNs
        )
    }
    
    fun computeEducationEffectivenessIndex(): Double {
        val completionScore = averageCompletionRate
        val accessibilityScore = platformAccessibilityScore
        val indigenousInclusionScore = indigenousParticipationRate
        val qualityScore = courses.count { it.value.meetsCompletionTarget() }.toDouble() / courses.size.max(1).toDouble()
        return (completionScore * 0.30 + accessibilityScore * 0.25 + 
                indigenousInclusionScore * 0.25 + qualityScore * 0.20).coerceIn(0.0, 1.0)
    }
    
    private fun logAudit(action: String, courseId: ULong?, learnerId: ULong?, 
                        timestampNs: Long, success: Boolean, details: String, riskScore: Double) {
        val entry = LearningAuditEntry(
            entryId = auditLog.size.toULong(),
            action = action,
            courseId = courseId,
            learnerId = learnerId,
            timestampNs = timestampNs,
            success = success,
            details = details,
            riskScore = riskScore
        )
        auditLog.add(entry)
    }
    
    fun getAuditTrail(fromNs: Long, toNs: Long): List<LearningAuditEntry> {
        return auditLog.filter { it.timestampNs in fromNs..toNs }
    }
}

data class PlatformStatus(
    val platformId: ULong,
    val cityCode: String,
    val totalCourses: Int,
    val activeCourses: Int,
    val highRatedCourses: Int,
    val totalLearningPaths: Int,
    val onTrackPaths: Int,
    val totalLearners: Int,
    val activeLearners: Int,
    val indigenousLearners: Int,
    val learnersWithAccessibilityNeeds: Int,
    val totalAssessments: Int,
    val passedAssessments: Int,
    val totalEnrollments: ULong,
    val totalCompletions: ULong,
    val averageCompletionRate: Double,
    val platformAccessibilityScore: Double,
    val indigenousParticipationRate: Double,
    val lastUpdateNs: Long
) {
    fun educationEquityIndex(): Double {
        val indigenousAccess = indigenousLearners.toDouble() / totalLearners.max(1).toDouble()
        val accessibilityAccess = learnersWithAccessibilityNeeds.toDouble() / totalLearners.max(1).toDouble()
        val completionEquity = averageCompletionRate
        return (indigenousAccess * 0.4 + accessibilityAccess * 0.3 + completionEquity * 0.3).coerceIn(0.0, 1.0)
    }
}

fun createPhoenixLearningPlatform(platformId: ULong, nowNs: Long): AdaptiveLearningPlatform {
    return AdaptiveLearningPlatform(platformId, "PHOENIX_AZ", nowNs)
}
