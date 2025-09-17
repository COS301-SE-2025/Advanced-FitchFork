import { api, apiDownload, buildQuery } from '@/utils/api';

export type MetricsBucket = 'day' | 'week' | 'month' | 'year';

export interface MetricsPoint {
  ts: string;
  cpu_avg: number;
  mem_pct: number;
}

export interface MetricsResponse { points: MetricsPoint[] }

export function getSystemMetrics(params: { start?: string; end?: string; bucket?: MetricsBucket }) {
  return api.get<MetricsResponse>('/system/metrics', params);
}

export function exportSystemMetrics(params: { start?: string; end?: string; bucket?: MetricsBucket }) {
  const query = params ? buildQuery(params) : '';
  const endpoint = query ? `/system/metrics/export?${query}` : '/system/metrics/export';
  return apiDownload(endpoint);
}
