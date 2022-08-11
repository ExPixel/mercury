export interface EmailPath {
    name: string;
    address: string;
}

const emailPathRegex = /([^<>]+)?\s*(<[^<>]+>)?/;
export function parseValidEmailPath(address: string): EmailPath {
    const matches = address.match(emailPathRegex);
    if (!matches) {
        throw new Error("invalid email path");
    }
    return { name: matches[1], address: matches[2] };
}