import { Button, Popconfirm, Space, Typography } from 'antd';
import type { ButtonType } from 'antd/es/button';
import type { ReactNode } from 'react';

export interface QuickAction {
  key: string;
  label: string;
  icon?: ReactNode;
  onClick: () => void;
  type?: ButtonType;
  danger?: boolean;
  confirm?: {
    title: string;
    okText?: string;
    cancelText?: string;
  };
  disabled?: boolean;
}

interface QuickActionsProps {
  actions: QuickAction[];
  type?: ButtonType; // optional override for all buttons
  align?: 'left' | 'center' | 'right'; // controls label + icon alignment
}

export default function QuickActions({ actions, type, align = 'left' }: QuickActionsProps) {
  const justifyClass =
    align === 'center' ? '!justify-center' : align === 'right' ? '!justify-end' : '!justify-start';

  return (
    <Space.Compact direction="vertical" className="w-full">
      {actions.map((action) => {
        const btn = (
          <Button
            key={action.key}
            type={action.type || type || 'default'}
            danger={action.danger}
            onClick={action.onClick}
            disabled={action.disabled}
            className={`!h-14 px-4 flex items-center ${justifyClass} text-base`}
            block
          >
            <Typography.Text
              className={`flex items-center gap-2 ${
                align === 'center'
                  ? 'text-center justify-center w-full'
                  : align === 'right'
                    ? 'flex-row-reverse'
                    : 'text-left'
              }`}
            >
              {action.icon}
              {action.label}
            </Typography.Text>
          </Button>
        );

        return action.confirm ? (
          <Popconfirm
            key={action.key}
            title={action.confirm.title}
            okText={action.confirm.okText || 'Yes'}
            cancelText={action.confirm.cancelText || 'No'}
            onConfirm={action.onClick}
          >
            {btn}
          </Popconfirm>
        ) : (
          btn
        );
      })}
    </Space.Compact>
  );
}
