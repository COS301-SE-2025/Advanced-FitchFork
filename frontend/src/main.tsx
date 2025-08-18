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
import { AppMuiTheme } from './context/AppMuiTheme.tsx';
import { MessageContextHolder } from './utils/message.tsx';
import { ViewSlotProvider } from './context/ViewSlotContext.tsx';

createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ThemeProvider>
      <AppMuiTheme>
        <UIProvider>
          <AuthProvider>
            <AntApp>
              <MessageContextHolder />
              <BreadcrumbProvider>
                <ViewSlotProvider>
                  <App />
                </ViewSlotProvider>
              </BreadcrumbProvider>
            </AntApp>
          </AuthProvider>
        </UIProvider>
      </AppMuiTheme>
    </ThemeProvider>
  </React.StrictMode>,
);
