import { useQuery } from 'react-query';
import { Link } from 'react-router-dom';
import {
  FolderIcon,
  ChartBarIcon,
  ClockIcon,
  ExclamationTriangleIcon,
} from '@heroicons/react/24/outline';

import { api } from '@services/api';
import LoadingSpinner from '@components/LoadingSpinner';

function Dashboard() {
  const {
    data: projects,
    isLoading: projectsLoading,
    error: projectsError,
  } = useQuery('projects', api.projects.getAll);

  const {
    data: dashboardStats,
    isLoading: statsLoading,
    error: statsError,
  } = useQuery('dashboard-stats', api.dashboard.getStats);

  if (projectsLoading || statsLoading) {
    return (
      <div className="flex items-center justify-center min-h-96">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (projectsError || statsError) {
    return (
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="bg-error-50 dark:bg-error-900/20 border border-error-200 dark:border-error-800 rounded-md p-4">
          <div className="flex">
            <ExclamationTriangleIcon className="h-5 w-5 text-error-400" aria-hidden="true" />
            <div className="ml-3">
              <h3 className="text-sm font-medium text-error-800 dark:text-error-200">
                Error loading dashboard
              </h3>
              <p className="mt-2 text-sm text-error-700 dark:text-error-300">
                {(projectsError instanceof Error ? projectsError.message : statsError instanceof Error ? statsError.message : 'An unexpected error occurred')}
              </p>
            </div>
          </div>
        </div>
      </div>
    );
  }

  const recentProjects = projects?.slice(0, 5) || [];
  const totalProjects = dashboardStats?.total_projects || projects?.length || 0;
  const analysesThisWeek = dashboardStats?.analyses_this_week || 0;
  const avgProcessingTime = dashboardStats?.avg_processing_time_minutes;
  const criticalErrors = dashboardStats?.critical_errors || 0;

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      {/* Page header */}
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white">Dashboard</h1>
        <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
          Welcome to LogLens. Monitor your log analysis projects and recent activity.
        </p>
      </div>

      {/* Stats overview */}
      <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4 mb-8">
        <div className="card">
          <div className="card-body">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <FolderIcon className="h-6 w-6 text-primary-600" aria-hidden="true" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">
                    Total Projects
                  </dt>
                  <dd className="text-lg font-medium text-gray-900 dark:text-white">
                    {totalProjects}
                  </dd>
                </dl>
              </div>
            </div>
          </div>
        </div>

        <div className="card">
          <div className="card-body">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <ChartBarIcon className="h-6 w-6 text-success-600" aria-hidden="true" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">
                    Analyses This Week
                  </dt>
                  <dd className="text-lg font-medium text-gray-900 dark:text-white">
                    {analysesThisWeek}
                  </dd>
                </dl>
              </div>
            </div>
          </div>
        </div>

        <div className="card">
          <div className="card-body">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <ClockIcon className="h-6 w-6 text-warning-600" aria-hidden="true" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">
                    Avg Processing Time
                  </dt>
                  <dd className="text-lg font-medium text-gray-900 dark:text-white">
                    {avgProcessingTime ? `${avgProcessingTime.toFixed(1)}m` : '-'}
                  </dd>
                </dl>
              </div>
            </div>
          </div>
        </div>

        <div className="card">
          <div className="card-body">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <ExclamationTriangleIcon className="h-6 w-6 text-error-600" aria-hidden="true" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">
                    Critical Errors
                  </dt>
                  <dd className="text-lg font-medium text-gray-900 dark:text-white">
                    {criticalErrors}
                  </dd>
                </dl>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Recent projects */}
      <div className="card">
        <div className="card-header">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-medium text-gray-900 dark:text-white">
              Recent Projects
            </h3>
            <Link
              to="/projects"
              className="text-sm font-medium text-primary-600 hover:text-primary-500 dark:text-primary-400 dark:hover:text-primary-300"
            >
              View all
            </Link>
          </div>
        </div>
        <div className="card-body">
          {recentProjects.length === 0 ? (
            <div className="text-center py-12">
              <FolderIcon className="mx-auto h-12 w-12 text-gray-400" />
              <h3 className="mt-2 text-sm font-medium text-gray-900 dark:text-white">
                No projects yet
              </h3>
              <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
                Get started by creating your first log analysis project.
              </p>
              <div className="mt-6">
                <Link
                  to="/projects"
                  className="btn-primary"
                >
                  Create Project
                </Link>
              </div>
            </div>
          ) : (
            <div className="space-y-4">
              {recentProjects.map((project) => (
                <div
                  key={project.id}
                  className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-lg"
                >
                  <div className="flex items-center space-x-3">
                    <div className="flex-shrink-0">
                      <FolderIcon className="h-5 w-5 text-gray-400" aria-hidden="true" />
                    </div>
                    <div>
                      <h4 className="text-sm font-medium text-gray-900 dark:text-white">
                        {project.name}
                      </h4>
                      {project.description && (
                        <p className="text-sm text-gray-500 dark:text-gray-400">
                          {project.description}
                        </p>
                      )}
                    </div>
                  </div>
                  <div className="flex items-center space-x-3">
                    <span className="text-sm text-gray-500 dark:text-gray-400">
                      {new Date(project.created_at).toLocaleDateString()}
                    </span>
                    <Link
                      to={`/projects/${project.id}`}
                      className="text-sm font-medium text-primary-600 hover:text-primary-500 dark:text-primary-400 dark:hover:text-primary-300"
                    >
                      View
                    </Link>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export default Dashboard;