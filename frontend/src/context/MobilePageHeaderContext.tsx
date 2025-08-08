import { createContext, useContext, useState } from 'react';
import type { ReactNode } from 'react';

const MobilePageHeaderContext = createContext<{
  content: ReactNode;
  setContent: (node: ReactNode) => void;
}>({
  content: null,
  setContent: () => {},
});

export const MobilePageHeaderProvider = ({ children }: { children: ReactNode }) => {
  const [content, setContent] = useState<ReactNode>(null);

  return (
    <MobilePageHeaderContext.Provider value={{ content, setContent }}>
      {children}
    </MobilePageHeaderContext.Provider>
  );
};

export const useMobilePageHeader = () => {
  const context = useContext(MobilePageHeaderContext);
  if (!context) {
    throw new Error('useMobilePageHeader must be used within MobilePageHeaderProvider');
  }
  return context;
};
