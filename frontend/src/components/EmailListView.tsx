// SPDX-License-Identifier: GPL-3.0-or-later

import { Box, Loader, Text, createStyles } from '@mantine/core';
import * as React from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useMercury } from '../api';
import { MailListItem } from '../api/response';
import EmailList from './EmailList';

const useStyles = createStyles(() => ({
    loadingContainer: {
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'center',
        alignItems: 'center',
        flex: 1,
    }
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
    const emailsRef = React.useRef(emails);

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
            const mailMaxId = (emailsRef.current || []).reduce((acc, item) => {
                console.log({ acc, item });
                return Math.max(acc, item.id);
            }, 0);
            console.debug('new mail available, fetching mail after %d', mailMaxId);

            mercury.getMailList({ after: mailMaxId }).then((mailList) => {
                console.debug('loaded new emails', mailList);
                setEmails(mailList.concat(emailsRef.current || []));
            });
        });

        return () => {
            mercury.removeListener(listenerId);
        };
    }, []);

    if (!!emails) {
        return <EmailList selectedId={mailId} onSelect={onSelectMailItem} emails={emails} />;
    } else {
        return <Box className={classes.loadingContainer}>
            <Text color="dimmed" size="lg">Fetching Emails...</Text>
            <Loader size="xl" variant='bars' />
        </Box>;
    }
}