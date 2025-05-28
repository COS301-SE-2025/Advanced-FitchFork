import { createContext, useContext, useState } from 'react';

type BreadcrumbMap = Record<string, string>;

interface BreadcrumbContextValue {
  customLabels: BreadcrumbMap;
  setBreadcrumbLabel: (key: string, label: string) => void;
}

const BreadcrumbContext = createContext<BreadcrumbContextValue | undefined>(undefined);

export function BreadcrumbProvider({ children }: { children: React.ReactNode }) {
  const [customLabels, setCustomLabels] = useState<BreadcrumbMap>({});

  const setBreadcrumbLabel = (key: string, label: string) => {
    setCustomLabels((prev) => ({ ...prev, [key]: label }));
  };

  return (
    <BreadcrumbContext.Provider value={{ customLabels, setBreadcrumbLabel }}>
      {children}
    </BreadcrumbContext.Provider>
  );
}

export function useBreadcrumbContext() {
  const ctx = useContext(BreadcrumbContext);
  if (!ctx) throw new Error('useBreadcrumbContext must be used inside BreadcrumbProvider');
  return ctx;
}
