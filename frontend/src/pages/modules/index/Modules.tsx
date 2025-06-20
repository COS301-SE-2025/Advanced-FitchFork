import { useAuth } from '@/context/AuthContext';
import { useLocation } from 'react-router-dom';
import ModulesAdmin from './ModulesTable';
import ModulesUser from './ModulesGrid';

const Modules = () => {
  const { isAdmin, modulesByRole, modules } = useAuth();
  const location = useLocation();

  if (isAdmin) {
    return <ModulesAdmin />;
  }

  const pathname = location.pathname;

  if (pathname.endsWith('/enrolled')) {
    return <ModulesUser title="My Enrolled Modules" modules={modulesByRole.Student} />;
  } else if (pathname.endsWith('/tutoring')) {
    return <ModulesUser title="My Tutoring Modules" modules={modulesByRole.Tutor} />;
  } else if (pathname.endsWith('/lecturing')) {
    return <ModulesUser title="My Lecturing Modules" modules={modulesByRole.Lecturer} />;
  } else {
    return <ModulesUser title="My Modules" modules={modules} />;
  }
};

export default Modules;
