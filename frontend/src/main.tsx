import { createRoot } from 'react-dom/client';
import './index.css';
import App from './App.tsx';
import React from 'react';
import 'antd/dist/reset.css';
import { AuthProvider } from './context/AuthContext.tsx';
import { ThemeProvider } from './context/ThemeContext.tsx';
import { App as AntApp } from 'antd';

createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ThemeProvider>
      <AuthProvider>
        <AntApp>
          <App />
        </AntApp>
      </AuthProvider>
    </ThemeProvider>
  </React.StrictMode>,
);
