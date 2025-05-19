import DashboardLayout from '@layouts/DashboardLayout';

export default function Dashboard() {
  return (
    <DashboardLayout>
      <h1 className="text-2xl font-semibold mb-4">Welcome, Admin</h1>
      <p className="text-gray-600">Manage your marking pipeline, submissions, and reports.</p>
    </DashboardLayout>
  );
}
