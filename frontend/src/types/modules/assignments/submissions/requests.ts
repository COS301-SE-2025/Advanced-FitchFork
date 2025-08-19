
// ─────────────────────────────────────────────────────────────
// POST Requests Types
// ─────────────────────────────────────────────────────────────

export type RemarkRequest =
  | { submission_ids: number[]; all?: undefined }
  | { all: true; submission_ids?: undefined };
