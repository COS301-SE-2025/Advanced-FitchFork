import { createContext, useContext } from 'react';
import { useOutletContext } from 'react-router-dom';
import type { FormInstance } from 'antd';
import { Button, Dropdown } from 'antd';
import {
  EllipsisOutlined,
  DownloadOutlined,
  RollbackOutlined,
  SaveOutlined,
} from '@ant-design/icons';
import type { AssignmentConfig } from '@/types/modules/assignments/config';

export type AssignmentConfigCtx = {
  form: FormInstance<AssignmentConfig>;
  rawView: boolean;
  setRawView: (v: boolean) => void;
  loading: boolean;
  save: () => Promise<void>;
  revert: () => void;
  download: () => void;
  syncFormToRaw: () => void;
  syncRawToForm: () => void;
  rawText: string;
  setRawText: (t: string) => void;
};

const ReactCtx = createContext<AssignmentConfigCtx | null>(null);
export const AssignmentConfigProvider = ReactCtx.Provider;

export const useAssignmentConfig = (): AssignmentConfigCtx => {
  const viaReact = useContext(ReactCtx);
  if (viaReact) return viaReact;
  return useOutletContext<AssignmentConfigCtx>();
};

interface ConfigActionsProps {
  className?: string;
  saveLabel?: string;
}

export function ConfigActions({ className, saveLabel }: ConfigActionsProps) {
  const { save, revert, download, loading } = useAssignmentConfig();

  return (
    <div className={`pt-4 flex justify-start ${className ?? ''}`}>
      <Dropdown.Button
        type="primary"
        onClick={save}
        className="w-full sm:min-w-fit"
        buttonsRender={() => {
          return [
            <Button
              key="save"
              type="primary"
              loading={loading}
              icon={<SaveOutlined />}
              className="w-full sm:w-auto"
              onClick={save}
            >
              {saveLabel ?? 'Save Config'}
            </Button>,
            <Button key="more" type="primary" icon={<EllipsisOutlined />} />,
          ];
        }}
        menu={{
          items: [
            {
              key: 'download',
              icon: <DownloadOutlined />,
              label: 'Download as File',
              onClick: download,
            },
            { type: 'divider' as const },
            {
              key: 'revert',
              icon: <RollbackOutlined />,
              label: 'Revert to Default',
              danger: true,
              onClick: revert,
            },
          ],
        }}
        placement="bottomRight"
      />
    </div>
  );
}
