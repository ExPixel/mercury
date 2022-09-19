import React, { useContext } from "react";
import { RawMailListItem } from "./raw-response";
import { MailListItem } from "./response";

export default class Mercury {
    private origin: string;

    constructor(origin: string) {
        while (origin.endsWith('/')) {
            origin = origin.substring(0, origin.length - 1);
        }
        this.origin = origin;
    }

    public async getMailList(): Promise<MailListItem[]> {
        const rawList: RawMailListItem[] = await this.get('/mail');
        return rawList.map((raw) => new MailListItem(raw));
    }

    private async get(path: string, query?: URLSearchParams | Record<string, string | number | boolean>): Promise<any> {
        const response = await fetch(this.getUrl(path, query), { mode: 'cors' });
        if (response.ok) {
            return await response.json();
        } else {
            throw new APIError(response.status, await response.text());
        }
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

        return url;
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

export const MercuryContext = React.createContext(new Mercury('http://localhost:8080/api'));
export function useMercury(): Mercury {
    return useContext(MercuryContext);
}