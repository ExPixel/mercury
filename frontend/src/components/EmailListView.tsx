// SPDX-License-Identifier: GPL-3.0-or-later

import { Box, Loader, ScrollArea, Text, createStyles } from '@mantine/core';
import * as React from 'react';
import { useNavigate, useParams } from 'react-router-dom';
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
    const navigate = useNavigate();
    const params = useParams();

    let mailId: number | null = Number.parseInt(params['mailId'], 10);
    if (!Number.isFinite(mailId)) {
        mailId = null;
    }

    const { classes } = useStyles();
    const mercury = useMercury();
    const [emails, setEmails] = React.useState<MailListItem[]>();

    const onSelectMailItem = (mail: MailListItem) => {
        console.debug('selected email', mail);
        navigate(`/mail/${mail.id}`);
    };

    React.useEffect(() => {
        mercury.getMailList().then((mailList) => {
            console.debug('loaded emails', mailList);
            setEmails(mailList);
        });

        const listenerId = mercury.listenForNewMail(() => {
            console.debug('new mail available');
        });

        return () => {
            mercury.removeListener(listenerId);
        };
    }, []);

    if (!!emails) {
        return <ScrollArea className={classes.scroll} type="auto">
            <EmailList selectedId={mailId} onSelect={onSelectMailItem} emails={emails} />
        </ScrollArea>;
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