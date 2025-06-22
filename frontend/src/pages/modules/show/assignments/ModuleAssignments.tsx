import { useAuth } from '@/context/AuthContext';
import { useModule } from '@/context/ModuleContext';
import ModuleAssignmentsList from './ModuleAssignmentsList';
import ModuleAssignmentsTable from './ModuleAssignmentsTable';
import Unauthorized from '@/pages/shared/status/Unauthorized';

const ModuleAssignments = () => {
  const module = useModule();
  const { isAdmin, isStudent, isLecturer, isTutor } = useAuth();

  if (isAdmin || isLecturer(module.id)) {
    return <ModuleAssignmentsTable />;
  }

  if (isStudent(module.id) || isTutor(module.id)) {
    return <ModuleAssignmentsList />;
  }

  return <Unauthorized />;
};

export default ModuleAssignments;
