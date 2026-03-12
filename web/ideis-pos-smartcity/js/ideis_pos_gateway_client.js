export class IdeisPosGatewayClient {
    constructor(wsUrl, sessionProvider) {
        this.wsUrl = wsUrl;
        this.sessionProvider = sessionProvider;
        this.socket = null;
    }

    async connect() {
        if (this.socket && this.socket.readyState === WebSocket.OPEN) return;
        this.socket = new WebSocket(this.wsUrl);
        return new Promise((resolve, reject) => {
            this.socket.onopen = () => resolve();
            this.socket.onerror = (e) => reject(e);
        });
    }

    async sendPosNonFinancial(action, payload) {
        await this.connect();
        const token = await this.sessionProvider();
        const msg = {
            kind: "IDEIS_POS_NONFINANCIAL",
            action,
            token,
            payload,
        };
        this.socket.send(JSON.stringify(msg));
    }

    close() {
        if (this.socket) {
            this.socket.close();
        }
    }
}
