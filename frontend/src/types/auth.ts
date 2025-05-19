
export const UserRole = {
  Admin: 'admin',
  Editor: 'editor',
  Viewer: 'viewer',
} as const;

export type UserRole = (typeof UserRole)[keyof typeof UserRole];

export interface User {
  username: string;
  roles: UserRole[];
  token: string;
}
