// Shared helpers for k6 scenarios.
// Env: API_URL, API_KEY (pick noquota or quota outside), PROXY_PATH.
import http from 'k6/http';
import { check } from 'k6';

export const API_URL = __ENV.API_URL || 'http://localhost:8080';
export const API_KEY = __ENV.API_KEY || '';
export const PROXY_PATH = __ENV.PROXY_PATH || '/proxy/mock/ping';

if (!API_KEY) throw new Error('API_KEY env is required');

export function hit() {
  const res = http.get(`${API_URL}${PROXY_PATH}`, {
    headers: { 'x-api-key': API_KEY },
    tags: { endpoint: 'proxy' },
  });
  check(res, { 'status 200': (r) => r.status === 200 });
  return res;
}
