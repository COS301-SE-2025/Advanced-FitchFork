import type { Timestamp } from "@/types/common";

export type ModuleRole = "Lecturer" | "Tutor" | "Student";
export const MODULE_ROLES: ModuleRole[] = ['Lecturer', 'Tutor', 'Student'];

export interface Module extends Timestamp {
  readonly id: number;
  code: string;
  year: number;
  description: string;
  credits: number;
}