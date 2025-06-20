import PageHeader from '@/components/PageHeader';
import { Card } from 'antd';
import { Calendar } from 'antd';

export default function CalendarPage() {
  return (
    <div className="p-4 sm:p-6">
      <PageHeader
        title="Assignment Calendar"
        description="View upcomming assignments that are due"
      />
      <Card className="mt-4">
        <Calendar fullscreen />
      </Card>
    </div>
  );
}
