import { useEffect } from 'react';
import { useViewSlot } from '@/context/ViewSlotContext';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';

/**
 * Sets MobilePageHeader's back target to the assignment's Config menu root.
 * Clears it on unmount.
 */
export default function useConfigBackTo() {
  const { setBackTo } = useViewSlot();
  const module = useModule();
  const { assignment } = useAssignment();

  useEffect(() => {
    if (!module?.id || !assignment?.id) return;
    const base = `/modules/${module.id}/assignments/${assignment.id}/config`;
    setBackTo(base);
    return () => setBackTo(null);
  }, [module?.id, assignment?.id, setBackTo]);
}
