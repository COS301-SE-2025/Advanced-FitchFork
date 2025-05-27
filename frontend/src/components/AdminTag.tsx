// src/components/AdminTag.tsx
import { Tag } from 'antd';

interface AdminTagProps {
  isAdmin: boolean;
}

export default function AdminTag({ isAdmin }: AdminTagProps) {
  return <Tag color={isAdmin ? 'green' : undefined}>{isAdmin ? 'Admin' : 'Regular'}</Tag>;
}
