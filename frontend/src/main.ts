// SPDX-License-Identifier: GPL-3.0-or-later

import * as React from 'react';
import ReactDOM from 'react-dom/client';
import App from './components/App';

function main() {
    const container = document.querySelector('#root');
    const root = ReactDOM.createRoot(container);
    root.render(React.createElement(App));
}
main();