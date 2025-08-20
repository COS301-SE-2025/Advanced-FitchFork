import http from 'k6/http';
import { check } from 'k6';
import { BASE_URL } from './config.js';

export function login({ username, password }) {
  const res = http.post(`${BASE_URL}/auth/login`, JSON.stringify({ username, password }), {
    headers: { 'Content-Type': 'application/json' },
  });

  check(res, { 'login success': (r) => r.status === 200 });
  return res.json()?.data?.token;
}

export function register(username, email, password = '1') {
  const res = http.post(`${BASE_URL}/auth/register`, JSON.stringify({ username, email, password }), {
    headers: { 'Content-Type': 'application/json' },
  });

  check(res, {
    'register status 201 or 409': (r) => r.status === 201 || r.status === 409,
  });

  return res;
}