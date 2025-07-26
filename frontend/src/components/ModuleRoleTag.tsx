import { Tag } from 'antd';
import type { ModuleRole } from '@/types/modules';

interface Props {
  role: ModuleRole;
}

const roleColors: Record<ModuleRole, string> = {
  lecturer: 'purple',
  tutor: 'blue',
  student: 'green',
  assistant_lecturer: 'pink',
};

function capitalize(word: string) {
  return word.charAt(0).toUpperCase() + word.slice(1);
}

export default function ModuleRoleTag({ role }: Props) {
  return <Tag color={roleColors[role]}>{capitalize(role)}</Tag>;
}
