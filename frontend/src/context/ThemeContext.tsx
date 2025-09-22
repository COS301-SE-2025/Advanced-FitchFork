import React, { createContext, useContext, useEffect, useState } from 'react';

type ThemeMode = 'light' | 'dark' | 'system';

interface ThemeContextType {
  mode: ThemeMode;
  isDarkMode: boolean;
  setMode: (mode: ThemeMode) => void;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export const useTheme = () => {
  const ctx = useContext(ThemeContext);
  if (!ctx) throw new Error('useTheme must be used within ThemeProvider');
  return ctx;
};

export const ThemeProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const getInitialMode = (): ThemeMode => {
    const stored = localStorage.getItem('theme');
    return stored === 'light' || stored === 'dark' ? stored : 'system';
  };

  const [mode, setMode] = useState<ThemeMode>(getInitialMode);
  const [isDarkMode, setIsDarkMode] = useState(
    () =>
      mode === 'dark' ||
      (mode === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches),
  );

  useEffect(() => {
    const handleChange = () => {
      const systemDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      const effectiveDark = mode === 'dark' || (mode === 'system' && systemDark);
      setIsDarkMode(effectiveDark);
      document.documentElement.classList.toggle('dark', effectiveDark);
    };

    handleChange();
    localStorage.setItem('theme', mode);
    if (mode === 'system') {
      const media = window.matchMedia('(prefers-color-scheme: dark)');
      media.addEventListener('change', handleChange);
      return () => media.removeEventListener('change', handleChange);
    }
  }, [mode]);

  return (
    <ThemeContext.Provider value={{ mode, isDarkMode, setMode }}>{children}</ThemeContext.Provider>
  );
};
