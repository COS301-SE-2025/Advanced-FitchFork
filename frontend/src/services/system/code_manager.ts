import { api } from '@/utils/api';


export async function getMaxConcurrent() {
  return api.get('/system/code-manager/max-concurrent');
}

export async function setMaxConcurrent(max: number) {
  return api.post('/system/code-manager/max-concurrent', { max_concurrent: max });
}

