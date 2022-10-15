// SPDX-License-Identifier: GPL-3.0-or-later

import * as React from 'react';
import { createStyles } from '@mantine/core';
import { MailListItem } from '../api/response';
import EmailListItem from './EmailListItem';

export interface EmailListProps {
    emails: MailListItem[],
    selectedId: number | null,
    onSelect?: (item: MailListItem) => void,
}

const useStyles = createStyles((theme) => ({
    item: {
        borderBottom: `1px solid ${theme.colors.gray[7]}`,

        '&:first-child': {
            borderTop: `1px solid ${theme.colors.gray[7]}`,
        }
    }
}));


export default function EmailList(props: EmailListProps) {
    const { classes } = useStyles();
    const { emails, selectedId, onSelect } = props;

    const emailCards = emails.map((email) => {
        const onItemClick = () => onSelect && onSelect(email);
        const selected = email.id === selectedId;
        return <EmailListItem className={classes.item} selected={selected} onClick={onItemClick} key={email.id} email={email} />;
    });
    return <>{emailCards}</>;
}