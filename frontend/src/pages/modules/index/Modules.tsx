import { useAuth } from '@/context/AuthContext';
import ModulesAdmin from './ModulesTable';
import ModulesUser from './ModulesGrid';

const Modules = () => {
  const { isAdmin, modules } = useAuth();

  if (isAdmin) {
    return <ModulesAdmin />;
  }

  return <ModulesUser title="My Modules" modules={modules} />;
};

export default Modules;
