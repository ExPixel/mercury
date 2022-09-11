// SPDX-License-Identifier: GPL-3.0-or-later

import { Box, Text, createStyles, Group } from '@mantine/core';
import { formatRelative } from 'date-fns';
import * as React from 'react';
import { MailListItem } from '../api/response';
import { EmailPath, parseValidEmailPath } from '../util/email';

export interface EmailCardProps {
    email: MailListItem,
}

const useStyles = createStyles(() => ({
    root: {
        overflowX: 'hidden',
        textOverflow: 'ellipsis',
        whiteSpace: 'nowrap',
        display: 'flex',
        flexDirection: 'column',
    },

    metadata: {
        padding: 0,
        margin: 0,

        '& .key': {
            textAlign: 'end',
        },

        '& .value': {
            paddingLeft: '4px',
        }
    },
}));

export default function EmailListItem(props: EmailCardProps) {
    const { classes } = useStyles();
    const { email } = props;

    return <Box className={classes.root}>
        <table className={classes.metadata} cellPadding={0} cellSpacing={0}>
            <tr>
                <td className="key"><Text span size="sm" weight="bold" color="dimmed">Subj</Text></td>
                <td className="value"><Text span>Some really long subject line that gets cutoff</Text></td>
            </tr>
            <tr>
                <td className="key"><Text span size="sm" weight="bold" color="dimmed">To</Text></td>
                <td className="value"><Text span>{email.recipientDisplayString}</Text></td>
            </tr>

            <tr>
                <td className="key"><Text span size="sm" weight="bold" color="dimmed">From</Text></td>
                <td className="value"><Text span>{email.sender.toString()}</Text></td>
            </tr>

            <tr>
                <td colSpan={2}>
                    <Text size="sm" color="dimmed">
                        {formatRelative(email.createdAt, new Date())}
                    </Text>
                </td>
            </tr>
        </table>
    </Box>;
}