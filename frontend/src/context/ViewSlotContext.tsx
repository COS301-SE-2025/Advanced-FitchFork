import { createContext, useContext, useState } from 'react';
import type { ReactNode } from 'react';

/** The value shape stored in a view slot context. */
type ViewSlotContextValue<T = ReactNode> = {
  /** Current value in the slot (e.g., header content). */
  value: T;
  /** Replace the current value. */
  setValue: (val: T) => void;

  /** Optional explicit back target for mobile headers etc. */
  backTo: string | null;
  /** Set/clear the back target. Pass null to clear. */
  setBackTo: (route: string | null) => void;
};

/**
 * Factory to create a new view slot context.
 *
 * @template T The type of value the slot holds (e.g., ReactNode, string, etc.)
 * @param defaultValue Initial value for the slot.
 * @returns A tuple: `[Provider, useViewSlot]`
 */
export function createViewSlotContext<T = ReactNode>(defaultValue: T) {
  const Context = createContext<ViewSlotContextValue<T>>({
    value: defaultValue,
    setValue: () => {},
    backTo: null,
    setBackTo: () => {},
  });

  const Provider = ({ children }: { children: ReactNode }) => {
    const [value, setValue] = useState<T>(defaultValue);
    const [backTo, setBackTo] = useState<string | null>(null);

    return (
      <Context.Provider value={{ value, setValue, backTo, setBackTo }}>{children}</Context.Provider>
    );
  };

  const useViewSlot = () => {
    const ctx = useContext(Context);
    if (!ctx) throw new Error('useViewSlot must be used within its Provider');
    return ctx;
  };

  return [Provider, useViewSlot] as const;
}

/** Default global slot */
export const [ViewSlotProvider, useViewSlot] = createViewSlotContext<ReactNode>(null);
