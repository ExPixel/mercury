// SPDX-License-Identifier: GPL-3.0-or-later

import * as React from 'react';
import { MailListItem } from '../api/response';
import EmailListItem from './EmailListItem';

export interface EmailListProps {
    emails: MailListItem[],
}

export default function EmailList(props: EmailListProps) {
    const emailCards = props.emails.map((email) =>
        <EmailListItem key={email.id} email={email} />
    );
    return <>{emailCards}</>;
}