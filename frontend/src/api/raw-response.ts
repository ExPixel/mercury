export interface RawMailListItem {
    id: number;
    metadata: RawMailMetadata;
    created_at: string;
}

export interface RawMailMetadata {
    from: string;
    to: string[];
}