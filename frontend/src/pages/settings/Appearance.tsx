import { CompressOutlined, ColumnHeightOutlined } from '@ant-design/icons';
import { Divider, Switch, Tooltip } from 'antd';
import SettingsGroup from '@/components/SettingsGroup';
import PageHeader from '@/components/PageHeader';
import { useTheme } from '@/context/ThemeContext';
import { useUI } from '@/context/UIContext';

const ThemeOption = ({
  value,
  selected,
  onClick,
  children,
}: {
  value: string;
  selected: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) => {
  const borderColor = selected ? '#1677ff' : 'transparent';
  const label =
    value === 'light' ? 'Light Theme' : value === 'dark' ? 'Dark Theme' : 'Follow System Theme';

  return (
    <Tooltip title={label}>
      <div
        onClick={onClick}
        className="rounded-lg overflow-hidden transition cursor-pointer w-40 h-28 border-1"
        style={{
          borderColor,
          boxShadow: selected ? `0 0 0 2px ${borderColor}` : undefined,
        }}
      >
        {children}
      </div>
    </Tooltip>
  );
};

const FakeWindow = ({ variant }: { variant: 'light' | 'dark' }) => {
  const isDark = variant === 'dark';
  const base = isDark ? 'bg-gray-900 text-white' : 'bg-white text-black';
  const bar = isDark ? 'bg-gray-700/40' : 'bg-gray-300/50';
  const block1 = isDark ? 'bg-gray-700' : 'bg-gray-200';
  const block2 = isDark ? 'bg-gray-600' : 'bg-gray-100';

  return (
    <div className={`h-full w-full flex flex-col ${base}`}>
      <div className={`h-3 ${bar} flex gap-1 p-1`}>
        <div className="h-2 w-2 rounded-full bg-red-400" />
        <div className="h-2 w-2 rounded-full bg-yellow-400" />
        <div className="h-2 w-2 rounded-full bg-green-400" />
      </div>
      <div className="flex-1 grid grid-cols-3 gap-1 p-1">
        <div className={`${block1} col-span-1 rounded-sm`} />
        <div className={`${block2} col-span-2 rounded-sm`} />
      </div>
    </div>
  );
};

const SystemWindow = () => (
  <div className="relative w-full h-full">
    <div className="absolute inset-0 z-0">
      <FakeWindow variant="dark" />
    </div>
    <div
      className="absolute inset-0 z-10"
      style={{
        WebkitMaskImage: 'linear-gradient(135deg, black 50%, transparent 50%)',
        maskImage: 'linear-gradient(135deg, black 50%, transparent 50%)',
        WebkitMaskSize: '100% 100%',
        maskSize: '100% 100%',
        WebkitMaskRepeat: 'no-repeat',
        maskRepeat: 'no-repeat',
      }}
    >
      <FakeWindow variant="light" />
    </div>
  </div>
);

const Appearance = () => {
  const { mode, setMode } = useTheme();
  const { compact, setCompact, motion, setMotion } = useUI();

  return (
    <div className="w-full max-w-6xl space-y-12 bg-gray-50 dark:bg-gray-950">
      <PageHeader title="Appearance" description="Customize how the interface looks and feels." />

      <SettingsGroup
        title="Theme"
        description="Choose between light and dark mode or follow your system preference."
      >
        <div className="flex gap-4">
          <ThemeOption
            value="system"
            selected={mode === 'system'}
            onClick={() => setMode('system')}
          >
            <SystemWindow />
          </ThemeOption>

          <ThemeOption value="light" selected={mode === 'light'} onClick={() => setMode('light')}>
            <FakeWindow variant="light" />
          </ThemeOption>

          <ThemeOption value="dark" selected={mode === 'dark'} onClick={() => setMode('dark')}>
            <FakeWindow variant="dark" />
          </ThemeOption>
        </div>
      </SettingsGroup>
      <Divider />
      <SettingsGroup
        title="Interface Density"
        description="Adjust spacing and sizing for a more comfortable experience."
      >
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          {[
            { key: false, label: 'Comfortable', icon: ColumnHeightOutlined },
            { key: true, label: 'Compact', icon: CompressOutlined },
          ].map(({ key, label, icon: Icon }) => {
            const isSelected = compact === key;
            return (
              <div
                key={String(key)}
                onClick={() => setCompact(key)}
                className={`cursor-pointer p-4 rounded-xl border transition ${
                  isSelected
                    ? 'border-blue-500 ring-2 ring-blue-500'
                    : 'border-gray-200 dark:border-gray-800'
                } hover:bg-gray-100 dark:hover:bg-gray-800`}
              >
                <div className="flex items-center gap-3">
                  <Icon className="text-lg" />
                  <div>
                    <div className="text-sm font-semibold">{label}</div>
                    <div className="text-xs text-gray-500 dark:text-gray-400">
                      {key
                        ? 'Condensed layout to fit more info.'
                        : 'Spacious layout with room to breathe.'}
                    </div>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </SettingsGroup>
      <Divider />
      <SettingsGroup
        title="Animations"
        description="Enable or disable interface animations and transitions."
      >
        <div
          onClick={() => setMotion(!motion)}
          className={`flex items-center justify-between p-4 rounded-xl cursor-pointer transition border ${
            motion ? 'border-blue-500 ring-2 ring-blue-500' : 'border-gray-200 dark:border-gray-800'
          } hover:bg-gray-100 dark:hover:bg-gray-800`}
        >
          <div className="flex flex-col">
            <span className="text-sm font-semibold">Enable animations</span>
            <span className="text-xs text-gray-500 dark:text-gray-400">
              Smooth transitions and UI feedback.
            </span>
          </div>
          <Switch checked={motion} onChange={setMotion} />
        </div>
      </SettingsGroup>
    </div>
  );
};

export default Appearance;
