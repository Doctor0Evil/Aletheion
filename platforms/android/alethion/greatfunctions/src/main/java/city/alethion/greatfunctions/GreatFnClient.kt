package city.alethion.greatfunctions

data class GreatFnId(
    val city: Int,
    val district: Int,
    val block: Int,
    val lane: Int
) {
    override fun toString(): String = "$city-$district-$block-$lane"
}

data class GreatFnCall(
    val target: GreatFnId,
    val cityZone: Int,
    val epoch: Long,
    val payloadScalar: Long? = null,
    val payloadVector: IntArray? = null,
    val payloadGrid: ShortArray? = null,
    val priority: Int = 0
)

interface GreatFnGateway {
    fun route(call: GreatFnCall)
    fun tick()
}

class InMemoryGreatFnGateway : GreatFnGateway {
    private val queue: ArrayDeque<GreatFnCall> = ArrayDeque()

    override fun route(call: GreatFnCall) {
        queue.addLast(call)
    }

    override fun tick() {
        var processed = 0
        while (queue.isNotEmpty() && processed < 64) {
            queue.removeFirst()
            processed++
        }
    }

    fun queueSize(): Int = queue.size
}
