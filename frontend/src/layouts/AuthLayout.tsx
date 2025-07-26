import { Outlet } from 'react-router-dom';

export default function AuthLayout() {
  return (
    <div className="bg-gray-100 dark:bg-gray-900 flex w-full h-screen items-center justify-center px-4 sm:px-6 md:px-10 py-12">
      <Outlet />
    </div>
  );
}
