import { useCallback } from 'react';
import { App, Dropdown } from 'antd';
import { ExclamationCircleFilled } from '@ant-design/icons';
import { useAssignment } from '@/context/AssignmentContext';

type Props = {
  primaryText: string;
  // Allow any promise result (boolean, void, etc.)
  onPrimary: () => void | Promise<unknown>;
  disabled?: boolean;
  resetTitle?: string;
  resetContent?: string;
};

export default function AssignmentConfigActions({
  primaryText,
  onPrimary,
  disabled,
  resetTitle = 'Reset configuration?',
  resetContent = 'This will overwrite the current execution config (and the rest) with system defaults.',
}: Props) {
  const { modal, message } = App.useApp();
  const { resetConfig } = useAssignment();

  const handleMenuClick = useCallback(
    ({ key }: { key: string }) => {
      if (key !== 'reset') return;

      modal.confirm({
        title: resetTitle,
        icon: <ExclamationCircleFilled />,
        content: resetContent,
        okText: 'Reset',
        cancelText: 'Cancel',
        okButtonProps: { danger: true },
        centered: true, // center the modal
        async onOk() {
          await resetConfig();
          message.success('Configuration reset to defaults');
        },
      });
    },
    [modal, message, resetConfig, resetContent, resetTitle],
  );

  return (
    <Dropdown.Button
      type="primary"
      disabled={disabled}
      // swallow any returned promise/boolean to satisfy the signature
      onClick={() => {
        void onPrimary();
      }}
      menu={{
        items: [{ key: 'reset', danger: true, label: 'Reset to Defaults' }],
        onClick: handleMenuClick,
      }}
    >
      {primaryText}
    </Dropdown.Button>
  );
}
