import { useAuth } from '@/context/AuthContext';
import ModulesList from './ModulesList';
import ModulesUser from './ModulesGrid';

const Modules = () => {
  const { isAdmin, modules } = useAuth();

  if (isAdmin) {
    return <ModulesList />;
  }

  return <ModulesUser title="My Modules" modules={modules} />;
};

export default Modules;
