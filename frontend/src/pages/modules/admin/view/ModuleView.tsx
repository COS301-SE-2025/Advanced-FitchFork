import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import AppLayout from '@/layouts/AppLayout';
import { Tabs, Spin } from 'antd';
import { TeamOutlined, BookOutlined, InfoCircleOutlined } from '@ant-design/icons';
import GeneralInfoSection from './GeneralInfoSection';
import PersonnelSection from './PersonnelSection';
import AssignmentsSection from './AssignmentsSection';
import { ModulesService } from '@/services/modules';
import type { ModuleDetailsResponse } from '@/types/modules';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

export default function ModuleView() {
  const { id } = useParams();
  const moduleId = parseInt(id!, 10);
  const { setBreadcrumbLabel } = useBreadcrumbContext();

  const [moduleDetails, setModuleDetails] = useState<ModuleDetailsResponse | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    ModulesService.getModuleDetails(moduleId).then((res) => {
      if (res.success) {
        setModuleDetails(res.data);
        setBreadcrumbLabel(
          `modules/${res.data.id}`,
          `${res.data.code} - ${res.data.description || 'Module'}`,
        );
      }
      setLoading(false);
    });
  }, [moduleId]);

  if (loading || !moduleDetails) {
    return (
      <AppLayout title="Module View">
        <Spin />
      </AppLayout>
    );
  }

  return (
    <AppLayout
      title={
        <>
          {moduleDetails.code} <span className="text-gray-400">{moduleDetails.year}</span>
        </>
      }
      description={moduleDetails.description}
    >
      <Tabs
        defaultActiveKey="general"
        items={[
          {
            key: 'general',
            label: 'General Info',
            icon: <InfoCircleOutlined />,
            children: <GeneralInfoSection module={moduleDetails} />,
          },
          {
            key: 'personnel',
            label: 'Personnel',
            icon: <TeamOutlined />,
            children: <PersonnelSection moduleId={moduleId} />,
          },
          {
            key: 'assignments',
            label: 'Assignments',
            icon: <BookOutlined />,
            children: <AssignmentsSection moduleId={moduleId} />,
          },
        ]}
      />
    </AppLayout>
  );
}
