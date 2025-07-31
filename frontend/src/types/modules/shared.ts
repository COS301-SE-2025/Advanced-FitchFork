import type { Timestamp } from "@/types/common";

export const MODULE_ROLES = [
  "lecturer",
  "assistant_lecturer",
  "tutor",
  "student"
] as const;

export type ModuleRole = typeof MODULE_ROLES[number];

export interface Module extends Timestamp {
  readonly id: number;
  code: string;
  year: number;
  description: string;
  credits: number;
}
