import AppLayout from '@/layouts/AppLayout';
import { TeamOutlined, BookOutlined, InfoCircleOutlined } from '@ant-design/icons';
import { Tabs } from 'antd';
import GeneralInfoSection from './GeneralInfoSection';
import PersonnelSection from './PersonnelSection';
import AssignmentsSection from './AssignmentsSection';

export default function ModuleView() {
  return (
    <AppLayout
      title="Module View"
      description="Admin view of the module with assignments and user management"
    >
      <Tabs
        defaultActiveKey="general"
        items={[
          {
            key: 'general',
            label: 'General Info',
            icon: <InfoCircleOutlined />,
            children: <GeneralInfoSection />,
          },
          {
            key: 'personnel',
            label: 'Personnel',
            icon: <TeamOutlined />,
            children: <PersonnelSection />,
          },
          {
            key: 'assignments',
            label: 'Assignments',
            icon: <BookOutlined />,
            children: <AssignmentsSection />,
          },
        ]}
      />
    </AppLayout>
  );
}
