// Smoke: single VU, 30s. Sanity + warm steady-state p50.
import { hit } from './common.js';

export const options = {
  vus: 1,
  duration: '30s',
  thresholds: {
    http_req_failed: ['rate<0.01'],
    http_req_duration: ['p(95)<50'],
  },
};

export default function () {
  hit();
}
