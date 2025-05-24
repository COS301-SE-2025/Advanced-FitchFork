import { useAuth } from '@/context/AuthContext';
import AdminModuleTableView from '@/pages/modules/AdminModuleTableView';
import UserModuleGridView from '@/pages/modules/UserModuleGridView';

/**
 * This component conditionally renders the module list view:
 * - Admins see a full editable table.
 * - Other users see a card-based grid view of their modules.
 */
const ModuleList: React.FC = () => {
  const { user } = useAuth();

  if (!user) return null;

  return user.admin ? <AdminModuleTableView /> : <UserModuleGridView filter="student" />;
};

export default ModuleList;
