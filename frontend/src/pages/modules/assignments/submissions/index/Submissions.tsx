import { useAuth } from '@/context/AuthContext';
import { useModule } from '@/context/ModuleContext';
import SubmissionsList from './SubmissionsList';

const Submissions = () => {
  const module = useModule();
  const { isStudent } = useAuth();

  if (isStudent(module.id)) return <SubmissionsList />;
  return <SubmissionsList />;
};

export default Submissions;
