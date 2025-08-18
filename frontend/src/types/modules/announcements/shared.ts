import type { Timestamp } from "@/types/common";

export interface Announcement extends Timestamp {
  id: number;            // BIGINT
  module_id: number;     // FK -> modules.id
  user_id: number;       // FK -> users.id
  title: string;         // TEXT
  body: string;          // TEXT (markdown)
  pinned: boolean;       // BOOLEAN (default false)
}