/**
 * A tiny, generic "view slot" context factory.
 *
 * ## What is a view slot?
 * Think of it like a named placeholder in your layout (e.g., a header area on mobile).
 * Any child route/screen can set content into that slot without prop-drilling.
 *
 * ## Why not a specific context?
 * This factory lets you create *multiple* independent slots if you ever need them
 * (e.g., MobileHeaderSlot, ToolbarSlot, FooterSlot), all with the same simple API.
 *
 * ## Basic usage
 * ```tsx
 * // 1) Create a slot (typically in your context file)
 * export const [MobileHeaderProvider, useMobileHeader] = createViewSlotContext<ReactNode>(null);
 *
 * // 2) Wrap a subtree (e.g., in main.tsx or a layout component)
 * <MobileHeaderProvider>
 *   <App />
 * </MobileHeaderProvider>
 *
 * // 3) Read from the slot (e.g., your header component)
 * const { value: headerContent } = useMobileHeader();
 *
 * // 4) Write to the slot (e.g., inside a page)
 * const { setValue } = useMobileHeader();
 * useEffect(() => {
 *   setValue(<span>My Page Title</span>);
 *   return () => setValue(null); // optional: clear on unmount
 * }, []);
 * ```
 */

import { createContext, useContext, useState } from 'react';
import type { ReactNode } from 'react';

/**
 * The value shape stored in a view slot context.
 * @template T The type of the slot's value (defaults to ReactNode for UI content).
 */
type ViewSlotContextValue<T = ReactNode> = {
  /** Current value in the slot. */
  value: T;
  /** Replace the current value. Usually called from pages/routes that want to set header content, etc. */
  setValue: (val: T) => void;
};

/**
 * Factory to create a new view slot context.
 *
 * @template T The type of value the slot holds (e.g., ReactNode, string, etc.)
 * @param defaultValue Initial value for the slot.
 * @returns A tuple: `[Provider, useViewSlot]`
 *
 * - **Provider**: Wrap your subtree that should share the slot.
 * - **useViewSlot**: Hook to get `{ value, setValue }`.
 */
export function createViewSlotContext<T = ReactNode>(defaultValue: T) {
  const Context = createContext<ViewSlotContextValue<T>>({
    value: defaultValue,
    // eslint-disable-next-line @typescript-eslint/no-empty-function
    setValue: () => {},
  });

  /**
   * Provider that holds the slot's state.
   * Wrap any part of the app that should share the slot value.
   */
  const Provider = ({ children }: { children: ReactNode }) => {
    const [value, setValue] = useState<T>(defaultValue);
    return <Context.Provider value={{ value, setValue }}>{children}</Context.Provider>;
  };

  /**
   * Hook to read/write the slot's value.
   * Must be used under the matching Provider.
   */
  const useViewSlot = () => {
    const ctx = useContext(Context);
    if (!ctx) {
      throw new Error('useViewSlot must be used within its Provider');
    }
    return ctx;
  };

  return [Provider, useViewSlot] as const;
}

/**
 * Default "global" slot:
 * - Use this if you just need one slot in the app (e.g., for a mobile page header).
 * - If you need multiple distinct slots, create more with `createViewSlotContext`.
 */
export const [ViewSlotProvider, useViewSlot] = createViewSlotContext<ReactNode>(null);
