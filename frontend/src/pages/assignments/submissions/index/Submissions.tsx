import { useAuth } from '@/context/AuthContext';
import { useModule } from '@/context/ModuleContext';
import Unauthorized from '@/pages/shared/status/Unauthorized';
import SubmissionsList from './SubmissionsList';

const Submissions = () => {
  const module = useModule();
  const { isStudent } = useAuth();

  if (isStudent(module.id)) return <SubmissionsList />;
  return <Unauthorized />;
};

export default Submissions;
