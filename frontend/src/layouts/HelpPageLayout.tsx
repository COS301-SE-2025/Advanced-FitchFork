// src/pages/help/HelpPageLayout.tsx
import { useMemo, useState, useEffect, useRef } from 'react';
import { Layout, Menu, Anchor, Button } from 'antd';
import { LeftOutlined, RightOutlined } from '@ant-design/icons';
import { Outlet, useLocation, useNavigate } from 'react-router-dom';
import { HelpProvider, useHelp } from '@/context/HelpContext';
import { HELP_MENU_ITEMS, HELP_LEAF_ROUTES } from '@/pages/help/menuConfig';
import { useUI } from '@/context/UIContext';
import { useTheme } from '@/context/ThemeContext';

const { Sider, Content, Header } = Layout;

function RightOfContentAnchor() {
  const { items, activeHref, setActiveHref, getContainer, targetOffset, extra } = useHelp();

  return (
    <div className="!pt-1.5 !pr-3 !pb-3 !pl-3">
      <Anchor
        items={items}
        affix={false}
        getContainer={getContainer}
        getCurrentAnchor={() => activeHref || (items[0]?.href ?? '')}
        targetOffset={targetOffset}
        className="!p-0 !m-0"
        onClick={(e, link) => {
          e.preventDefault();
          const container = getContainer();
          const raw = link?.href || '';
          const id = raw.startsWith('#') ? raw.slice(1) : raw.split('#').pop() || '';
          const el = container.querySelector<HTMLElement>(`#${CSS.escape(id)}`);
          if (!el) return;

          const top =
            el.getBoundingClientRect().top -
            container.getBoundingClientRect().top +
            container.scrollTop -
            targetOffset;

          container.scrollTo({ top, behavior: 'smooth' });
          setActiveHref(`#${id}`);
          window.history.replaceState(null, '', `#${id}`);
        }}
      />
      {extra ? <div className="mt-4">{extra}</div> : null}
    </div>
  );
}

function LeftSider({ topOffset }: { topOffset: number }) {
  const navigate = useNavigate();
  const location = useLocation();

  const selectedKey = useMemo(() => {
    const idx = location.pathname.indexOf('/help/');
    if (idx === -1) return 'getting-started/overview';
    const sub = location.pathname.slice(idx + '/help/'.length).replace(/\/+$/, '');
    return sub || 'getting-started/overview';
  }, [location.pathname]);

  const [openKeys, setOpenKeys] = useState<string[]>(() => [selectedKey.split('/')[0]]);
  useEffect(() => {
    const top = selectedKey.split('/')[0];
    setOpenKeys((prev) => (prev.includes(top) ? prev : [top]));
  }, [selectedKey]);

  return (
    <Sider
      width={280}
      collapsedWidth={0}
      breakpoint="lg"
      className="!bg-white dark:!bg-gray-950 border-r border-gray-200 dark:border-gray-800 !sticky !p-0"
      style={{ top: topOffset, height: `calc(100vh - ${topOffset}px)` }}
    >
      <div className="h-full flex flex-col overflow-hidden">
        {/* Removed the 'Browse topics' header */}
        <div className="flex-1 min-h-0 overflow-y-auto no-scrollbar !pb-24">
          <Menu
            mode="inline"
            items={HELP_MENU_ITEMS}
            selectedKeys={[selectedKey]}
            openKeys={openKeys}
            onOpenChange={(keys) => setOpenKeys(keys as string[])}
            onClick={({ key }) => navigate(`/help/${key}`)}
            className="!bg-transparent !p-0"
            style={{ border: 'none' }}
          />
          <div className="h-4" aria-hidden />
        </div>
      </div>
    </Sider>
  );
}

function PrevNextBar() {
  const { isMobile } = useUI();
  const location = useLocation();
  const navigate = useNavigate();

  const current = useMemo(() => {
    const idx = location.pathname.indexOf('/help/');
    return idx === -1
      ? 'getting-started/overview'
      : location.pathname.slice(idx + '/help/'.length).replace(/\/+$/, '') ||
          'getting-started/overview';
  }, [location.pathname]);

  const i = HELP_LEAF_ROUTES.findIndex((r) => r.key === current);
  const prev = i > 0 ? HELP_LEAF_ROUTES[i - 1] : null;
  const next = i >= 0 && i < HELP_LEAF_ROUTES.length - 1 ? HELP_LEAF_ROUTES[i + 1] : null;

  if (!prev && !next) return null;

  if (isMobile) {
    return (
      <div className="mt-6 flex flex-col gap-3">
        {prev ? (
          <Button block icon={<LeftOutlined />} onClick={() => navigate(`/help/${prev.key}`)}>
            Previous: {prev.label}
          </Button>
        ) : null}
        {next ? (
          <Button
            block
            type="primary"
            onClick={() => navigate(`/help/${next.key}`)}
            icon={<RightOutlined />}
          >
            Next: {next.label}
          </Button>
        ) : null}
      </div>
    );
  }

  return (
    <div className="mt-8 flex flex-wrap items-center justify-between gap-2">
      {prev ? (
        <Button icon={<LeftOutlined />} onClick={() => navigate(`/help/${prev.key}`)}>
          Previous: {prev.label}
        </Button>
      ) : (
        <span />
      )}
      {next ? (
        <Button
          type="primary"
          onClick={() => navigate(`/help/${next.key}`)}
          icon={<RightOutlined />}
        >
          Next: {next.label}
        </Button>
      ) : null}
    </div>
  );
}

