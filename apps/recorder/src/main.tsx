import React from 'react';
import ReactDOM from 'react-dom/client';
import { App } from './app';
import './main.css';

const rootEl = document.getElementById('root');
if (rootEl) {
  rootEl.classList.add('min-h-svh');
  const root = ReactDOM.createRoot(rootEl);
  root.render(
    <React.StrictMode>
      <App />
    </React.StrictMode>
  );
}
