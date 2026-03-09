package aletheion.services.citizen.assistant

const val PERSONAL_ASSISTANT_VERSION = 20260310L
const val MAX_TASKS_PER_CITIZEN = 1024
const val MAX_REMINDERS_PER_CITIZEN = 2048
const val MAX_GOALS_PER_CITIZEN = 256
const val MAX_CITIZENS = 524288

enum class TaskPriority {
    LOW(1), NORMAL(2), HIGH(3), URGENT(4), CRITICAL(5)
    val level: Int
    constructor(level: Int) { this.level = level }
}

enum class TaskCategory {
    HEALTH, FINANCE, TRANSPORTATION, HOUSING, EDUCATION,
    EMPLOYMENT, UTILITIES, GOVERNANCE, COMMUNITY, PERSONAL
}

enum class TaskStatus {
    PENDING, IN_PROGRESS, COMPLETED, CANCELLED, DEFERRED, RECURRING
}

data class Task(
    val taskId: ULong,
    val citizenDid: String,
    val category: TaskCategory,
    val priority: TaskPriority,
    val title: String,
    val description: String,
    val createdAtNs: Long,
    val dueAtNs: Long?,
    val completedAtNs: Long?,
    val status: TaskStatus,
    val recurring: Boolean,
    val recurrencePattern: String?,
    val reminderIds: List<ULong>,
    val relatedGoalId: ULong?,
    val accessibilityAccommodations: List<String>,
    val indigenousCulturalConsiderations: Boolean,
    val privacyLevel: UInt
) {
    fun isOverdue(nowNs: Long): Boolean = 
        dueAtNs != null && nowNs > dueAtNs && status != TaskStatus.COMPLETED
    fun isDueToday(nowNs: Long): Boolean = 
        dueAtNs != null && Math.abs(dueAtNs - nowNs) < 86400000000000L
}

data class Reminder(
    val reminderId: ULong,
    val taskId: ULong,
    val citizenDid: String,
    val reminderTimeNs: Long,
    val delivered: Boolean,
    val deliveredAtNs: Long?,
    val acknowledged: Boolean,
    val acknowledgedAtNs: Long?,
    val reminderType: String,
    val channel: String,
    val repeatCount: UInt,
    val lastRemindedNs: Long?
)

data class Goal(
    val goalId: ULong,
    val citizenDid: String,
    val goalName: String,
    val goalDescription: String,
    val category: TaskCategory,
    val targetValue: Double,
    val currentValue: Double,
    val unit: String,
    val startDateNs: Long,
    val targetDateNs: Long,
    val completed: Boolean,
    val completedAtNs: Long?,
    val relatedTaskIds: List<ULong>,
    val milestoneCount: UInt,
    val milestonesCompleted: UInt,
    val privacyLevel: UInt
) {
    fun progressPct(): Double = 
        if (targetValue == 0.0) 0.0 else (currentValue / targetValue * 100.0).coerceIn(0.0, 100.0)
    fun isOnTrack(nowNs: Long): Boolean {
        val elapsed = nowNs - startDateNs
        val total = targetDateNs - startDateNs
        if (total == 0L) return false
        val expectedProgress = elapsed.toDouble() / total.toDouble() * targetValue
        return currentValue >= expectedProgress * 0.8
    }
}

