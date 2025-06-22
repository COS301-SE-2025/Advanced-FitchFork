import { useLocation } from 'react-router-dom';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

interface BreadcrumbItem {
  path: string;
  label: string;
  isLast: boolean;
}

export function useBreadcrumbs(): BreadcrumbItem[] {
  const location = useLocation();
  const { customLabels } = useBreadcrumbContext();

  const segments = location.pathname.split('/').filter(Boolean);

  return segments.map((_, index, arr) => {
    const path = '/' + arr.slice(0, index + 1).join('/');
    const isLast = index === arr.length - 1;
    const fullKey = arr.slice(0, index + 1).join('/');

    // Prefer full key match, otherwise fallback to just the segment
    const label =
      customLabels[fullKey] ??
      arr[index];

    return {
      path,
      label: label.charAt(0).toUpperCase() + label.slice(1),
      isLast,
    };
  });
}
