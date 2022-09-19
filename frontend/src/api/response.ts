// SPDX-License-Identifier: GPL-3.0-or-later

import { RawAddress, RawAddressType, RawGroup, RawMailbox, RawMailListItem, RawMailMetadata } from "./raw-response";
import { parseISO } from 'date-fns';

export enum DisplayMode {
    Short,
    Long,
}

export class MailListItem {
    public id: number;
    public createdAt: Date;
    public from: Mailbox[];
    public to: (Mailbox | Group)[];
    public subject: string;
    #sender?: Mailbox;

    constructor(raw: RawMailListItem) {
        this.id = raw.id;
        this.createdAt = parseISO(raw.created_at);
        this.from = raw.from ? raw.from.map(f => new Mailbox(f)) : [];
        this.to = raw.to.map(t => t.type === RawAddressType.Mailbox ? new Mailbox(t) : new Group(t));
        this.subject = raw.subject;
        this.#sender = raw.sender ? new Mailbox(raw.sender) : undefined;
    }

    public get recipientDisplayString(): string {
        return this.to.reduce((acc, item) => {
            if (acc.length > 0) {
                return acc + ', ' + item.toString();
            } else {
                return item.toString();
            }
        }, '');
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

export class Mailbox extends Object {
    public displayName: string | null;
    public address: string;

    constructor(raw: RawMailbox) {
        super();
        this.displayName = raw.display_name || null;
        this.address = raw.address;
    }

    public override toString(displayMode: DisplayMode = DisplayMode.Short): string {
        const d = this.displayName && this.displayName.length > 0;
        if (displayMode === DisplayMode.Long) {
            return (d ? this.displayName + ' <' : '<') + this.address + '>';
        } else if (d) {
            return this.displayName;
        } else {
            return this.address;
        }
    }
}

export class Group extends Object {
    public displayName: string | null;
    public mailboxes: Mailbox[];

    constructor(raw: RawGroup) {
        super();
        this.displayName = raw.display_name || null;
        this.mailboxes = raw.mailboxes.map(m => new Mailbox(m));
    }

    public override toString(): string {
        throw new Error('not yet implemented');
    }
}