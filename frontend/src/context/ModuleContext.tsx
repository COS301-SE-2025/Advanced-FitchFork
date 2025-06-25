// src/context/ModuleContext.tsx
import { createContext, useContext } from 'react';
import type { User } from '@/types/users';
import type { Module } from '@/types/modules';

interface ModuleDetails extends Module {
  lecturers: User[];
  tutors: User[];
  students: User[];
}

interface ModuleContextValue {
  module: ModuleDetails;
}

const ModuleContext = createContext<ModuleContextValue | null>(null);

export const useModule = (): ModuleDetails => {
  const context = useContext(ModuleContext);
  if (!context) throw new Error('useModule must be used within a ModuleProvider');
  return context.module;
};

export const ModuleProvider = ModuleContext.Provider;
