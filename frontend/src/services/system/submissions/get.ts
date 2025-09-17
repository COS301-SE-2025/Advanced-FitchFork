import { api, apiDownload, buildQuery } from '@/utils/api';

export type SubsBucket = 'hour' | 'day' | 'week' | 'month' | 'year';

export interface SubmissionsPoint { period: string; count: number }
export interface SubmissionsResponse { points: SubmissionsPoint[] }

export function getSubmissionsOverTime(params: { start?: string; end?: string; bucket?: SubsBucket; assignment_id?: number }) {
  return api.get<SubmissionsResponse>('/system/submissions', params);
}

export function exportSubmissionsOverTime(params: { start?: string; end?: string; bucket?: SubsBucket; assignment_id?: number }) {
  const query = params ? buildQuery(params) : '';
  const endpoint = query ? `/system/submissions/export?${query}` : '/system/submissions/export';
  return apiDownload(endpoint);
}
