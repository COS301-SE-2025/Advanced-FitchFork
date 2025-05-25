import { useNotifier } from '@/components/Notifier';
import AppLayout from '@/layouts/AppLayout';
import { Space, Button } from 'antd';

export default function Home() {
  const { notifyInfo, notifyError, notifySuccess } = useNotifier();
  return (
    <AppLayout
      title="Welcome Home, Admin"
      description="Manage your marking pipeline, submissions, and reports."
    >
      <p>Lorem</p>
      <Space>
        <Button onClick={() => notifyInfo('Info', 'This is an informational message.')}>
          Show Info
        </Button>
        <Button onClick={() => notifyError('Error', 'Something went wrong.')}>Show Error</Button>
        <Button onClick={() => notifySuccess('Success', 'Something went right.')}>
          Show Succes
        </Button>
      </Space>
    </AppLayout>
  );
}
