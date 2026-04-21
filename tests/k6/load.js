// Load: ramping VUs to 50, hold, ramp down. Target: p95/p99 under realistic load.
import { hit } from './common.js';

export const options = {
  scenarios: {
    ramping: {
      executor: 'ramping-vus',
      startVUs: 1,
      stages: [
        { duration: '15s', target: 10 },
        { duration: '30s', target: 50 },
        { duration: '60s', target: 50 },
        { duration: '15s', target: 0 },
      ],
      gracefulRampDown: '5s',
    },
  },
  thresholds: {
    http_req_failed: ['rate<0.01'],
    http_req_duration: ['p(95)<100', 'p(99)<200'],
  },
};

export default function () {
  hit();
}
