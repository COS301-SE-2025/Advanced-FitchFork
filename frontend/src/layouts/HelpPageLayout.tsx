// src/pages/help/HelpPageLayout.tsx
import { useMemo, useState, useEffect } from 'react';
import { Layout, Menu, Anchor, Button } from 'antd';
import {
  InfoCircleOutlined,
  AppstoreOutlined,
  FileTextOutlined,
  SettingOutlined,
  LeftOutlined,
  RightOutlined,
} from '@ant-design/icons';
import { Outlet, useLocation, useNavigate } from 'react-router-dom';
import { HelpProvider, useHelp } from '@/context/HelpContext';

const { Sider, Content } = Layout;

type MenuItem = Required<NonNullable<Parameters<typeof Menu>[0]['items']>>[number];

// Flatten leaf routes in visual order
type Leaf = { key: string; label: string };
function flattenLeafs(items: MenuItem[] | undefined): Leaf[] {
  const out: Leaf[] = [];
  const walk = (arr?: any[]) => {
    if (!arr) return;
    for (const it of arr) {
      if (!it) continue;
      if (it.type === 'group') {
        walk(it.children);
        continue;
      }
      if (Array.isArray(it.children) && it.children.length) {
        walk(it.children);
        continue;
      }
      if (typeof it.key === 'string' && typeof it.label === 'string') {
        out.push({ key: it.key, label: it.label });
      }
    }
  };
  walk(items);
  return out;
}

// ===================== MENU =====================
const MENU_ITEMS: MenuItem[] = [
  {
    key: 'getting-started',
    icon: <InfoCircleOutlined />,
    label: 'Getting Started',
    children: [{ key: 'getting-started/overview', label: 'Overview' }],
  },
  {
    key: 'modules',
    icon: <AppstoreOutlined />,
    label: 'Modules',
    children: [
      { key: 'modules/overview', label: 'Module Overview' },
      { key: 'modules/announcements', label: 'Announcements' },
      { key: 'modules/attendance', label: 'Attendance' },
      { key: 'modules/grades', label: 'Module Grades' },
      { key: 'modules/personnel', label: 'Personnel & Roles' },
    ],
  },
  {
    key: 'assignments',
    icon: <FileTextOutlined />,
    label: 'Assignments',
    children: [
      // --- Setup ---
      {
        type: 'group',
        label: 'Setup',
        children: [{ key: 'assignments/setup', label: 'Full Setup Guide' }],
      },

      // --- Assignment Config (collapsible) ---
      {
        key: 'assignments/config-sections',
        label: 'Assignment Config',
        children: [
          { key: 'assignments/config/overview', label: 'Overview' },
          { key: 'assignments/config/project', label: 'Language & Mode' },
          { key: 'assignments/config/execution', label: 'Execution' },
          { key: 'assignments/config/output', label: 'Output' },
          { key: 'assignments/config/marking', label: 'Marking' },
          { key: 'assignments/config/security', label: 'Security' },
          { key: 'assignments/config/gatlam', label: 'GATLAM' },
        ],
      },

      // --- Files ---
      {
        type: 'group',
        label: 'Files',
        children: [
          { key: 'assignments/files/main-files', label: 'Main File' },
          { key: 'assignments/files/makefile', label: 'Makefile' },
          { key: 'assignments/files/memo-files', label: 'Memo Files' },
          { key: 'assignments/files/specification', label: 'Specification' },
        ],
      },

      // --- Concepts (now includes Code Coverage + GATLAM) ---
      {
        type: 'group',
        label: 'Concepts',
        children: [
          { key: 'assignments/tasks', label: 'Tasks' },
          { key: 'assignments/code-coverage', label: 'Code Coverage' },
          { key: 'assignments/gatlam', label: 'GATLAM & Interpreter' },
        ],
      },

      // --- Submissions (own group) ---
      {
        type: 'group',
        label: 'Submissions',
        children: [
          { key: 'assignments/submissions/how-to-submit', label: 'How to Submit' },
        ],
      },

      // --- Plagiarism ---
      {
        type: 'group',
        label: 'Plagiarism',
        children: [{ key: 'assignments/plagiarism/moss', label: 'Plagiarism & MOSS' }],
      },

      // --- Grading (own group) ---
      {
        type: 'group',
        label: 'Grading',
        children: [
          { key: 'assignments/memo-output', label: 'Memo Output' },
          { key: 'assignments/mark-allocator', label: 'Mark Allocation' },
        ],
      },
    ],
  },
  {
    key: 'support',
    icon: <SettingOutlined />,
    label: 'Support',
    children: [
      { key: 'support/troubleshooting', label: 'Troubleshooting' },
      { key: 'support/system-monitoring', label: 'System Monitoring' },
      { key: 'support/contact', label: 'Contact' },
    ],
  },
];

const LEAF_ROUTES: Leaf[] = flattenLeafs(MENU_ITEMS);

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
            items={MENU_ITEMS}
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
  const location = useLocation();
  const navigate = useNavigate();

  const current = useMemo(() => {
    const idx = location.pathname.indexOf('/help/');
    return idx === -1
      ? 'getting-started/overview'
      : location.pathname.slice(idx + '/help/'.length).replace(/\/+$/, '') ||
          'getting-started/overview';
  }, [location.pathname]);

  const i = LEAF_ROUTES.findIndex((r) => r.key === current);
  const prev = i > 0 ? LEAF_ROUTES[i - 1] : null;
  const next = i >= 0 && i < LEAF_ROUTES.length - 1 ? LEAF_ROUTES[i + 1] : null;

  if (!prev && !next) return null;

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
  return (
    <Layout className="!bg-white dark:!bg-gray-950 !h-full !min-h-0">
      <LeftSider />

      <Layout className="!bg-white dark:!bg-gray-950 flex-1 !min-h-0">
        <Content className="!min-h-0">
          <div
            id="help-scroll-container"
            className="h-full overflow-y-auto"
            style={{ scrollBehavior: 'auto', scrollPaddingTop: 12, scrollPaddingBottom: 120 }}
          >
            <div className="grid grid-cols-[minmax(0,960px)_240px]">
              <div className="px-5 md:px-8 pt-4 pb-12">
                <Outlet />
                <PrevNextBar />
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
