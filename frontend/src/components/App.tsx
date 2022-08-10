// SPDX-License-Identifier: GPL-3.0-or-later

import * as React from 'react';
import {
    MantineProvider,
    AppShell,
    Navbar,
    Header,
} from '@mantine/core';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import EmailWithSidebarView from './EmailWithSidebarView';
import EmailListView from './EmailListView';

export function AppNav() {
    return (<>
        <Navbar.Section mt='xs'>[ HEADER ]</Navbar.Section>
        <Navbar.Section grow mt='md'>
            <EmailListView />
        </Navbar.Section>
        <Navbar.Section>[ FOOTER ]</Navbar.Section>
    </>);
}

export function AppContent() {
    return (
        <AppShell
            padding='md'
            navbar={<Navbar width={{ base: 320 }} p='xs'><AppNav /></Navbar>}
        >
            <div>Some Application Content</div>
        </AppShell>
    );
}

export default function App() {
    return (
        <React.Fragment>
            <MantineProvider withGlobalStyles withNormalizeCSS theme={{ colorScheme: 'dark' }}>
                <BrowserRouter>
                    <AppContent />
                </BrowserRouter>
            </MantineProvider>
        </React.Fragment>
    );
}
