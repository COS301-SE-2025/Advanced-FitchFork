import { Card, Typography } from 'antd';
import type { PlagiarismCaseItem } from '@/types/modules/assignments/plagiarism';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import PlagiarismStatusTag from './PlagiarismStatusTag';
import { PercentageTag } from '../common';

dayjs.extend(relativeTime);
const { Text } = Typography;

interface Props {
  caseItem: PlagiarismCaseItem;
  actions?: React.ReactNode[];
  onClick?: () => void;
}

// Normalize similarity: supports 0..1 or 0..100
const toPercent = (v: number) => {
  const p = v <= 1 ? v * 100 : v;
  return Math.max(0, Math.min(100, Math.round(p)));
};

const PlagiarismCaseCard = ({ caseItem, actions, onClick }: Props) => {
  const pct = toPercent(caseItem.similarity);
  const updatedAgo = dayjs(caseItem.updated_at).fromNow();

  return (
    <Card
      hoverable
      size="small"
      onClick={onClick}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          onClick?.();
        }
      }}
      aria-label={`Case #${caseItem.id}, ${caseItem.status}, similarity ${pct}%`}
      className="w-full cursor-pointer !bg-white dark:!bg-gray-900"
      bodyStyle={{ padding: 12 }}
      actions={actions}
      data-testid="plagiarism-case-card"
    >
      <div className="min-w-0">
        {/* Title row with perfectly aligned status tag */}
        <div className="flex items-center justify-between gap-2 min-w-0">
          <Text className="font-medium truncate text-black dark:text-white leading-[22px]">
            Case #{caseItem.id}
          </Text>
          {/* Baseline-nudged wrapper to align with AntD Tag's line height */}
          <span className="inline-flex items-center relative -top-[1px]">
            <PlagiarismStatusTag status={caseItem.status} />
          </span>
        </div>

        {/* Participants (single truncated line) */}
        <Text type="secondary" className="block truncate">
          {caseItem.submission_1.user.username} vs {caseItem.submission_2.user.username}
        </Text>

        {/* Meta: colored similarity as plain text + relative time */}
        <div className="text-xs mt-0.5 flex items-center gap-2">
          <PercentageTag
            asText
            value={pct}
            decimals={0}
            suffix="%"
            // Higher similarity should look worse -> red toward 100
            scheme="red-green"
            className="leading-[18px]"
          />
          <Text type="secondary" className="leading-[18px]">
            Updated {updatedAgo}
          </Text>
        </div>
      </div>
    </Card>
  );
};

export default PlagiarismCaseCard;
