export interface RawMailListItem {
    id: number;
    created_at: string;
    from?: RawMailbox[];
    sender?: RawMailbox;
}

export interface RawMailbox {
    display_name?: string | null;
    address: string;
}

export interface RawMailMetadata {
    from: string;
    to: string[];
}