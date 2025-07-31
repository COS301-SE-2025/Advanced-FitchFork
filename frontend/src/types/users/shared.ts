import type { Timestamp } from "@/types/common";

export interface User extends Timestamp {
  id: number;
  username: string;
  email: string;
  admin: boolean;
}