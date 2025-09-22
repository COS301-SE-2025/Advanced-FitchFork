import { useEffect } from 'react';
import { Typography } from 'antd';
import { useViewSlot } from '@/context/ViewSlotContext';
import { findHelpLeafLabel } from './menuConfig';

const HeaderText = Typography.Text;

type Options = {
  /** Override the derived label. */
  label?: string;
  /** Custom back target for the mobile header. Defaults to the help menu. */
  backTo?: string | null;
};

export function useHelpPageHeader(routeKey: string, options?: Options) {
  const { setValue, setBackTo } = useViewSlot();

  const derivedLabel = options?.label ?? findHelpLeafLabel(routeKey) ?? routeKey;
  const backTarget = options?.backTo ?? '/help';

  useEffect(() => {
    setValue(
      <HeaderText className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        {derivedLabel}
      </HeaderText>,
    );
    setBackTo(backTarget);

    return () => {
      setValue(null);
      setBackTo(null);
    };
  }, [derivedLabel, backTarget, setValue, setBackTo]);
}
