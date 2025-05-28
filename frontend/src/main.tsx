import { createRoot } from 'react-dom/client';
import './index.css';
import App from './App.tsx';
import React from 'react';
import 'antd/dist/reset.css';
import { AuthProvider } from './context/AuthContext.tsx';
import { ThemeProvider } from './context/ThemeContext.tsx';
import { App as AntApp } from 'antd';
import { BreadcrumbProvider } from './context/BreadcrumbContext.tsx';

createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ThemeProvider>
      <AuthProvider>
        <AntApp>
          <BreadcrumbProvider>
            <App />
          </BreadcrumbProvider>
        </AntApp>
      </AuthProvider>
    </ThemeProvider>
  </React.StrictMode>,
);
