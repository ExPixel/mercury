// SPDX-License-Identifier: GPL-3.0-or-later

import { Box, Loader, ScrollArea, Text, createStyles } from '@mantine/core';
import * as React from 'react';
import { useMercury } from '../api';
import { MailListItem } from '../api/response';
import EmailList from './EmailList';

const useStyles = createStyles(() => ({
    scroll: {
        '& > div > div': {
            display: 'flex !important',
            flexDirection: 'column',
        }
    },
}));

export default function EmailListView() {
    const { classes } = useStyles();
    const mercury = useMercury();
    const [emails, setEmails] = React.useState<MailListItem[]>();

    React.useEffect(() => {
        mercury.getMailList().then((mailList) => {
            console.log('emails', mailList);
            setEmails(mailList);
        });
    }, []);

    if (!!emails) {
        return <ScrollArea className={classes.scroll} offsetScrollbars type="auto">
            <EmailList emails={emails} />
        </ScrollArea >;
    } else {
        return <Box sx={{
            display: 'flex',
            flexDirection: 'column',
            justifyContent: 'center',
            alignItems: 'center',
            flex: 1,
        }}>
            <Text color="dimmed" size="lg">Fetching Emails...</Text>
            <Loader size="xl" variant='bars' />
        </Box>;
    }
}