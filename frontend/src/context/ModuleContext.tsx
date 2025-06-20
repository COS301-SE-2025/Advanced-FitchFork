// src/context/ModuleContext.tsx
import { createContext, useContext } from 'react';
import type { ModuleDetailsResponse } from '@/types/modules';

interface ModuleContextValue {
  module: ModuleDetailsResponse;
}

const ModuleContext = createContext<ModuleContextValue | null>(null);

export const useModule = (): ModuleDetailsResponse => {
  const context = useContext(ModuleContext);
  if (!context) throw new Error('useModule must be used within a ModuleProvider');
  return context.module;
};

export const ModuleProvider = ModuleContext.Provider;
