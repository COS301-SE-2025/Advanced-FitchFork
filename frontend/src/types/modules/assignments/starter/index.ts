export type StarterId = string;

export interface StarterPack {
  id: StarterId;                 // matches assets/starters/<id>/
  name: string;                  // human-friendly label
  language: string;              // e.g. "cpp", "java"
  description?: string;
  tags?: string[];
}

export interface CreateStarterRequest {
  id: StarterId;
}
