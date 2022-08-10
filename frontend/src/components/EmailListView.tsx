// SPDX-License-Identifier: GPL-3.0-or-later

import * as React from 'react';
import { useMercury } from '../api';
import EmailList from './EmailList';

export default function EmailListView() {
    const mercury = useMercury();
    return <EmailList></EmailList>;
}