class PersonalAssistantTaskManager(
    private val managerId: ULong,
    private val cityCode: String,
    private val initTimestampNs: Long
) {
    private val tasks = mutableMapOf<ULong, Task>()
    private val reminders = mutableMapOf<ULong, Reminder>()
    private val goals = mutableMapOf<ULong, Goal>()
    private val citizenTaskCounts = mutableMapOf<String, UInt>()
    private val auditLog = mutableListOf<AssistantAuditEntry>()
    private var nextTaskId: ULong = 1UL
    private var nextReminderId: ULong = 1UL
    private var nextGoalId: ULong = 1UL
    private var totalTasksCreated: ULong = 0UL
    private var totalTasksCompleted: ULong = 0UL
    private var totalRemindersDelivered: ULong = 0UL
    private var averageCompletionTimeH: Double = 0.0
    private var citizenSatisfactionScore: Double = 0.0
    
    data class AssistantAuditEntry(
        val entryId: ULong,
        val action: String,
        val taskId: ULong?,
        val timestampNs: Long,
        val success: Boolean,
        val details: String,
        val riskScore: Double
    )
    
    fun createTask(task: Task, nowNs: Long): Result<ULong> {
        val currentCount = citizenTaskCounts[task.citizenDid] ?: 0U
        if (currentCount >= MAX_TASKS_PER_CITIZEN.toULong()) {
            logAudit("TASK_CREATE", null, nowNs, false, "Task limit exceeded for citizen", 0.3)
            return Result.failure(Error("TASK_LIMIT_EXCEEDED"))
        }
        tasks[nextTaskId] = task
        citizenTaskCounts[task.citizenDid] = currentCount + 1U
        totalTasksCreated++
        logAudit("TASK_CREATE", nextTaskId, nowNs, true, "Task created: ${task.title}", 0.02)
        val taskId = nextTaskId
        nextTaskId++
        return Result.success(taskId)
    }
    
    fun completeTask(taskId: ULong, nowNs: Long): Result<Unit> {
        val task = tasks[taskId] ?: return Result.failure(Error("TASK_NOT_FOUND"))
        val updatedTask = task.copy(
            status = TaskStatus.COMPLETED,
            completedAtNs = nowNs
        )
        tasks[taskId] = updatedTask
        totalTasksCompleted++
        val completionTime = (nowNs - task.createdAtNs) / 3600000000000
        updateAverageCompletionTime(completionTime.toDouble())
        logAudit("TASK_COMPLETE", taskId, nowNs, true, "Task completed", 0.01)
        return Result.success(Unit)
    }
    
    fun createReminder(reminder: Reminder): Result<ULong> {
        val task = tasks[reminder.taskId] ?: return Result.failure(Error("TASK_NOT_FOUND"))
        val citizenCount = reminders.count { it.value.citizenDid == task.citizenDid }
        if (citizenCount >= MAX_REMINDERS_PER_CITIZEN) {
            return Result.failure(Error("REMINDER_LIMIT_EXCEEDED"))
        }
        reminders[nextReminderId] = reminder
        val reminderId = nextReminderId
        nextReminderId++
        return Result.success(reminderId)
    }
    
    fun deliverReminder(reminderId: ULong, nowNs: Long): Result<Unit> {
        val reminder = reminders[reminderId] ?: return Result.failure(Error("REMINDER_NOT_FOUND"))
        val updatedReminder = reminder.copy(
            delivered = true,
            deliveredAtNs = nowNs,
            lastRemindedNs = nowNs
        )
        reminders[reminderId] = updatedReminder
        totalRemindersDelivered++
        return Result.success(Unit)
    }
    
    fun acknowledgeReminder(reminderId: ULong, nowNs: Long): Result<Unit> {
        val reminder = reminders[reminderId] ?: return Result.failure(Error("REMINDER_NOT_FOUND"))
        val updatedReminder = reminder.copy(
            acknowledged = true,
            acknowledgedAtNs = nowNs
        )
        reminders[reminderId] = updatedReminder
        return Result.success(Unit)
    }
    
    fun createGoal(goal: Goal): Result<ULong> {
        val citizenCount = goals.count { it.value.citizenDid == goal.citizenDid }
        if (citizenCount >= MAX_GOALS_PER_CITIZEN) {
            return Result.failure(Error("GOAL_LIMIT_EXCEEDED"))
        }
        goals[nextGoalId] = goal
        val goalId = nextGoalId
        nextGoalId++
        return Result.success(goalId)
    }
    
    fun updateGoalProgress(goalId: ULong, newValue: Double, nowNs: Long): Result<Unit> {
        val goal = goals[goalId] ?: return Result.failure(Error("GOAL_NOT_FOUND"))
        val updatedGoal = goal.copy(currentValue = newValue)
        goals[goalId] = updatedGoal
        if (newValue >= goal.targetValue && !goal.completed) {
            val completedGoal = updatedGoal.copy(
                completed = true,
                completedAtNs = nowNs
            )
            goals[goalId] = completedGoal
        }
        return Result.success(Unit)
    }
    
    fun getOverdueTasks(citizenDid: String, nowNs: Long): List<Task> {
        return tasks.values.filter { 
            it.citizenDid == citizenDid && it.isOverdue(nowNs) 
        }.sortedBy { it.priority.level }
    }
    
    fun getDueTodayTasks(citizenDid: String, nowNs: Long): List<Task> {
        return tasks.values.filter { 
            it.citizenDid == citizenDid && it.isDueToday(nowNs) 
        }.sortedBy { it.priority.level }
    }
    
    private fun updateAverageCompletionTime(newTimeH: Double) {
        averageCompletionTimeH = (averageCompletionTimeH * (totalTasksCompleted - 1).toDouble() + newTimeH) / 
            totalTasksCompleted.toDouble().max(1.0)
    }
    
    fun getAssistantStatus(nowNs: Long): AssistantStatus {
        val activeTasks = tasks.count { it.value.status == TaskStatus.PENDING || it.value.status == TaskStatus.IN_PROGRESS }
        val completedTasks = tasks.count { it.value.status == TaskStatus.COMPLETED }
        val pendingReminders = reminders.count { !it.value.delivered }
        val activeGoals = goals.count { !it.value.completed }
        val onTrackGoals = goals.count { it.value.isOnTrack(nowNs) }
        return AssistantStatus(
            managerId = managerId,
            cityCode = cityCode,
            totalTasks = tasks.size,
            activeTasks = activeTasks,
            completedTasks = completedTasks,
            totalReminders = reminders.size,
            pendingReminders = pendingReminders,
            deliveredReminders = totalRemindersDelivered,
            totalGoals = goals.size,
            activeGoals = activeGoals,
            onTrackGoals = onTrackGoals,
            totalTasksCreated = totalTasksCreated,
            totalTasksCompleted = totalTasksCompleted,
            averageCompletionTimeH = averageCompletionTimeH,
            citizenSatisfactionScore = citizenSatisfactionScore,
            lastUpdateNs = nowNs
        )
    }
    
    fun computeProductivityIndex(): Double {
        val completionRate = totalTasksCompleted.toDouble() / totalTasksCreated.max(1UL).toDouble()
        val timelinessScore = if (averageCompletionTimeH < 24.0) 1.0 else (24.0 / averageCompletionTimeH).coerceAtMost(1.0)
        val goalProgressScore = goals.values.map { it.progressPct() }.average() / 100.0
        return (completionRate * 0.4 + timelinessScore * 0.3 + goalProgressScore * 0.3).coerceIn(0.0, 1.0)
    }
    
    private fun logAudit(action: String, taskId: ULong?, timestampNs: Long, 
                        success: Boolean, details: String, riskScore: Double) {
        val entry = AssistantAuditEntry(
            entryId = auditLog.size.toULong(),
            action = action,
            taskId = taskId,
            timestampNs = timestampNs,
            success = success,
            details = details,
            riskScore = riskScore
        )
        auditLog.add(entry)
    }
    
    fun getAuditTrail(fromNs: Long, toNs: Long): List<AssistantAuditEntry> {
        return auditLog.filter { it.timestampNs in fromNs..toNs }
    }
}

