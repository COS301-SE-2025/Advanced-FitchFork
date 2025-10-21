import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import utc from 'dayjs/plugin/utc';
import timezone from 'dayjs/plugin/timezone';

import type { ActivityItem } from '@/types/me/activity';

// Configure dayjs once for the activity timeline utilities
dayjs.extend(relativeTime);
dayjs.extend(utc);
dayjs.extend(timezone);

export const toLocalInstant = (iso: string) => {
  const hasTzSuffix = /([Zz]|[+\-]\d{2}:\d{2})$/.test(iso);
  const localTz = dayjs.tz.guess();
  return hasTzSuffix ? dayjs.utc(iso).tz(localTz) : dayjs(iso);
};

export const formatTimestampAbsoluteLocal = (iso: string): string => {
  const instant = toLocalInstant(iso);
  return instant.isValid() ? instant.format('MMM D, YYYY HH:mm') : iso;
};

const DATE_REGEX =
  /\b(?:\d{4}-\d{2}-\d{2}(?:[ T]\d{2}:\d{2}(?::\d{2})?(?:Z|[+\-]\d{2}:\d{2})?)?|\b(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[a-z]*\s+\d{1,2},\s*\d{4}(?:\s+\d{2}:\d{2}(?::\d{2})?)?)\b/gi;

const stripTrailingKeywords = (value: string) =>
  value
    .replace(
      /\b(Due|Available|Opens|Closes|Closing|Starts|Starting|Ends|Ending)\s*[:\-–—]?\s*$/i,
      '',
    )
    .trim()
    .replace(/\s*[-–—|]\s*$/g, '')
    .trim();

export const stripDates = (text?: string) => {
  if (!text) return text ?? '';
  const withoutDates = text.replace(DATE_REGEX, ' ').replace(/\s{2,}/g, ' ').trim();
  return stripTrailingKeywords(withoutDates);
};

export const sanitizeForSingleDate = (
  activityType: string,
  title: string,
  summary?: string,
): { title: string; summary?: string } => {
  if (activityType === 'assignment_available' || activityType === 'assignment_due') {
    return {
      title: stripDates(title),
      summary: stripDates(summary),
    };
  }

  return { title, summary };
};

export const groupActivitiesByTime = (activities: ActivityItem[]) => {
  const startOfToday = dayjs().startOf('day');
  const endOfToday = dayjs().endOf('day');

  const buckets = {
    upcoming: [] as ActivityItem[],
    today: [] as ActivityItem[],
    recent: [] as ActivityItem[],
  };

  for (const activity of activities) {
    const instant = toLocalInstant(activity.timestamp);
    if (!instant.isValid()) {
      buckets.recent.push(activity);
      continue;
    }

    if (instant.isAfter(endOfToday)) {
      buckets.upcoming.push(activity);
    } else if (instant.isBefore(startOfToday)) {
      buckets.recent.push(activity);
    } else {
      buckets.today.push(activity);
    }
  }

  const desc = (lhs: ActivityItem, rhs: ActivityItem) =>
    toLocalInstant(rhs.timestamp).valueOf() - toLocalInstant(lhs.timestamp).valueOf();

  buckets.upcoming.sort(desc);
  buckets.today.sort(desc);
  buckets.recent.sort(desc);

  return buckets;
};
