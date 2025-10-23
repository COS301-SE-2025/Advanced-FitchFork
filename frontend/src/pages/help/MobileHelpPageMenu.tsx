import { useEffect, useMemo, type ReactNode } from 'react';
import { HELP_MENU_ITEMS, type HelpMenuItem } from './menuConfig';
import { useViewSlot } from '@/context/ViewSlotContext';
import { Space, Button, Typography, Divider } from 'antd';
import { useNavigate } from 'react-router-dom';
import { RightOutlined } from '@ant-design/icons';

type Leaf = { key: string; label: string };
interface MobileGroup {
  key: string;
  label?: string;
  items: Leaf[];
}
interface MobileSection {
  key: string;
  label: string;
  icon?: ReactNode;
  groups: MobileGroup[];
}

/** Narrow unknown to HelpMenuItem (section) */
function isSection(x: unknown): x is HelpMenuItem {
  return !!x && typeof x === 'object' && 'key' in (x as Record<string, unknown>);
}

/** Convert arbitrary node to Leaf | null (for mapping) */
function toLeaf(item: unknown): Leaf | null {
  if (!item || typeof item !== 'object') return null;
  const key = typeof (item as any).key === 'string' ? (item as any).key : null;
  const label = typeof (item as any).label === 'string' ? (item as any).label : null;
  if (key && label) return { key, label };
  return null;
}

const isLeaf = (value: Leaf | null): value is Leaf => !!value;

function buildGroups(section: HelpMenuItem): MobileGroup[] {
  const groups: MobileGroup[] = [];
  const ungrouped: Leaf[] = [];

  // children can be (HelpMenuItem | {type:'group',...} | Leaf | null | undefined)[]
  const children: unknown[] = Array.isArray((section as any).children)
    ? ((section as any).children as unknown[])
    : [];

  children.forEach((child: unknown, idx: number) => {
    if (!child) return;

    // Group node
    if ((child as any).type === 'group') {
      const rawKids: unknown[] = ((child as any).children ?? []) as unknown[];
      const leafs = rawKids.map(toLeaf).filter(isLeaf);
      if (leafs.length) {
        groups.push({
          key: `${String((section as any).key ?? 'section')}-group-${idx}`,
          label: typeof (child as any).label === 'string' ? (child as any).label : undefined,
          items: leafs,
        });
      }
      return;
    }

    // Collapsible/parent node with children
    if (Array.isArray((child as any).children) && (child as any).children.length) {
      const rawKids: unknown[] = (child as any).children as unknown[];
      const leafs = rawKids.map(toLeaf).filter(isLeaf);
      if (leafs.length) {
        groups.push({
          key:
            typeof (child as any).key === 'string'
              ? (child as any).key
              : `${String((section as any).key ?? 'section')}-group-${idx}`,
          label: typeof (child as any).label === 'string' ? (child as any).label : undefined,
          items: leafs,
        });
      }
      return;
    }

    // Single leaf
    const leaf = toLeaf(child);
    if (leaf) ungrouped.push(leaf);
  });

  if (ungrouped.length) {
    groups.unshift({
      key: `${String((section as any).key ?? 'section')}-root`,
      items: ungrouped,
    });
  }

  return groups;
}

function toMobileSections(items: (HelpMenuItem | null | undefined)[]): MobileSection[] {
  return items
    .filter(isSection)
    .map((section: HelpMenuItem, idx: number) => {
      const groups = buildGroups(section);
      if (!groups.length) return null;

      const keyStr =
        typeof (section as any).key === 'string' ? (section as any).key : `section-${idx}`;

      const labelStr =
        typeof (section as any).label === 'string' ? (section as any).label : 'Section';

      return {
        key: keyStr,
        label: labelStr,
        icon: (section as any).icon,
        groups,
      } as MobileSection;
    })
    .filter((x): x is MobileSection => !!x);
}

const MobileHelpPageMenu = () => {
  const navigate = useNavigate();
  const { setValue, setBackTo } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Help Center
      </Typography.Text>,
    );
    setBackTo(null);

    return () => {
      setBackTo(null);
    };
  }, [setValue, setBackTo]);

  const sections = useMemo(() => toMobileSections(HELP_MENU_ITEMS), []);

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto space-y-6 pb-24">
        {sections.map((section) => (
          <div key={section.key}>
            <div className="flex items-center gap-2 mb-3">
              {section.icon ? (
                <span className="flex items-center justify-center w-9 h-9 rounded-lg bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-300">
                  {section.icon}
                </span>
              ) : null}
              <Typography.Text className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                {section.label}
              </Typography.Text>
            </div>

            {section.groups.map((group, groupIdx) => (
              <div key={group.key} className="w-full">
                {group.label ? (
                  <Typography.Text className="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400 block mb-2">
                    {group.label}
                  </Typography.Text>
                ) : null}

                <Space.Compact direction="vertical" className="w-full">
                  {group.items.map((leaf) => (
                    <Button
                      key={leaf.key}
                      type="default"
                      block
                      className="!h-14 px-4 flex items-center !justify-between text-base"
                      onClick={() => navigate(`/help/${leaf.key}`)}
                    >
                      <Typography.Text className="text-left text-base text-gray-900 dark:text-gray-100">
                        {leaf.label}
                      </Typography.Text>
                      <RightOutlined />
                    </Button>
                  ))}
                </Space.Compact>

                {groupIdx < section.groups.length - 1 ? (
                  <Divider className="!my-3 !bg-gray-200 dark:!bg-gray-800" />
                ) : null}
              </div>
            ))}
          </div>
        ))}
      </div>
    </div>
  );
};

export default MobileHelpPageMenu;
