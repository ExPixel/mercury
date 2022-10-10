export type NewMailCallback = () => void;

export class WebSocketApi {
    private ws: WebSocket;
    private messageQueue: string[] = [];
    private closed: boolean = false;
    private ready: boolean = false;
    private listeningForNewMail: boolean = false;
    private heartbeatIntervalId: number = -1;
    private newMailCallbacks: Map<number, NewMailCallback> = new Map();

    private static NEXT_ID = 0;
    private static readonly HEARTBEAT_INTERVAL_MS = 30_000;

    constructor(ws: WebSocket) {
        this.ws = ws;
        this.initializeSocketListeners();
    }

    public listenForNewMail(callback: NewMailCallback): number {
        const id = ++WebSocketApi.NEXT_ID;
        this.newMailCallbacks.set(id, callback);

        if (this.listeningForNewMail) {
            return id;
        }

        this.send({ type: MessageType.ListenForNewMail });
        this.listeningForNewMail = true;
        return id;
    }

    public removeListener(listenerId: number) {
        this.newMailCallbacks.delete(listenerId);
    }

    private initializeSocketListeners() {
        this.ws.addEventListener('open', this.onSocketOpen.bind(this));
        this.ws.addEventListener('message', this.onSocketMessage.bind(this));
        this.ws.addEventListener('close', this.onSocketClose.bind(this));
        this.ws.addEventListener('error', this.onSocketError.bind(this));
    }

    private startHeartbeat() {
        if (this.heartbeatIntervalId >= 0) {
            return;
        }
        console.debug('starting websocket heartbeat');
        this.heartbeatIntervalId = setInterval(this.heartbeatTick.bind(this), WebSocketApi.HEARTBEAT_INTERVAL_MS);
    }

    private refreshHeartbeat() {
        if (this.heartbeatIntervalId < 0) {
            return;
        }
        clearInterval(this.heartbeatIntervalId);
        this.heartbeatIntervalId = -1;
        this.heartbeatIntervalId = setInterval(this.heartbeatTick.bind(this), WebSocketApi.HEARTBEAT_INTERVAL_MS);
    }

    private stopHeartbeat() {
        if (this.heartbeatIntervalId < 0) {
            return;
        }
        console.debug('stopping websocket heartbeat');
        clearInterval(this.heartbeatIntervalId);
        this.heartbeatIntervalId = -1;
    }

    private heartbeatTick() {
        if (!this.ready) { return; } // don't want this queued
        this.send({ type: MessageType.Heartbeat });
    }

    private send(message: WsApiMessage) {
        const messageString = JSON.stringify(message);
        if (!this.ready) {
            this.messageQueue.push(messageString);
            return;
        }
        this.ws.send(messageString);

        if (message.type !== MessageType.Heartbeat) {
            this.refreshHeartbeat();
        }
    }

    private onSocketOpen(event: Event) {
        this.ready = true;
        console.debug('socket ready');

        for (let message of this.messageQueue) {
            console.debug('sending queued socket message', { message });
            this.ws.send(message);
        }
        this.messageQueue.length = 0;

        this.startHeartbeat();
    }

    private onSocketMessage(event: MessageEvent) {
        console.debug('received socket message', { event });
        this.refreshHeartbeat();
    }

    private onSocketClose(event: CloseEvent) {
        this.uninit();
        console.debug('socket closed', { event });
    }

    private onSocketError(event: Event) {
        this.uninit();
        console.error('socket error: ', event);
    }

    private uninit() {
        this.closed = true;
        this.ready = false;
        this.listeningForNewMail = false;
        this.stopHeartbeat();
    }

    get isReady(): boolean {
        return this.ready;
    }

    get isOpen(): boolean {
        return !this.closed;
    }
}

enum MessageType {
    ListenForNewMail = "ListenForNewMail",
    Heartbeat = "Heartbeat",
}

interface ListenForNewMailData {
    type: MessageType.ListenForNewMail;
}

interface Heartbeat {
    type: MessageType.Heartbeat;
}

type WsApiMessage = ListenForNewMailData | Heartbeat;