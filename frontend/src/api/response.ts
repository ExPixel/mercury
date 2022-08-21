import { RawMailbox, RawMailListItem, RawMailMetadata } from "./raw-response";
import { parseISO } from 'date-fns';

export class MailListItem {
    public id: number;
    public createdAt: Date;
    public from: Mailbox[];
    #sender?: Mailbox;

    constructor(raw: RawMailListItem) {
        this.id = raw.id;
        this.createdAt = parseISO(raw.created_at);
        this.from = raw.from ? raw.from.map(f => new Mailbox(f)) : [];
        this.#sender = raw.sender ? new Mailbox(raw.sender) : undefined;
    }

    public get sender(): Mailbox {
        if (this.#sender === undefined) {
            console.assert(this.from.length > 0, "no sender or from");
            return this.from[0];
        }
        return this.#sender;
    }
}

export class MailMetadata {
    public from: string;
    public to: string[];

    constructor(raw: RawMailMetadata) {
        this.to = raw.to;
        this.from = raw.from;
    }
}

export class Mailbox {
    public displayName: string | null;
    public address: string;

    constructor(raw: RawMailbox) {
        this.displayName = raw.display_name || null;
        this.address = raw.address;
    }

    public get displayNameOrAddress(): string {
        return this.displayName === null ? this.address : this.displayName;
    }
}