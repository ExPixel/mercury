export enum RawAddressType {
    Group = "group",
    Mailbox = "mailbox",
}

export interface RawMailListItem {
    id: number;
    created_at: string;
    from?: RawMailbox[];
    sender?: RawMailbox;
    to: RawAddress[];
}

export type RawAddressMailbox = { type: RawAddressType.Mailbox } & RawMailbox;
export type RawAddressGroup = { type: RawAddressType.Group } & RawGroup;
export type RawAddress = RawAddressMailbox | RawAddressGroup;

export interface RawMailbox {
    display_name?: string | null;
    address: string;
}

export interface RawGroup {
    display_name?: string | null;
    mailboxes: RawMailbox[];
}

export interface RawMailMetadata {
    from: string;
    to: string[];
}