data class AssistantStatus(
    val managerId: ULong,
    val cityCode: String,
    val totalTasks: Int,
    val activeTasks: Int,
    val completedTasks: Int,
    val totalReminders: Int,
    val pendingReminders: Int,
    val deliveredReminders: ULong,
    val totalGoals: Int,
    val activeGoals: Int,
    val onTrackGoals: Int,
    val totalTasksCreated: ULong,
    val totalTasksCompleted: ULong,
    val averageCompletionTimeH: Double,
    val citizenSatisfactionScore: Double,
    val lastUpdateNs: Long
) {
    fun taskManagementEfficiency(): Double {
        val completionRate = completedTasks.toDouble() / totalTasks.max(1).toDouble()
        val reminderDeliveryRate = deliveredReminders.toDouble() / totalReminders.max(1).toDouble()
        val goalOnTrackRate = onTrackGoals.toDouble() / activeGoals.max(1).toDouble()
        return (completionRate * 0.4 + reminderDeliveryRate.coerceAtMost(1.0) * 0.3 + 
                goalOnTrackRate.coerceAtMost(1.0) * 0.3).coerceIn(0.0, 1.0)
    }
}

fun createPhoenixPersonalAssistant(managerId: ULong, nowNs: Long): PersonalAssistantTaskManager {
    return PersonalAssistantTaskManager(managerId, "PHOENIX_AZ", nowNs)
}
