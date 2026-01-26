import React from 'react';
import ReactDOM from 'react-dom/client';
import { App } from './App';
import './index.css';
import { OpenAPI } from './api/generated/core/OpenAPI';

// Configure API base URL
OpenAPI.BASE = 'http://localhost:5150';

ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
        <App />
    </React.StrictMode>
);
