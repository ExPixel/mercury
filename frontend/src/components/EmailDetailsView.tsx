// SPDX-License-Identifier: GPL-3.0-or-later

import * as React from 'react';

interface EmailDetailsViewProps {
    className?: string;
}

export default function EmailDetailsView(props: EmailDetailsViewProps) {
    console.log(props);
    return <div className={props.className}>Email Details View</div>
}