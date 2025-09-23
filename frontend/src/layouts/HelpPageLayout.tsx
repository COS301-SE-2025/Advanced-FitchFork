// src/pages/help/HelpPageLayout.tsx
import { useMemo, useState, useEffect } from 'react';
import { Layout, Menu, Anchor, Button } from 'antd';
import { LeftOutlined, RightOutlined } from '@ant-design/icons';
import { Outlet, useLocation, useNavigate } from 'react-router-dom';
import { HelpProvider, useHelp } from '@/context/HelpContext';
import { HELP_MENU_ITEMS, HELP_LEAF_ROUTES } from '@/pages/help/menuConfig';
import { useUI } from '@/context/UIContext';
import MobilePageHeader from '@/components/common/MobilePageHeader';

const { Sider, Content } = Layout;

function RightOfContentAnchor() {
  const { items, activeHref, setActiveHref, getContainer, targetOffset, extra } = useHelp();

  return (
    <div className="!p-4">
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

function LeftSider() {
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
      className="!bg-white dark:!bg-gray-950 border-r border-gray-200 dark:border-gray-800 !sticky !top-0 !h-screen !p-0"
    >
      {/* Make the whole sider a column; header stays, menu area flexes + scrolls */}
      <div className="h-full flex flex-col overflow-hidden">
        {/* Header (fixed) */}
        <div className="px-6 pt-5 pb-3">
          <div className="font-semibold text-gray-800 dark:text-gray-100 text-lg">Help Center</div>
          <div className="text-xs text-gray-500 dark:text-gray-400">Docs &amp; Guides</div>
        </div>

        {/* Scroll area — no scrollbar, always fully scrollable, extra bottom padding so last item isn’t cut */}
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
          {/* small spacer so the last item never kisses the bottom edge */}
          <div className="h-4" aria-hidden />
        </div>
      </div>
    </Sider>
  );
}

// Bottom bar rendered by the LAYOUT. Pages don't know about it.
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
    // MOBILE: vertical, full width
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

  // DESKTOP: existing horizontal layout
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
  const location = useLocation();

  // true only when you're on the mobile help index (MobileHelpPageMenu)
  const atMobileHelpIndex = isMobile && /^\/help\/?$/.test(location.pathname);

  return (
    <Layout className="!bg-white dark:!bg-gray-950 !h-full !min-h-0">
      {!isMobile && <LeftSider />}

      <Layout className="!bg-white dark:!bg-gray-950 flex-1 !min-h-0">
        {isMobile && <MobilePageHeader />}
        <Content className="!min-h-0">
          <div
            id="help-scroll-container"
            className="h-full overflow-y-auto"
            style={{
              scrollBehavior: 'auto',
              scrollPaddingTop: isMobile ? 56 : 12,
              // optional: smaller bottom padding when the bar is hidden
              scrollPaddingBottom: atMobileHelpIndex ? 16 : 120,
            }}
          >
            <div className="grid grid-cols-1 lg:grid-cols-[minmax(0,960px)_240px]">
              <div className="px-4 md:px-8 pt-4 pb-12">
                <Outlet />
                {/* show Prev/Next everywhere EXCEPT on the MobileHelpPageMenu */}
                {!atMobileHelpIndex && <PrevNextBar />}
              </div>

              <div className="hidden lg:block border-l border-gray-200 dark:border-gray-800">
                <div className="!sticky !top-0">
                  <RightOfContentAnchor />
                </div>
              </div>
            </div>
          </div>
        </Content>
      </Layout>
    </Layout>
  );
}

export default function HelpPageLayout() {
  return (
    <HelpProvider>
      <Shell />
    </HelpProvider>
  );
}
