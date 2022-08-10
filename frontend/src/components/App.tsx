/*
    Mercury Mail Testing Tool
    Copyright (C) 2022 Adolph Celestin

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

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
