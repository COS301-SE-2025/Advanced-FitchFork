import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from 'react';

export type TocItem = { key: string; href: string; title: React.ReactNode };

type HelpContextValue = {
  items: TocItem[];
  setItems: (items: TocItem[]) => void;

  extra: ReactNode | null;
  setExtra: (node: ReactNode | null) => void;

  activeHref: string;
  setActiveHref: (next: string | ((prev: string) => string)) => void;

  getContainer: () => HTMLElement;
  targetOffset: number;
};

const HelpContext = createContext<HelpContextValue | null>(null);

export const HelpProvider: React.FC<{
  children: React.ReactNode;
  containerId?: string;
  targetOffset?: number;
}> = ({ children, containerId = 'help-scroll-container', targetOffset = 12 }) => {
  const [items, setItems] = useState<TocItem[]>([]);
  const [extra, setExtra] = useState<ReactNode | null>(null);
  const [activeHref, _setActiveHref] = useState<string>('');
  const containerIdRef = useRef(containerId);

  const setActiveHref: HelpContextValue['setActiveHref'] = (next) => {
    _setActiveHref((prev) => (typeof next === 'function' ? (next as any)(prev) : next));
  };

  const getContainer = useCallback(() => {
    const el = document.getElementById(containerIdRef.current);
    return (el ?? document.documentElement) as HTMLElement;
  }, []);

  const value = useMemo(
    () => ({
      items,
      setItems,
      extra,
      setExtra,
      activeHref,
      setActiveHref,
      getContainer,
      targetOffset,
    }),
    [items, extra, activeHref, getContainer, targetOffset],
  );

  return <HelpContext.Provider value={value}>{children}</HelpContext.Provider>;
};

export function useHelp() {
  const ctx = useContext(HelpContext);
  if (!ctx) throw new Error('useHelp must be used within <HelpProvider>');
  return ctx;
}

export function useHelpToc(opts: {
  items: TocItem[];
  ids: string[];
  extra?: ReactNode | null;
  onMountScrollToHash?: boolean;
}) {
  const { setItems, setExtra, setActiveHref, getContainer, targetOffset } = useHelp();
  const { items, ids, extra = null, onMountScrollToHash = true } = opts;

  useEffect(() => {
    setItems(items);
    setExtra(extra);
    return () => {
      setItems([]);
      setExtra(null);
      setActiveHref('');
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [JSON.stringify(items)]);

  useEffect(() => {
    const container = getContainer();
    if (!container) return;

    let rafId = 0;
    const recompute = () => {
      const containerTop = container.getBoundingClientRect().top;
      let bestId = ids[0] || '';
      let bestTop = -Infinity;

      for (const id of ids) {
        const el = container.querySelector<HTMLElement>(`#${CSS.escape(id)}`);
        if (!el) continue;
        const top = el.getBoundingClientRect().top - containerTop - targetOffset;
        if (top <= 1 && top > bestTop) {
          bestTop = top;
          bestId = id;
        }
      }
      if (bestId) {
        setActiveHref((prev) => (prev === `#${bestId}` ? prev : `#${bestId}`));
      }
    };

    const onScroll = () => {
      if (rafId) cancelAnimationFrame(rafId);
      rafId = requestAnimationFrame(recompute);
    };

    recompute();
    container.addEventListener('scroll', onScroll, { passive: true });
    window.addEventListener('resize', onScroll, { passive: true });

    return () => {
      container.removeEventListener('scroll', onScroll);
      window.removeEventListener('resize', onScroll);
      if (rafId) cancelAnimationFrame(rafId);
    };
  }, [ids.join('|'), getContainer, setActiveHref, targetOffset]);

  useEffect(() => {
    if (!onMountScrollToHash) return;
    const container = getContainer();
    if (!container) return;
    const hash = decodeURIComponent(window.location.hash || '').replace(/^#/, '');
    if (!hash) return;
    const target = container.querySelector<HTMLElement>(`#${CSS.escape(hash)}`);
    if (!target) return;

    const top =
      target.getBoundingClientRect().top -
      container.getBoundingClientRect().top +
      container.scrollTop -
      targetOffset;

    container.scrollTo({ top, behavior: 'auto' });
  }, [getContainer, targetOffset, onMountScrollToHash]);
}
