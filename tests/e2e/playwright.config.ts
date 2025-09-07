import { defineConfig, devices } from '@playwright/test';
import path from 'node:path';

const baseURL = process.env.E2E_BASE_URL ?? 'http://localhost:5173';
const apiURL  = process.env.E2E_API_URL  ?? 'http://localhost:3000';

// Centralized Playwright artifact directory
const PW_DIR       = path.join(__dirname, '.playwright');
const AUTH_DIR     = path.join(PW_DIR, 'auth');
const REPORT_DIR   = path.join(PW_DIR, 'reports');
const RESULTS_DIR  = path.join(PW_DIR, 'results');

export default defineConfig({
  testDir: './tests',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,

  reporter: [
    ['html', { open: 'never', outputFolder: REPORT_DIR }],
    ['list'],
  ],

  outputDir: RESULTS_DIR,

  use: {
    baseURL,
    // If you want every test to be logged in as admin by default, uncomment:
    // storageState: path.join(AUTH_DIR, 'admin.json'),
    locale: 'en-US',
    trace: 'retain-on-failure',
    video: 'retain-on-failure',
    screenshot: 'only-on-failure',
  },

  expect: { timeout: 10_000 },

  globalSetup: path.join(__dirname, 'setup', 'global-setup.ts'),
  globalTeardown: path.join(__dirname, 'setup', 'global-teardown.ts'),

  projects: [
    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        viewport: { width: 1920, height: 1080 },
      },
      testMatch: ['**/*.spec.ts'],
    },
    {
      name: 'firefox',
      use: {
        ...devices['Desktop Firefox'],
        viewport: { width: 1920, height: 1080 },
      },
      testMatch: ['**/*.spec.ts'],
    },
    {
      name: 'webkit',
      use: {
        ...devices['Desktop Safari'],
        viewport: { width: 1920, height: 1080 },
      },
      testMatch: ['**/*.spec.ts'],
    },
    {
      name: 'mobile',
      use: { ...devices['Pixel 7'] },
      testMatch: ['**/*.spec.ts'],
    },
  ],
});
