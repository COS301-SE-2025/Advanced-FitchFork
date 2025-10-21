import { Link } from 'react-router-dom';
import { Card, Space, Tag, Typography } from 'antd';

import PercentageTag from '@/components/common/PercentageTag';
import type { ActivityItem } from '@/types/me/activity';

import type { ActivityAppearance } from './appearance';
import { formatTimestampAbsoluteLocal, sanitizeForSingleDate } from './timelineUtils';

interface Props {
  activity: ActivityItem;
  appearance: ActivityAppearance;
}

const extractSubmissionDetails = (summary?: string) => {
  if (!summary) return null;

  const attemptMatch = summary.match(/Attempt\s+(\d+)/i);
  const scoreMatch = summary.match(/Score\s+([\d.]+)\/([\d.]+)/i);

  const attempt = attemptMatch ? attemptMatch[1] : undefined;
  const earned = scoreMatch ? parseFloat(scoreMatch[1]) : undefined;
  const total = scoreMatch ? parseFloat(scoreMatch[2]) : undefined;
  const percent = earned !== undefined && total && total > 0 ? (earned / total) * 100 : undefined;

  let status: string | undefined;
  if (!scoreMatch) {
    const parts = summary
      .split(/[-–—]/)
      .map((part) => part.trim())
      .filter(Boolean);
    status = parts.length > 1 ? parts.slice(1).join(' - ') : undefined;
  }

  return { attempt, percent, status };
};

const ActivityTimelineItemCard = ({ activity, appearance }: Props) => {
  const moduleLink = activity.module ? `/modules/${activity.module.id}` : undefined;
  const targetHref = activity.link ?? moduleLink;

  const { title: cleanTitle, summary: cleanSummary } = sanitizeForSingleDate(
    activity.activity_type,
    activity.title,
    activity.summary,
  );

  const absoluteLocal = formatTimestampAbsoluteLocal(activity.timestamp);
  const submissionDetails =
    activity.activity_type === 'submission' ? extractSubmissionDetails(activity.summary) : null;

  return (
    <Card
      size="small"
      className="w-fit max-w-full rounded-xl border border-gray-200 bg-white shadow-none dark:border-gray-800 dark:bg-gray-900"
    >
      <Space direction="vertical" size={10} className="w-full">
        <Typography.Title
          level={4}
          className="!mb-0 text-base font-semibold min-w-0 text-gray-900 dark:text-gray-100"
        >
          {targetHref ? (
            <Link
              to={targetHref}
              className="
                no-underline hover:underline decoration-1 underline-offset-2
                !text-[inherit] hover:!text-[inherit] visited:!text-[inherit]
                active:!text-[inherit] focus:!text-[inherit]
                focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2
                focus-visible:ring-blue-500 dark:focus-visible:ring-blue-400
              "
            >
              {cleanTitle}
            </Link>
          ) : (
            cleanTitle
          )}
        </Typography.Title>

        <Typography.Text type="secondary" className="text-sm">
          {absoluteLocal}
        </Typography.Text>

        {submissionDetails ? (
          <div className="flex items-center gap-2 overflow-x-auto sm:gap-3 sm:overflow-visible">
            <Tag color={appearance.color} className="flex-shrink-0 font-medium">
              {appearance.label}
            </Tag>
            {activity.module && (
              <Link to={moduleLink!} className="no-underline">
                <Tag color="default" className="flex-shrink-0">
                  {activity.module.code} ({activity.module.year})
                </Tag>
              </Link>
            )}
            {submissionDetails.attempt && (
              <Tag color="geekblue" className="flex-shrink-0">
                Attempt {submissionDetails.attempt}
              </Tag>
            )}
            {submissionDetails.percent !== undefined ? (
              <PercentageTag
                value={submissionDetails.percent}
                decimals={Number.isInteger(submissionDetails.percent) ? 0 : 1}
                scheme="red-green"
                className="flex-shrink-0"
              />
            ) : submissionDetails.status ? (
              <Tag className="flex-shrink-0">{submissionDetails.status}</Tag>
            ) : null}
          </div>
        ) : (
          <Space size={[8, 8]} wrap>
            <Tag color={appearance.color} className="font-medium">
              {appearance.label}
            </Tag>
            {activity.module && (
              <Link to={moduleLink!} className="no-underline">
                <Tag color="default">
                  {activity.module.code} ({activity.module.year})
                </Tag>
              </Link>
            )}
          </Space>
        )}

        {!submissionDetails && cleanSummary ? (
          <Typography.Text className="text-sm text-gray-700 dark:text-gray-300">
            {cleanSummary}
          </Typography.Text>
        ) : null}
      </Space>
    </Card>
  );
};

export default ActivityTimelineItemCard;
