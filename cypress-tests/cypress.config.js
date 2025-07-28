import { defineConfig } from 'cypress';

export default defineConfig({
  e2e: {
    baseUrl: 'http://localhost:5173',
    supportFile: 'support/e2e.js',
    fixturesFolder: 'fixtures',
    specPattern: 'e2e/**/*.cy.{js,tsx}',
    setupNodeEvents(on, config) {
      // optional
    },
  },
  screenshotsFolder: './screenshots',
  viewportWidth: 1920,
  viewportHeight: 1080,
});
