// src/theme/AppMuiTheme.tsx
import { ThemeProvider as MuiThemeProvider, createTheme, CssBaseline } from '@mui/material';
import { useTheme } from '@/context/ThemeContext';

export const AppMuiTheme = ({ children }: { children: React.ReactNode }) => {
  const { isDarkMode } = useTheme();

  const muiTheme = createTheme({
    palette: {
      mode: isDarkMode ? 'dark' : 'light',
    },
  });

  return (
    <MuiThemeProvider theme={muiTheme}>
      <CssBaseline />
      {children}
    </MuiThemeProvider>
  );
};
