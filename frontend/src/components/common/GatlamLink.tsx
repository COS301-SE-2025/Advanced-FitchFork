import type { ReactNode } from 'react';
import { LinkOutlined } from '@ant-design/icons';
import clsx from 'clsx';

export const GATLAM_PAPER_URL =
  'https://repository.up.ac.za/items/734932b2-1784-4e2c-885e-1b633472a9bc';

type GatlamLinkProps = {
  /** Override the displayed label; defaults to “GATLAM”. */
  children?: ReactNode;
  /** Use `inherit` when the surrounding component controls color. */
  tone?: 'default' | 'inherit';
  /** Hide the external-link glyph for very tight layouts (e.g. inside tags). */
  icon?: boolean;
  className?: string;
  /** Disable the default hover underline when embedding in other styled text. */
  underline?: boolean;
};

const baseClasses =
  'inline-flex items-center gap-1 align-middle font-medium transition-colors decoration-dotted decoration-from-font';

export default function GatlamLink({
  children,
  tone = 'default',
  icon = true,
  className,
  underline = true,
}: GatlamLinkProps) {
  const label = children ?? 'GATLAM';

  return (
    <a
      href={GATLAM_PAPER_URL}
      target="_blank"
      rel="noreferrer noopener"
      className={clsx(
        baseClasses,
        underline && 'hover:underline',
        tone === 'default'
          ? 'text-purple-600 hover:text-purple-700 dark:text-purple-300 dark:hover:text-purple-200'
          : 'text-inherit hover:text-inherit',
        className,
      )}
      aria-label="Read more about GATLAM (opens in a new tab)"
    >
      <span>{label}</span>
      {icon ? (
        <LinkOutlined className={tone === 'default' ? 'text-purple-400' : 'text-inherit/60'} />
      ) : null}
    </a>
  );
}
