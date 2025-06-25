import type { Timestamp } from "@/types/common";

export interface User extends Timestamp {
  id: number;
  student_number: string;
  email: string;
  admin: boolean;
}