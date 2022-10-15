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
import { MercuryContext, useMercury } from '../api';

function AppEmailList() {
    const navigation =
        <>
            <Navbar.Section mt='xs'>[ HEADER ]</Navbar.Section>
            <Navbar.Section grow mt='md' sx={{ overflow: 'auto' }}>
                <EmailListView />
            </Navbar.Section>
            <Navbar.Section>[ FOOTER ]</Navbar.Section>
        </>;

    const content =
        <AppShell padding='md' navbar={<Navbar width={{ base: 320 }}>{navigation}</Navbar>}>
            <div>Some Application Content</div>
        </AppShell>;

    return content;
}

function AppRoutes() {
    return <Routes>
        <Route path="/" element={<AppEmailList />} />
        <Route path="/mail/:mailId" element={<AppEmailList />} />
    </Routes>;
}

export default function App() {
    const mercury = useMercury();
    return (
        <React.Fragment>
            <MantineProvider withGlobalStyles withNormalizeCSS theme={{ colorScheme: 'dark' }}>
                <BrowserRouter>
                    <MercuryContext.Provider value={mercury}>
                        <AppRoutes />
                    </MercuryContext.Provider>
                </BrowserRouter>
            </MantineProvider>
        </React.Fragment>
    );
}
