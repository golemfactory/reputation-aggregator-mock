import React, { useState } from 'react';
import 'bootstrap/scss/bootstrap.scss';

export function App() {
    const [tab, setTab] = useState('p');

    return (<div className="container-fluid">
        <h1>Reputation Aggregator</h1>
        <ul className="nav nav-pills">
            <li className="nav-item">
                <a className={tab === 'p' ? 'nav-link active' : 'nav-link'} href="#" onClick={() => setTab('p')}>Providers</a>
            </li>
            <li className="nav-item">
                <a className={tab === 'r' ? 'nav-link active' : 'nav-link'} onClick={() => setTab('r')} href="#">Requestors</a>
            </li>
        </ul>
    </div>);
}
