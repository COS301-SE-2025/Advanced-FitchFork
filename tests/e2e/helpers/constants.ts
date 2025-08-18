import path from 'node:path';

export const PW_DIR = path.join(__dirname, '..', '.playwright');
export const AUTH_DIR = path.join(PW_DIR, 'auth');
export const REPORT_DIR = path.join(PW_DIR, 'reports');
export const RESULTS_DIR = path.join(PW_DIR, 'results');
