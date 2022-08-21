// SPDX-License-Identifier: GPL-3.0-or-later

import { Box, Loader, Text } from '@mantine/core';
import * as React from 'react';
import { useMercury } from '../api';
import { MailListItem } from '../api/response';
import EmailList from './EmailList';

export default function EmailListView() {
    const mercury = useMercury();
    const [emails, setEmails] = React.useState<MailListItem[]>();

    React.useEffect(() => {
        mercury.getMailList().then((mailList) => {
            console.log('emails', mailList);
            setEmails(mailList);
        });
    }, []);

    if (!!emails) {
        return <Box sx={{
            display: 'flex',
            flexDirection: 'column',
        }}>
            <EmailList emails={emails} />
        </Box>;
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