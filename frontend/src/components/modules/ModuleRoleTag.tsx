import { Tag } from 'antd';
import type { FC } from 'react';
import type { ModuleRole } from '@/types/modules';

interface Props {
  role: ModuleRole;
  bordered?: boolean;
}

const roleColors: Record<ModuleRole, string> = {
  lecturer: 'volcano',
  assistant_lecturer: 'purple',
  tutor: 'geekblue',
  student: 'green',
};

export const roleLabels: Record<ModuleRole, string> = {
  lecturer: 'Lecturer',
  assistant_lecturer: 'Assistant Lecturer',
  tutor: 'Tutor',
  student: 'Student',
};

const ModuleRoleTag: FC<Props> = ({ role, bordered = false }) => {
  return (
    <Tag color={roleColors[role]} bordered={bordered}>
      {roleLabels[role]}
    </Tag>
  );
};

export default ModuleRoleTag;
