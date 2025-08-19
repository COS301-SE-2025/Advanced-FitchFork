import type { Language } from './languages';

export const STARTER_BASE_PATH = '/assignment-defaults';

/**
 * Build language-specific starter file path
 */
export const starterPath = (
  lang: Language,
  file: 'main' | 'memo' | 'makefile',
): string => `${STARTER_BASE_PATH}/${lang}/${file}.zip`;
