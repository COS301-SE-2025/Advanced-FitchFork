import { createRoot } from 'react-dom/client';
import './index.css';
import App from './App.tsx';
import React from 'react';
import 'antd/dist/reset.css';
import { AuthProvider } from './context/AuthContext.tsx';
import { App as AntApp } from 'antd';
import { BreadcrumbProvider } from './context/BreadcrumbContext.tsx';
import { UIProvider } from './context/UIContext.tsx';
import { ThemeProvider } from './context/ThemeContext.tsx';

createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ThemeProvider>
      <UIProvider>
        <AuthProvider>
          <AntApp>
            <BreadcrumbProvider>
              <App />
            </BreadcrumbProvider>
          </AntApp>
        </AuthProvider>
      </UIProvider>
    </ThemeProvider>
  </React.StrictMode>,
);
