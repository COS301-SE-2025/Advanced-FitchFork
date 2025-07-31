import { defineConfig } from 'cypress';
import webpack from '@cypress/webpack-preprocessor';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export default defineConfig({
  e2e: {
    baseUrl: 'http://localhost:5173',
    supportFile: 'cypress/support/e2e.js',
    fixturesFolder: 'cypress/fixtures',
    specPattern: 'cypress/e2e/**/*.cy.{js,tsx}',
    setupNodeEvents(on, config) {
      on(
        'file:preprocessor',
        webpack({
          webpackOptions: {
            resolve: {
              alias: {
                '@utils': path.resolve(__dirname, 'cypress/support/commands/utils'),
              },
              extensions: ['.js', '.ts', '.json'],
            },
          },
        })
      );
    },
  },
  screenshotsFolder: './cypress/screenshots',
  viewportWidth: 1920,
  viewportHeight: 1080,
  experimentalMemoryManagement: true,
  numTestsKeptInMemory: 5
});
