package city.qpu.datashard.workflows

import city.qpu.datashard.pos.IdeisSmartCityClient
import city.qpu.datashard.pos.IdeisSessionToken
import city.qpu.datashard.pos.PosNonFinancialEvent
import city.qpu.datashard.pos.IdeisResponse

data class VNodePath(
    val path: String,
    val deviceType: String,
)

class SmartCityWorkflowBus(
    private val posClient: IdeisSmartCityClient,
    private val manifestProvider: (VNodePath) -> ByteArray,
) {

    fun posSale(
        vnode: VNodePath,
        token: IdeisSessionToken,
        event: PosNonFinancialEvent,
    ): IdeisResponse {
        val manifest = manifestProvider(vnode)
        return posClient.sendPosSale(token, manifest, event)
    }

    fun inventorySync(
        vnode: VNodePath,
        token: IdeisSessionToken,
        event: PosNonFinancialEvent,
    ): IdeisResponse {
        val manifest = manifestProvider(vnode)
        return posClient.inventorySync(token, manifest, event)
    }
}
