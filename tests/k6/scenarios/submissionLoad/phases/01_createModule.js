import { authorizedPost } from '../../../shared/http.js';
import { BASE_URL } from '../../../shared/config.js';
import { check } from 'k6';


export function createModule(token) {
  const code = 'MOD' + String(Math.floor(100 + Math.random() * 900)); // MOD + 3-digit number

  const module = {
    code,
    year: 2025,
    description: 'k6 stress test',
    credits: 12
  };

  const res = authorizedPost(`${BASE_URL}/modules`, module, token);
  check(res, { 'module created': (r) => r.status === 201 });

  const json = res.json();
  if (!json?.data?.id) {
    console.error('Failed to create module:', JSON.stringify(json));
    throw new Error('Module creation failed: Missing ID');
  }

  return json.data.id;
}
