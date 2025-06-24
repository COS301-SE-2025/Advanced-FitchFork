import { useAuth } from '@/context/AuthContext';
import { useModule } from '@/context/ModuleContext';
import Unauthorized from '@/pages/shared/status/Unauthorized';
import AssignmentsList from './AssignmentsList';
import AssignmentsTable from './AssignmentsTable';

const Assignments = () => {
  const module = useModule();
  const { isAdmin, isStudent, isLecturer, isTutor } = useAuth();

  if (isAdmin || isLecturer(module.id)) {
    return <AssignmentsTable />;
  }

  if (isStudent(module.id) || isTutor(module.id)) {
    return <AssignmentsList />;
  }

  return <Unauthorized />;
};

export default Assignments;
