import React, {useEffect, useState} from 'react';
import 'bootstrap/scss/bootstrap.scss';

export function App() {
    const [tab, setTab] = useState('p');
    const [nodes, setNodes] = useState([]);

    async function fetchData(tab : string) {
        console.log('tab', tab);
        let resp = await fetch(tab == 'p' ? "/provider" : "/requestor");
        let json = await resp.json();
        setNodes(json);
    }

    useEffect(() => {
        fetchData(tab);
    }, [tab]);

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
        <div className="row">
            {nodes.map((node) => <div className="col-4 p-1">
                <div key={node} className="card">
                <div className="card-header">{node}</div>
                <div className="card-body">
                    <div className="card-title">Show me</div>
                    <div className="card-text"></div>
                    <a href="#" className="btn btn-primary">details</a>
                </div>
                </div>
            </div>)}
        </div>
    </div>);
}
