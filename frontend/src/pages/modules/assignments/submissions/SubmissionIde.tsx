import { useLocation } from 'react-router-dom';
import IdePlayground from '@/pages/IdePlayground';
import type { VFile } from '@/pages/IdePlayground';

export default function SubmissionIde() {
  const { state } = useLocation();
  const files: VFile[] = state?.files ?? [];
  return <IdePlayground readOnly files={files} />;
}
