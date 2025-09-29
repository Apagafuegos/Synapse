import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from 'react-query';
import { Link } from 'react-router-dom';
import { PlusIcon, FolderIcon } from '@heroicons/react/24/outline';

import { api, ApiError } from '@services/api';
import LoadingSpinner from '@components/LoadingSpinner';

function Projects() {
  const [searchTerm, setSearchTerm] = useState('');
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [newProject, setNewProject] = useState({ name: '', description: '' });
  const [isCreating, setIsCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const queryClient = useQueryClient();

  const {
    data: projects,
    isLoading,
    error: queryError,
    refetch,
  } = useQuery('projects', api.projects.getAll);

  const createProjectMutation = useMutation(
    (projectData: { name: string; description?: string }) => api.projects.create(projectData),
    {
      onSuccess: () => {
        queryClient.invalidateQueries('projects');
        setIsCreateModalOpen(false);
        setNewProject({ name: '', description: '' });
        setError(null);
      },
      onError: (err: ApiError) => {
        setError(err.message || 'Failed to create project');
      },
    }
  );

  const filteredProjects = projects?.filter((project) =>
    project.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    project.description?.toLowerCase().includes(searchTerm.toLowerCase())
  ) || [];

  const handleCreateProject = async () => {
    if (!newProject.name.trim()) {
      setError('Project name is required');
      return;
    }

    setIsCreating(true);
    try {
      await createProjectMutation.mutateAsync({
        name: newProject.name.trim(),
        description: newProject.description.trim() || undefined,
      });
    } catch (err) {
      // Error is handled by mutation onError
    } finally {
      setIsCreating(false);
    }
  };

  const handleModalClose = () => {
    setIsCreateModalOpen(false);
    setNewProject({ name: '', description: '' });
    setError(null);
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-96">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      {/* Page header */}
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">Projects</h1>
          <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
            Manage your log analysis projects and track their progress.
          </p>
        </div>
        <button
          type="button"
          className="btn-primary"
          onClick={() => setIsCreateModalOpen(true)}
        >
          <PlusIcon className="h-4 w-4 mr-2" aria-hidden="true" />
          New Project
        </button>
      </div>

      {/* Search and filters */}
      <div className="mb-6">
        <div className="max-w-md">
          <label htmlFor="search" className="sr-only">
            Search projects
          </label>
          <input
            type="text"
            name="search"
            id="search"
            className="input"
            placeholder="Search projects..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
        </div>
      </div>

      {/* Projects grid */}
      {queryError ? (
        <div className="bg-error-50 dark:bg-error-900/20 border border-error-200 dark:border-error-800 rounded-md p-4">
          <p className="text-sm text-error-700 dark:text-error-300">
            {queryError instanceof Error ? queryError.message : 'Failed to load projects'}
          </p>
          <button
            type="button"
            onClick={() => refetch()}
            className="mt-2 text-sm font-medium text-error-800 dark:text-error-200 hover:text-error-900 dark:hover:text-error-100"
          >
            Try again
          </button>
        </div>
      ) : filteredProjects.length === 0 ? (
        <div className="text-center py-12">
          <FolderIcon className="mx-auto h-12 w-12 text-gray-400" />
          <h3 className="mt-2 text-sm font-medium text-gray-900 dark:text-white">
            {searchTerm ? 'No projects found' : 'No projects yet'}
          </h3>
          <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
            {searchTerm
              ? 'Try adjusting your search terms.'
              : 'Get started by creating your first log analysis project.'
            }
          </p>
          {!searchTerm && (
            <div className="mt-6">
              <button
                type="button"
                className="btn-primary"
                onClick={() => setIsCreateModalOpen(true)}
              >
                <PlusIcon className="h-4 w-4 mr-2" aria-hidden="true" />
                Create Project
              </button>
            </div>
          )}
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {filteredProjects.map((project) => (
            <Link
              key={project.id}
              to={`/projects/${project.id}`}
              className="card hover:shadow-md transition-shadow group"
            >
              <div className="card-body">
                <div className="flex items-center space-x-3">
                  <div className="flex-shrink-0">
                    <FolderIcon className="h-6 w-6 text-primary-600 group-hover:text-primary-500" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <h3 className="text-lg font-medium text-gray-900 dark:text-white group-hover:text-primary-600 dark:group-hover:text-primary-400 truncate">
                      {project.name}
                    </h3>
                    {project.description && (
                      <p className="text-sm text-gray-500 dark:text-gray-400 line-clamp-2">
                        {project.description}
                      </p>
                    )}
                  </div>
                </div>

                <div className="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
                  <div className="flex items-center justify-between text-sm text-gray-500 dark:text-gray-400">
                    <span>Created {new Date(project.created_at).toLocaleDateString()}</span>
                    {project.analysis_count !== undefined && (
                      <span>{project.analysis_count} analyses</span>
                    )}
                  </div>
                </div>
              </div>
            </Link>
          ))}
        </div>
      )}

      {/* Create Project Modal */}
      {isCreateModalOpen && (
        <div className="fixed inset-0 z-50 overflow-y-auto">
          <div className="flex items-center justify-center min-h-screen px-4 pt-4 pb-20 text-center sm:block sm:p-0">
            {/* Background overlay */}
            <div
              className="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity"
              onClick={handleModalClose}
            ></div>

            {/* Modal panel */}
            <div className="inline-block w-full max-w-md p-6 my-8 overflow-hidden text-left align-bottom transition-all transform bg-white dark:bg-gray-800 rounded-lg shadow-xl sm:my-8 sm:align-middle sm:w-full">
              <div className="sm:flex sm:items-start">
                <div className="mt-3 text-center sm:mt-0 sm:ml-4 sm:text-left w-full">
                  <h3 className="text-lg leading-6 font-medium text-gray-900 dark:text-white mb-4">
                    Create New Project
                  </h3>
                  
                  <div className="mt-2 space-y-4">
                    <div>
                      <label htmlFor="project-name" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        Project Name *
                      </label>
                      <input
                        type="text"
                        id="project-name"
                        className="input"
                        placeholder="Enter project name"
                        value={newProject.name}
                        onChange={(e) => setNewProject({ ...newProject, name: e.target.value })}
                        disabled={isCreating}
                      />
                    </div>
                    
                    <div>
                      <label htmlFor="project-description" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        Description (optional)
                      </label>
                      <textarea
                        id="project-description"
                        rows={3}
                        className="input"
                        placeholder="Enter project description"
                        value={newProject.description}
                        onChange={(e) => setNewProject({ ...newProject, description: e.target.value })}
                        disabled={isCreating}
                      />
                    </div>

                    {error && (
                      <div className="bg-error-50 dark:bg-error-900/20 border border-error-200 dark:border-error-800 rounded-md p-3">
                        <p className="text-sm text-error-700 dark:text-error-300">
                          {error}
                        </p>
                      </div>
                    )}
                  </div>
                </div>
              </div>

              <div className="mt-5 sm:mt-4 sm:flex sm:flex-row-reverse">
                <button
                  type="button"
                  className="btn-primary w-full sm:w-auto sm:ml-3"
                  onClick={handleCreateProject}
                  disabled={isCreating}
                >
                  {isCreating ? (
                    <>
                      <svg className="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                        <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                        <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                      </svg>
                      Creating...
                    </>
                  ) : (
                    'Create Project'
                  )}
                </button>
                <button
                  type="button"
                  className="btn-secondary w-full sm:w-auto sm:ml-3"
                  onClick={handleModalClose}
                  disabled={isCreating}
                >
                  Cancel
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default Projects;