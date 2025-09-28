// HeaderBar.tsx
import { Button, Dropdown, Typography, Tooltip } from 'antd';
import { MenuOutlined, TeamOutlined } from '@ant-design/icons';
import { useAuth } from '@/context/AuthContext';
import UserAvatar from '../common/UserAvatar';
import BreadcrumbNav from '../common/BreadcrumbNav';
import { useUI } from '@/context/UIContext';
import { scaleColor } from '@/utils/color';

// ✨ NEW: wire to the WS context
import { useWsEvents, Topics, type PayloadOf } from '@/ws';
import React from 'react';
import { CapacityModal } from '../system';

const { Text } = Typography;

type HeaderBarProps = {
  profileMenuItems: any;
  onMenuClick: () => void;
};

const CmTooltip: React.FC<{
  cm?: { running?: number; waiting?: number; max_concurrent?: number | null };
}> = ({ cm }) => {
  const running = typeof cm?.running === 'number' ? cm.running : 0;
  const waiting = typeof cm?.waiting === 'number' ? cm.waiting : 0;
  const max = typeof cm?.max_concurrent === 'number' ? cm.max_concurrent : 0;
  const util = max > 0 ? Math.min(100, Math.round((running / max) * 100)) : 0;

  return (
    <div className="grid gap-1.5 min-w-[220px] p-3">
      <div className="text-[12px] font-semibold">Code manager</div>
      <div className="flex justify-between text-[12px]">
        <span>Running</span>
        <span>{max > 0 ? `${running}/${max}` : running}</span>
      </div>
      <div className="flex justify-between text-[12px]">
        <span>Queue</span>
        <span>{waiting}</span>
      </div>
      {max > 0 && (
        <div className="h-1.5 rounded-full bg-gray-200 dark:bg-gray-700 overflow-hidden mt-1">
          <div
            className="h-full transition-all"
            style={{ width: `${util}%`, backgroundColor: scaleColor(util, 'gray-red') }}
          />
        </div>
      )}
    </div>
  );
};

const loadClass = (v?: number) => {
  if (v == null) return 'text-gray-700 dark:text-gray-200';
  const x = Math.max(0, Math.min(100, v));
  if (x < 50) return 'text-gray-700 dark:text-gray-200';
  if (x < 75) return 'text-orange-500';
  if (x < 90) return 'text-orange-600';
  if (x < 100) return 'text-red-500';
  return 'text-red-600';
};

const HeaderBar = ({ profileMenuItems, onMenuClick }: HeaderBarProps) => {
  const { user, isAdmin } = useAuth();
  const { isMobile } = useUI();

  const [general, setGeneral] = React.useState<PayloadOf<'system.health'> | null>(null);
  const [admin, setAdmin] = React.useState<PayloadOf<'system.health_admin'> | null>(null);
  const [capOpen, setCapOpen] = React.useState(false);

  // Always subscribe to general system health
  useWsEvents([Topics.system()], {
    'system.health': (p) => setGeneral(p),
  });

  // If admin, also subscribe to admin stream (includes max_concurrent)
  useWsEvents(isAdmin ? [Topics.systemAdmin()] : [], {
    'system.health_admin': (p) => setAdmin(p),
  });

  // Prefer admin payload when available, otherwise general
  const payload = isAdmin && admin ? admin : general;

  const load = payload?.load;
  // general payload has {running, waiting}; admin adds {max_concurrent}
  const cm = (payload as any)?.code_manager as
    | { running?: number; waiting?: number; max_concurrent?: number | null }
    | undefined;

  const runningStr =
    typeof cm?.running === 'number' && typeof cm?.max_concurrent === 'number'
      ? `${cm.running}/${cm.max_concurrent}`
      : typeof cm?.running === 'number'
        ? String(cm.running)
        : '--';

  return (
    <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2 sm:gap-0 w-full h-full">
      {/* Mobile: profile + hamburger */}
      {isMobile && (
        <div className="flex items-center justify-between w-full h-full">
          <Dropdown menu={{ items: profileMenuItems }} trigger={['click']} placement="bottomRight">
            <div className="cursor-pointer flex items-center gap-2">
              <UserAvatar user={{ id: user?.id ?? -1, username: user?.username ?? 'User' }} />
              <Text className="text-gray-700 dark:text-gray-200 font-medium">
                {user?.username ?? 'User'}
              </Text>
            </div>
          </Dropdown>

          <Button
            type="text"
            icon={<MenuOutlined />}
            onClick={onMenuClick}
            className="text-gray-700 dark:text-gray-200"
          />
        </div>
      )}

      {/* Breadcrumbs (desktop only) */}
      {!isMobile && <BreadcrumbNav className="hidden sm:flex flex-1" />}

      {/* Desktop: compact health + profile */}
      {!isMobile && (
        <div className="flex items-center gap-4">
          {isAdmin && (
            <>
              <Button size="middle" onClick={() => setCapOpen(true)} icon={<TeamOutlined />}>
                Configure capacity
              </Button>
              <CapacityModal open={capOpen} onClose={() => setCapOpen(false)} />
            </>
          )}
          {/* Compact health panel */}
          <div className="hidden lg:flex items-center mr-2 px-2 py-1 rounded border bg-white dark:bg-gray-900 border-gray-200 dark:border-gray-800 text-[11px]">
            {/* Load averages */}
            <Tooltip title="Load averages (1/5/15 min)">
              <div className="flex items-center gap-2 pr-3">
                <Text className={loadClass(load?.one)}>
                  {load?.one != null ? `${load.one.toFixed(2)}` : '--'}
                </Text>
                <Text className={loadClass(load?.five)}>
                  {load?.five != null ? `${load.five.toFixed(2)}` : '--'}
                </Text>
                <Text className={loadClass(load?.fifteen)}>
                  {load?.fifteen != null ? `${load.fifteen.toFixed(2)}` : '--'}
                </Text>
              </div>
            </Tooltip>

            <div className="w-px h-4 bg-gray-300 dark:bg-gray-700 mx-2" />

            {/* Code manager with tooltip */}
            <Tooltip placement="bottom" title={<CmTooltip cm={cm} />}>
              <div className="flex items-center gap-2 cursor-default">
                <Text className="text-gray-500">Tasks running</Text>
                <Text className="text-gray-700">{runningStr}</Text>
                <Text className="text-gray-700">queue {cm?.waiting ?? '--'}</Text>
              </div>
            </Tooltip>
          </div>

          <Dropdown menu={{ items: profileMenuItems }} trigger={['click']} placement="bottomRight">
            <div className="cursor-pointer flex items-center gap-2 flex-row-reverse">
              <UserAvatar user={{ id: user?.id ?? -1, username: user?.username ?? 'User' }} />
              <Text className="hidden sm:inline text-gray-700 dark:text-gray-200 font-medium">
                {user?.username ?? 'User'}
              </Text>
            </div>
          </Dropdown>
        </div>
      )}
    </div>
  );
};

export default HeaderBar;
