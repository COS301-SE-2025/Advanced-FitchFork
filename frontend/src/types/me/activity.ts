export type ActivityModule = {
  id: number;
  code: string;
  year: number;
};

export type ActivityItem = {
  id: string;
  activity_type: string;
  title: string;
  summary: string;
  timestamp: string;
  module?: ActivityModule;
  link?: string;
};

export type ActivityFeed = {
  activities: ActivityItem[];
  page: number;
  per_page: number;
  total: number;
};
