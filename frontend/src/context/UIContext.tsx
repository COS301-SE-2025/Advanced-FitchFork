import React, { createContext, useContext, useEffect, useState } from 'react';
import { ConfigProvider, Grid, theme as antdTheme } from 'antd';
import { useTheme } from './ThemeContext';

const { useBreakpoint } = Grid;

interface UIContextType {
  compact: boolean;
  setCompact: (val: boolean) => void;
  motion: boolean;
  setMotion: (val: boolean) => void;
  isMobile: boolean;
  isSm: boolean;
  isMd: boolean;
  isLg: boolean;
}

const UIContext = createContext<UIContextType | undefined>(undefined);

export const useUI = () => {
  const ctx = useContext(UIContext);
  if (!ctx) throw new Error('useUI must be used within UIProvider');
  return ctx;
};

export const UIProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [compact, setCompact] = useState(() => localStorage.getItem('compact') === 'true');
  const [motion, setMotion] = useState(() => localStorage.getItem('motion') !== 'false');
  const [isMobile, setIsMobile] = useState(() => window.innerWidth <= 768);

  const { isDarkMode } = useTheme();
  const screens = useBreakpoint();

  // Derive screen size booleans (≥breakpoint)
  const isSm = !!screens.sm;
  const isMd = !!screens.md;
  const isLg = !!screens.lg;

  useEffect(() => {
    localStorage.setItem('compact', String(compact));
  }, [compact]);

  useEffect(() => {
    localStorage.setItem('motion', String(motion));
  }, [motion]);

  useEffect(() => {
    const mq = window.matchMedia('(max-width: 768px)');

    const updateMobile = () => setIsMobile(mq.matches);
    updateMobile();

    mq.addEventListener('change', updateMobile);
    return () => mq.removeEventListener('change', updateMobile);
  }, []);

  return (
    <UIContext.Provider
      value={{
        compact,
        setCompact,
        motion,
        setMotion,
        isMobile,
        isSm,
        isMd,
        isLg,
      }}
    >
      <ConfigProvider
        theme={{
          algorithm: [
            isDarkMode ? antdTheme.darkAlgorithm : antdTheme.defaultAlgorithm,
            ...(compact ? [antdTheme.compactAlgorithm] : []),
          ],
          token: {
            motion,
          },
        }}
      >
        {children}
      </ConfigProvider>
    </UIContext.Provider>
  );
};
