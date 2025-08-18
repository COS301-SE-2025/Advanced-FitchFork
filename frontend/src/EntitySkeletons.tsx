import { Card, Skeleton } from 'antd';

/** Grid-mode skeleton: identical styling to your inline version */
export function GridSkeleton({ count = 8 }: { count?: number }) {
  return (
    <div className="grid gap-4 grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4">
      {Array.from({ length: count }).map((_, i) => (
        <Card key={i}>
          <Skeleton active avatar paragraph={{ rows: 3 }} />
        </Card>
      ))}
    </div>
  );
}

/** List/Table-mode skeleton: identical styling to your inline version */
export function RowsSkeleton({ rows = 6 }: { rows?: number }) {
  return (
    <Card className="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-lg overflow-hidden">
      {Array.from({ length: rows }).map((_, i) => (
        <div key={i} className="px-4 py-3 border-b border-gray-100 dark:border-gray-800">
          <Skeleton active title={false} paragraph={{ rows: 1, width: '100%' }} />
        </div>
      ))}
    </Card>
  );
}
