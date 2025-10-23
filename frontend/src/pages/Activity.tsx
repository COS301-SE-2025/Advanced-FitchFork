import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Alert, Empty, Spin } from 'antd';

import ActivityHeader from '@/components/activity/ActivityHeader';
import ActivitySection from '@/components/activity/ActivitySection';
import { createTimelineItem } from '@/components/activity/createTimelineItem';
import { groupActivitiesByTime } from '@/components/activity/timelineUtils';
import { getMyActivity } from '@/services/me/activity/get';
import type { ActivityItem } from '@/types/me/activity';

const PER_PAGE = 20;

const ActivityPage = () => {
  const [activities, setActivities] = useState<ActivityItem[]>([]);
  const [loadingFirstPage, setLoadingFirstPage] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [refreshIndex, setRefreshIndex] = useState(0);

  const [page, setPage] = useState(1);
  const [total, setTotal] = useState(0);

  const sentinelRef = useRef<HTMLDivElement | null>(null);
  const hasMore = activities.length < total;

  const fetchPage = useCallback(async (nextPage: number) => {
    const isFirst = nextPage === 1;
    isFirst ? setLoadingFirstPage(true) : setLoadingMore(true);
    setError(null);

    try {
      const response = await getMyActivity({ page: nextPage, per_page: PER_PAGE });

      if (!response?.success || !response?.data) {
        if (isFirst) setActivities([]);
        setTotal(0);
        setError(response?.message || 'Failed to fetch activity.');
        return;
      }

      const newItems = response.data.activities ?? [];
      setTotal(response.data.total ?? 0);

      if (isFirst) {
        setActivities(newItems);
      } else {
        setActivities((prev) => {
          const seen = new Set(prev.map((a) => a.id));
          const appended = newItems.filter((a) => !seen.has(a.id));
          return [...prev, ...appended];
        });
      }

      setPage(response.data.page ?? nextPage);
    } catch (err) {
      console.error('Failed to load activity feed', err);
      if (nextPage === 1) setActivities([]);
      setError('Unable to load activity right now.');
    } finally {
      isFirst ? setLoadingFirstPage(false) : setLoadingMore(false);
    }
  }, []);

  useEffect(() => {
    setPage(1);
    setTotal(0);
    setActivities([]);
    fetchPage(1);
  }, [fetchPage, refreshIndex]);

  useEffect(() => {
    const element = sentinelRef.current;
    if (!element) return;

    const observer = new IntersectionObserver(
      (entries) => {
        const [entry] = entries;
        if (entry.isIntersecting && !loadingMore && !loadingFirstPage && hasMore) {
          fetchPage(page + 1);
        }
      },
      { root: null, rootMargin: '600px 0px', threshold: 0 },
    );

    observer.observe(element);
    return () => observer.disconnect();
  }, [fetchPage, hasMore, loadingFirstPage, loadingMore, page]);

  const groupedItems = useMemo(() => {
    const { upcoming, today, recent } = groupActivitiesByTime(activities);

    return {
      upcomingItems: upcoming.map(createTimelineItem),
      todayItems: today.map(createTimelineItem),
      recentItems: recent.map(createTimelineItem),
    };
  }, [activities]);

  return (
    <div className="h-full overflow-auto bg-transparent dark:bg-transparent">
      <div className="w-full p-4 space-y-6">
        <ActivityHeader
          onRefresh={() => setRefreshIndex((idx) => idx + 1)}
          disabled={loadingFirstPage || loadingMore}
        />

        {error && (
          <Alert type="error" showIcon message="Unable to load activity" description={error} />
        )}

        {loadingFirstPage ? (
          <div className="flex justify-center py-16">
            <Spin size="large" />
          </div>
        ) : activities.length === 0 ? (
          <Empty description="No recent activity yet" className="py-16" />
        ) : (
          <div className="space-y-8">
            <ActivitySection title="Today" items={groupedItems.todayItems} />
            <ActivitySection title="Upcoming" items={groupedItems.upcomingItems} />
            <ActivitySection title="Recent" items={groupedItems.recentItems} />

            <div ref={sentinelRef} />

            {loadingMore && (
              <div className="flex justify-center py-6">
                <Spin />
              </div>
            )}

            {!hasMore && activities.length > 0 && (
              <div className="py-6 text-center text-xs text-gray-500 dark:text-gray-400">
                You’re all caught up.
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};

export default ActivityPage;
