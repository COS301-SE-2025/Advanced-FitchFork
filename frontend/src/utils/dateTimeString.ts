import dayjs, { Dayjs } from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import utc from 'dayjs/plugin/utc';

dayjs.extend(relativeTime);
dayjs.extend(utc);

export type ValueLike = string | number | Date | Dayjs;

export type DateTimeVariant =
  | 'relative'
  | 'datetime'
  | 'date'
  | 'time'
  | 'compact'
  | 'long'
  | 'micro';

export function dateTimeString(
  value: ValueLike,
  variant: DateTimeVariant = 'datetime',
  opts?: { seconds?: boolean; format?: string },
): string {
  const dt = dayjs(value);
  if (!dt.isValid()) return 'Invalid date';

  const now = dayjs();
  const isSameDay = now.isSame(dt, 'day');
  const { seconds = false, format } = opts || {};

  // default formats
  const fmtDatetime = format ?? (seconds ? 'YYYY-MM-DD HH:mm:ss' : 'YYYY-MM-DD HH:mm');
  const fmtDate = format ?? 'YYYY-MM-DD';
  const fmtTime = format ?? (seconds ? 'HH:mm:ss' : 'HH:mm');
  const fmtCompact = format ?? (seconds ? 'MMM D, HH:mm:ss' : 'MMM D, HH:mm');
  const fmtLong = format ?? (seconds ? 'dddd, MMM D, YYYY HH:mm:ss' : 'dddd, MMM D, YYYY HH:mm');

  switch (variant) {
    case 'relative':
      return dt.fromNow();
    case 'date':
      return dt.format(fmtDate);
    case 'time':
      return dt.format(fmtTime);
    case 'compact':
      return dt.format(fmtCompact);
    case 'long':
      return dt.format(fmtLong);
    case 'micro':
      return isSameDay ? dt.format(fmtTime) : dt.format('MMM D');
    case 'datetime':
    default:
      return dt.format(fmtDatetime);
  }
}
