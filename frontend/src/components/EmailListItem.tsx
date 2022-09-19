// SPDX-License-Identifier: GPL-3.0-or-later

import { Box, Text, createStyles, Group, DefaultProps, useComponentDefaultProps } from '@mantine/core';
import { isWithinInterval, startOfToday, endOfToday, format } from 'date-fns';
import * as React from 'react';
import { MailListItem } from '../api/response';
import { EmailPath, parseValidEmailPath } from '../util/email';

export interface EmailListItemProps extends DefaultProps {
    email: MailListItem,
    selected: boolean,
    onClick?: React.MouseEventHandler<HTMLElement>,
}

const defaultProps: Partial<EmailListItemProps> = {
};

const useStyles = createStyles((theme) => ({
    root: {
        display: 'flex',
        flexDirection: 'column',
        padding: theme.spacing.xs,
        cursor: 'pointer',

        '&:hover': {
            backgroundColor: theme.colors.gray[8],
        },

        '&.selected': {
            backgroundColor: theme.primaryColor,
        },
    },

    sender: {
        flex: 1,
        textOverflow: 'ellipsis',
        overflow: 'hidden',
        whiteSpace: 'nowrap',
    },

    subject: {
        lineClamp: 2,
        textOverflow: 'ellipsis',
        overflow: 'hidden',
    },

    recipient: {
        flex: 1,
        textOverflow: 'ellipsis',
        overflow: 'hidden',
        whiteSpace: 'nowrap',
    }
}));

export default function EmailListItem(props: EmailListItemProps) {
    const { classes, cx } = useStyles();
    const { email, selected, className, onClick, ...others } = useComponentDefaultProps('EmailListItem', defaultProps, props);

    const dateFormat = isWithinInterval(email.createdAt,
        { start: startOfToday(), end: endOfToday() }) ? 'p' : 'P';
    const formattedDate = format(email.createdAt, dateFormat);

    return <Box onClick={onClick} className={cx(classes.root, className, { 'selected': selected })} {...others}>
        <Group noWrap>
            <Text className={classes.sender} span weight="bold">{email.sender.toString()}</Text>
            <Text span>{formattedDate}</Text>
        </Group>
        <Text className={classes.subject} span>{email.subject}</Text>
        <Group noWrap spacing="xs">
            <Text weight="bold" color="dimmed" span>To:</Text>
            <Text span>{email.recipientDisplayString}</Text>
        </Group>
    </Box>;
}