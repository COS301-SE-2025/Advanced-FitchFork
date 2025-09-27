import { api } from '@/utils/api';

export type MaxConcurrent = number;

export function getMaxConcurrent() {
  return api.get<MaxConcurrent>('/system/code-manager/max-concurrent');
}

export function setMaxConcurrent(max: number) {
  return api.post<MaxConcurrent>('/system/code-manager/max-concurrent', {
    max_concurrent: max,
  });
}