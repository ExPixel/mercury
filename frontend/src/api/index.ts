import React, { useContext } from "react";

export default class Mercury {
    private origin: string;

    constructor(origin: string) {
        while (origin.endsWith('/')) {
            origin = origin.substring(0, origin.length - 1);
        }
        this.origin = origin;
    }

    private getUrl(path: string): string {
        if (!path.startsWith('/')) {
            return this.origin + '/' + path;
        } else {
            return this.origin + path;
        }
    }
}

export const MercuryContext = React.createContext(new Mercury('http://localhost:8080'));
export function useMercury(): Mercury {
    return useContext(MercuryContext);
}