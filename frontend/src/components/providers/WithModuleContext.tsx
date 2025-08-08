import { useEffect, useState } from 'react';
import { Outlet, useParams } from 'react-router-dom';
import { Spin } from 'antd';
import { ModuleProvider } from '@/context/ModuleContext';
import { getModuleDetails } from '@/services/modules';
import type { Module } from '@/types/modules';
import type { User } from '@/types/users';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

export interface ModuleDetails extends Module {
  lecturers: User[];
  tutors: User[];
  students: User[];
}

export default function WithModuleContext() {
  const { id } = useParams();
  const moduleId = Number(id);
  const { setBreadcrumbLabel } = useBreadcrumbContext();

  const [module, setModule] = useState<ModuleDetails | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const load = async () => {
      const res = await getModuleDetails(moduleId);
      if (res.success && res.data) {
        setModule(res.data);

        setBreadcrumbLabel(`modules/${moduleId}`, res.data.code);
      }
      setLoading(false);
    };

    if (!isNaN(moduleId)) load();
  }, [moduleId]);

  if (loading || !module) {
    return (
      <div className="p-8">
        <Spin tip="Loading module..." />
      </div>
    );
  }

  return (
    <ModuleProvider value={{ module }}>
      <Outlet />
    </ModuleProvider>
  );
}
