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

import { css, jsx } from '@emotion/react';
import EmailDetailsView from './EmailDetailsView';
import EmailList from './EmailList';

export default function EmailWithSidebarView() {
    return <div>Hi</div>;
    // return <Box component='div' sx={{ display: 'flex' }}>
    //     <EmailList />
    //     <EmailDetailsView css={css`flex: 1;`} />
    // </Box>
}