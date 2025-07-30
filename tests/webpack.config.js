import { resolve as _resolve } from 'path';

export const resolve = {
  alias: {
    '@utils': _resolve(__dirname, 'cypress/support/commands/utils'),
  },
  extensions: ['.js', '.ts', '.json'],
};