function Shell() {
  const { isMobile } = useUI();
  const { isDarkMode } = useTheme();
  const location = useLocation();
  const navigate = useNavigate();

  const atMobileHelpIndex = isMobile && /^\/help\/?$/.test(location.pathname);

  const headerRef = useRef<HTMLDivElement | null>(null);
  const [headerOffset, setHeaderOffset] = useState<number>(80);
  useEffect(() => {
    const update = () => {
      if (headerRef.current) setHeaderOffset(headerRef.current.offsetHeight);
    };
    update();
    window.addEventListener('resize', update);
    return () => window.removeEventListener('resize', update);
  }, []);

  const logoSrc = isDarkMode ? '/ff_logo_dark.svg' : '/ff_logo_light.svg';

  const quickLinks = useMemo(
    () => [
      { key: 'dashboard', label: 'Dashboard', to: '/dashboard' },
      { key: 'modules', label: 'Modules', to: '/modules' },
      { key: 'assignments', label: 'Assignments Help', to: '/help/assignments/setup' },
      { key: 'contact', label: 'Contact', to: '/help/support/contact' },
    ],
    [],
  );

  return (
    <div className="h-screen flex flex-col bg-white dark:bg-gray-950">
      <Header
        ref={headerRef as any}
        className="flex items-center justify-between gap-4 border-b border-gray-200 dark:border-gray-800 px-4 md:px-8 !py-2 sticky top-0 z-40 !bg-white dark:!bg-gray-950"
        style={{ height: 'auto', lineHeight: 'normal' }}
      >
        <div className="flex items-center gap-3 min-w-0">
          <img src={logoSrc} alt="FitchFork logo" className="h-9 w-9 shrink-0" />
          <span className="truncate text-base sm:text-lg md:text-xl font-semibold text-gray-900 dark:text-gray-100">
            Help Center
          </span>
        </div>

        <div className="hidden sm:flex flex-wrap gap-2 justify-end">
          {quickLinks.map((link) => (
            <Button
              key={link.key}
              type={link.key === 'contact' ? 'primary' : 'default'}
              onClick={() => navigate(link.to)}
            >
              {link.label}
            </Button>
          ))}
        </div>
      </Header>

      <Layout className="flex-1 flex !bg-white dark:!bg-gray-950 !min-h-0 overflow-hidden">
        {!isMobile && <LeftSider topOffset={headerOffset} />}

        <Layout className="flex-1 !bg-white dark:!bg-gray-950 !min-h-0 overflow-hidden">
          <Content className="flex-1 !min-h-0 overflow-hidden">
            <div
              id="help-scroll-container"
              className="overflow-y-auto"
              style={{
                height: `calc(100vh - ${headerOffset}px)`,
                scrollBehavior: 'auto',

                scrollPaddingTop: headerOffset,
                scrollPaddingBottom: atMobileHelpIndex ? 16 : 120,
              }}
            >
              <div className="grid grid-cols-1 lg:grid-cols-[minmax(0,960px)_240px]">
                <div className="px-4 md:px-8 pt-4 pb-12">
                  <Outlet />
                  {!atMobileHelpIndex && <PrevNextBar />}
                </div>

                {/* RIGHT COLUMN: full-height vertical rule + sticky anchor */}
                <div
                  className="relative hidden lg:block"
                  style={{ minHeight: `calc(100vh - ${headerOffset}px)` }}
                >
                  {/* Full-height border that aligns exactly under the header */}
                  <div className="absolute inset-y-0 left-0 w-px bg-gray-200 dark:bg-gray-800" />

                  {/* Only the contents are sticky; sits tight under the header */}
                  <div className="sticky" style={{ top: Math.max(0, headerOffset - 45) }}>
                    <div className="max-h-[calc(100vh- var(--hdr))]">
                      <RightOfContentAnchor />
                    </div>
                  </div>
                </div>
                {/* NOTE: The absolute border above removes the gap you saw */}
              </div>
            </div>
          </Content>
        </Layout>
      </Layout>
    </div>
  );
}

export default function HelpPageLayout() {
  return (
    <HelpProvider>
      <Shell />
    </HelpProvider>
  );
}
