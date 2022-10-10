export type NewMailCallback = () => void;

export class WebSocketApi {
    private ws: WebSocket;
    private messageQueue: string[] = [];
    private closed: boolean = false;
    private ready: boolean = false;
    private listeningForNewMail: boolean = false;
    private newMailCallbacks: Map<number, NewMailCallback> = new Map();

    private static NEXT_ID = 0;

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

    private send(message: WsApiMessage) {
        const messageString = JSON.stringify(message);
        if (!this.ready) {
            this.messageQueue.push(messageString);
            return;
        }
        this.ws.send(messageString);
    }

    private onSocketOpen(event: Event) {
        this.ready = true;
        console.debug('socket ready');

        for (let message of this.messageQueue) {
            console.debug('sending queued socket message', { message });
            this.ws.send(message);
        }
        this.messageQueue.length = 0;
    }

    private onSocketMessage(event: MessageEvent) {
        console.debug('received socket message', { event });
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
}

interface ListenForNewMailData {
    type: MessageType.ListenForNewMail;
}

type WsApiMessage = ListenForNewMailData;