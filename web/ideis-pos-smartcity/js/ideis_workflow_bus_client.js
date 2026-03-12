import { IdeisPosGatewayClient } from "./ideis_pos_gateway_client.js";

export class IdeisWorkflowBusClient {
    constructor(wsUrl, sessionProvider, manifestProvider) {
        this.posClient = new IdeisPosGatewayClient(wsUrl, sessionProvider);
        this.manifestProvider = manifestProvider;
    }

    async posSale(vnodePath, event) {
        const manifest = await this.manifestProvider(vnodePath);
        const payload = {
            ...event,
            policy_profile: `${vnodePath}::POS`,
        };
        await this.posClient.sendPosNonFinancial("POS_SALE", {
            manifest,
            event: payload,
        });
    }

    async inventorySync(vnodePath, event) {
        const manifest = await this.manifestProvider(vnodePath);
        const payload = {
            ...event,
            policy_profile: `${vnodePath}::INVENTORY`,
        };
        await this.posClient.sendPosNonFinancial("INVENTORY_SYNC", {
            manifest,
            event: payload,
        });
    }
}
