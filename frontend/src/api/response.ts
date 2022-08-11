import { RawMailListItem, RawMailMetadata } from "./raw-response";
import { parseISO } from 'date-fns';

export class MailListItem {
    public id: number;
    public createdAt: Date;
    public metadata: MailMetadata;

    constructor(raw: RawMailListItem) {
        this.id = raw.id;
        this.createdAt = parseISO(raw.created_at);
        this.metadata = new MailMetadata(raw.metadata);
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