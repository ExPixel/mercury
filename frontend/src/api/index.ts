import React, { useContext } from "react";
import { RawMailListItem } from "./raw-response";
import { MailListItem } from "./response";
import { NewMailCallback, WebSocketApi } from "./socket";

export default class Mercury {
    private origin: string;
    private socket: WebSocketApi | null = null;

    constructor(origin: string) {
        while (origin.endsWith('/')) {
            origin = origin.substring(0, origin.length - 1);
        }
        this.origin = origin;
        this.socket = null;
    }

    public async getMailList(params?: MailListParams): Promise<MailListItem[]> {
        const rawList: RawMailListItem[] = await this.get('/mail', params);
        return rawList.map((raw) => new MailListItem(raw));
    }

    public listenForNewMail(callback: NewMailCallback): number {
        const socket = this.ensureWebSocketConnection();
        return socket.listenForNewMail(callback);
    }

    public removeListener(listenerId: number) {
        if (this.socket) {
            this.socket.removeListener(listenerId);
        }
    }

    private async get(path: string, query?: URLSearchParams | Record<string, string | number | boolean>): Promise<any> {
        const response = await fetch(this.getUrl(path, query), { mode: 'cors' });
        if (response.ok) {
            return await response.json();
        } else {
            throw new APIError(response.status, await response.text());
        }
    }

    private ensureWebSocketConnection(): WebSocketApi {
        if (this.socket && this.socket.isOpen) {
            return this.socket;
        }
        const webSocket = new WebSocket('ws://' + this.origin + '/listen');
        this.socket = new WebSocketApi(webSocket);
        return this.socket;
    }

    private getUrl(path: string, query?: URLSearchParams | Record<string, string | number | boolean>): string {
        let url: string;
        if (!path.startsWith('/')) {
            url = this.origin + '/' + path;
        } else {
            url = this.origin + path;
        }

        let queryString: string;
        if (query instanceof URLSearchParams) {
            queryString = query.toString();
        } else if (typeof query === 'object') {
            const queryObject = new URLSearchParams();
            for (const key of Object.keys(query)) {
                queryObject.append(key, encodeURIComponent(query[key]));
            }
            queryString = queryObject.toString();
        } else {
            queryString = '';
        }

        if (queryString.length > 0) {
            url += '?' + queryString;
        }

        return window.location.protocol + '//' + url;
    }
}

export class APIError extends Error {
    #statusCode: number;

    constructor(statusCode: number, message: string) {
        super(message);
        this.#statusCode = statusCode;
    }

    public get statusCode(): number {
        return this.#statusCode;
    }
}

export interface MailListParams extends Record<string, string | number | boolean> {
    before?: number;
    after?: number;
    max?: number;
}

export const MercuryContext = React.createContext(new Mercury('localhost:8080/api'));
export function useMercury(): Mercury {
    return useContext(MercuryContext);
}