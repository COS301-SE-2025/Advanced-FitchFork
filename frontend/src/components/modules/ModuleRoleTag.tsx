import { Tag } from 'antd';
import type { FC } from 'react';
import type { ModuleRole } from '@/types/modules';

interface Props {
  role: ModuleRole;
  bordered?: boolean;
  asAction?: boolean;
}

const roleColors: Record<ModuleRole, string> = {
  lecturer: 'volcano',
  assistant_lecturer: 'purple',
  tutor: 'geekblue',
  student: 'green',
};

// Default noun labels
export const roleLabels: Record<ModuleRole, string> = {
  lecturer: 'Lecturer',
  assistant_lecturer: 'Assistant Lecturer',
  tutor: 'Tutor',
  student: 'Student',
};

// Action-oriented labels
export const roleActionLabels: Record<ModuleRole, string> = {
  lecturer: 'Lecturing',
  assistant_lecturer: 'Assisting',
  tutor: 'Tutoring',
  student: 'Enrolled',
};

const ModuleRoleTag: FC<Props> = ({ role, bordered = false, asAction = false }) => {
  const label = asAction ? roleActionLabels[role] : roleLabels[role];

  return (
    <Tag color={roleColors[role]} bordered={bordered}>
      {label}
    </Tag>
  );
};

export default ModuleRoleTag;
