// SPDX-License-Identifier: GPL-3.0-or-later

import { Box, Text, createStyles } from '@mantine/core';
import { formatRelative } from 'date-fns';
import * as React from 'react';
import { MailListItem } from '../api/response';
import { EmailPath, parseValidEmailPath } from '../util/email';

export interface EmailCardProps {
    email: MailListItem,
}

const useStyles = createStyles(() => ({
    root: {
    },
}));

export default function EmailListItem(props: EmailCardProps) {
    const { classes } = useStyles();

    const sender = props.email.sender;
    const createdAt = props.email.createdAt;

    return <Box className={classes.root}>
        <Text weight={500}>{sender.displayNameOrAddress}</Text>
        <Text size="sm" color="dimmed">
            {formatRelative(createdAt, new Date())}
        </Text>
    </Box>;
}