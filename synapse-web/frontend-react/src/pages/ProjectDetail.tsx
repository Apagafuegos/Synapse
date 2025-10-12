import { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from 'react-query';
import { useSettings } from '@hooks/useSettings';
import { 
  ArrowUpTrayIcon, 
  DocumentTextIcon, 
  TrashIcon, 
  ChartBarIcon,
  ClockIcon,
  CheckCircleIcon,
  XCircleIcon,
  ArrowRightIcon
} from '@heroicons/react/24/outline';

import { api, ApiError } from '@services/api';
import LoadingSpinner from '@components/LoadingSpinner';
import NotFound from '@pages/NotFound';
import type { LogFile, Analysis } from '@/types';

function ProjectDetail() {
  const { projectId } = useParams<{ projectId: string }>();
  console.log('ProjectDetail: URL params from useParams():', { projectId, allParams: useParams() });
  console.log('ProjectDetail: Current URL:', window.location.href);
  console.log('ProjectDetail: Current pathname:', window.location.pathname);
  console.log('ProjectDetail: Using projectId instead of projectId:', projectId);
  
  const navigate = useNavigate();
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const [uploadError, setUploadError] = useState<string | null>(null);
  const [analyzingFiles, setAnalyzingFiles] = useState<Set<string>>(new Set());
  const [analysisError, setAnalysisError] = useState<string | null>(null);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [showAnalysisOptions, setShowAnalysisOptions] = useState<string | null>(null);
  const [userContext, setUserContext] = useState<string>('');
  const [timeoutSeconds, setTimeoutSeconds] = useState<number>(300);
  
  const queryClient = useQueryClient();
  const { settings: _, isLoading: settingsLoading, getProvider, getLevel, isConfigured } = useSettings();

  // First validate the project exists
  const { 
    data: project, 
    isLoading: projectLoading, 
    error: projectError 
  } = useQuery(
    ['project', projectId], 
    () => {
      if (!projectId) return Promise.reject('No project ID');
      console.log('ProjectDetail: Fetching project with ID:', projectId);
      return api.projects.getById(projectId);
    },
    {
      enabled: !!projectId,
      retry: false,
      onError: (error) => {
        console.error('ProjectDetail: Failed to fetch project:', error);
        console.error('ProjectDetail: Project ID was:', projectId);
      }
    }
  );

  // Only fetch project files if project exists
  const { 
    data: files, 
    isLoading: filesLoading, 
    error: filesError,
    refetch: refetchFiles 
  } = useQuery(
    ['project-files', projectId], 
    () => projectId ? api.files.getByProject(projectId) : Promise.resolve([]),
    {
      enabled: !!projectId && !!project
    }
  );

  // Only fetch project analyses if project exists
  const { 
    data: analyses, 
    isLoading: analysesLoading, 
    error: analysesError 
  } = useQuery(
    ['project-analyses', projectId], 
    () => projectId ? api.analysis.getByProject(projectId) : Promise.resolve({ analyses: [], pagination: { page: 1, per_page: 20, total: 0 } }),
    {
      enabled: !!projectId && !!project,
      // Add refetch interval when there are running analyses
      refetchInterval: (data) => {
        const hasRunningAnalysis = data?.analyses?.some(analysis => 
          analysis.status === 'running' || analysis.status === 'pending'
        );
        if (hasRunningAnalysis) {
          return 3000; // Poll every 3 seconds when there are running analyses
        }
        return false;
      },
    }
  );

  // Upload file mutation
  const uploadFileMutation = useMutation(
    (file: File) => projectId ? api.files.upload(projectId, file) : Promise.reject('No project ID'),
    {
      onSuccess: () => {
        if (projectId) queryClient.invalidateQueries(['project-files', projectId]);
        setSelectedFile(null);
        setUploadError(null);
      },
      onError: (err: ApiError) => {
        setUploadError(err.message || 'Failed to upload file');
      },
    }
  );

  // Create analysis mutation
  const createAnalysisMutation = useMutation(
    ({ fileId, options }: { fileId: string; options?: { userContext?: string; timeoutSeconds?: number } }) => projectId ? api.analysis.create(projectId, fileId, {
      provider: getProvider(),
      level: getLevel(),
      user_context: options?.userContext,
      timeout_seconds: options?.timeoutSeconds
    }) : Promise.reject('No project ID'),
    {
      onSuccess: () => {
        if (projectId) queryClient.invalidateQueries(['project-analyses', projectId]);
        setAnalysisError(null);
        // Reset analysis options
        setUserContext('');
        setTimeoutSeconds(300);
        setShowAnalysisOptions(null);
      },
      onError: (err: ApiError) => {
        setAnalysisError(err.message || 'Failed to start analysis');
      },
    }
  );

  // Delete file mutation
  const deleteFileMutation = useMutation(
    (fileId: string) => projectId ? api.files.delete(projectId, fileId) : Promise.reject('No project ID'),
    {
      onSuccess: () => {
        if (projectId) queryClient.invalidateQueries(['project-files', projectId]);
      },
      onError: (err: ApiError) => {
        console.error('Failed to delete file:', err);
      },
    }
  );

  const handleFileSelect = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      setSelectedFile(file);
      setUploadError(null);
    }
  };

  const handleUpload = async () => {
    if (!selectedFile || !projectId) return;

    setIsUploading(true);
    try {
      await uploadFileMutation.mutateAsync(selectedFile);
    } catch (err) {
      // Error is handled by mutation onError
    } finally {
      setIsUploading(false);
    }
  };

  const handleStartAnalysis = async (fileId: string, options?: { userContext?: string; selectedModel?: string; timeoutSeconds?: number }) => {
    // Check if settings are configured properly
    if (!isConfigured()) {
      setAnalysisError(
        `Analysis requires configuration. Please go to Settings and configure your AI provider (${getProvider()}) and API key.`
      );
      return;
    }

    setAnalyzingFiles(prev => new Set(prev).add(fileId));
    setAnalysisError(null);
    try {
      await createAnalysisMutation.mutateAsync({ fileId, options });
    } catch (err) {
      // Error is handled by mutation onError
    } finally {
      setAnalyzingFiles(prev => {
        const next = new Set(prev);
        next.delete(fileId);
        return next;
      });
    }
  };

  const handleDeleteFile = async (fileId: string) => {
    if (window.confirm('Are you sure you want to delete this file?')) {
      setDeleteError(null);
      try {
        await deleteFileMutation.mutateAsync(fileId);
      } catch (err) {
        console.error('Failed to delete file:', err);
        const errorMessage = err instanceof Error ? err.message : 'Failed to delete file';
        setDeleteError(errorMessage);
      }
    }
  };

  const formatFileSize = (bytes: number) => {
    if (!bytes || bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'completed':
        return <CheckCircleIcon className="h-5 w-5 text-green-500" />;
      case 'failed':
        return <XCircleIcon className="h-5 w-5 text-red-500" />;
      case 'running':
        return <ClockIcon className="h-5 w-5 text-blue-500 animate-spin" />;
      default:
        return <ClockIcon className="h-5 w-5 text-gray-500" />;
    }
  };

  const getStatusText = (status: string) => {
    switch (status) {
      case 'completed': return 'Completed';
      case 'failed': return 'Failed';
      case 'running': return 'Running';
      case 'pending': return 'Pending';
      default: return status;
    }
  };

  const handleViewAnalysis = (analysisId: string) => {
    navigate(`/analysis/${analysisId}`);
  };

  // Handle project loading state
  if (projectLoading) {
    return (
      <div className="flex items-center justify-center min-h-96">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  // Handle project not found or error
  if (projectError || !project) {
    console.log('ProjectDetail: Showing NotFound component');
    console.log('ProjectDetail: projectError:', projectError);
    console.log('ProjectDetail: project:', project);
    console.log('ProjectDetail: projectId:', projectId);
    return <NotFound />;
  }

  if (filesLoading || analysesLoading) {
    return (
      <div className="flex items-center justify-center min-h-96">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      {/* Project header */}
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
          Project Details
        </h1>
        <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
          Upload log files and start analysis for your project.
        </p>
      </div>

      {/* File Upload Section */}
      <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6 mb-8">
        <h2 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
          Upload Log Files
        </h2>
        
        <div className="space-y-4">
          <div className="border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg p-6">
            <div className="text-center">
              <ArrowUpTrayIcon className="mx-auto h-12 w-12 text-gray-400" />
              <div className="mt-4">
                <label htmlFor="file-upload" className="cursor-pointer">
                  <span className="mt-2 block text-sm font-medium text-gray-900 dark:text-gray-300">
                    Choose a log file
                  </span>
                  <span className="mt-1 block text-xs text-gray-500 dark:text-gray-400">
                    Supported formats: .log, .txt
                  </span>
                </label>
                <input
                  id="file-upload"
                  name="file-upload"
                  type="file"
                  className="sr-only"
                  accept=".log,.txt"
                  onChange={handleFileSelect}
                  disabled={isUploading}
                />
              </div>
            </div>
          </div>

          {selectedFile && (
            <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-3">
                  <DocumentTextIcon className="h-5 w-5 text-gray-400" />
                  <div>
                    <p className="text-sm font-medium text-gray-900 dark:text-white">
                      {selectedFile.name}
                    </p>
                    <p className="text-xs text-gray-500 dark:text-gray-400">
                      {formatFileSize(selectedFile.size)}
                    </p>
                  </div>
                </div>
                <button
                  type="button"
                  onClick={() => setSelectedFile(null)}
                  className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                >
                  <XCircleIcon className="h-5 w-5" />
                </button>
              </div>
            </div>
          )}

          {uploadError && (
            <div className="bg-error-50 dark:bg-error-900/20 border border-error-200 dark:border-error-800 rounded-md p-3">
              <p className="text-sm text-error-700 dark:text-error-300">
                {uploadError}
              </p>
            </div>
          )}

          <div className="flex justify-end">
            <button
              type="button"
              onClick={handleUpload}
              disabled={!selectedFile || isUploading}
              className="btn-primary"
            >
              {isUploading ? (
                <>
                  <svg className="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                  Uploading...
                </>
              ) : (
                'Upload File'
              )}
            </button>
          </div>
        </div>
      </div>

      {/* Files Section */}
      <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6 mb-8">
        <h2 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
          Log Files ({files?.length || 0})
        </h2>

        {filesError ? (
          <div className="bg-error-50 dark:bg-error-900/20 border border-error-200 dark:border-error-800 rounded-md p-4">
            <p className="text-sm text-error-700 dark:text-error-300">
              {filesError instanceof Error ? filesError.message : 'Failed to load files'}
            </p>
            <button
              type="button"
              onClick={() => refetchFiles()}
              className="mt-2 text-sm font-medium text-error-800 dark:text-error-200 hover:text-error-900 dark:hover:text-error-100"
            >
              Try again
            </button>
          </div>
        ) : files && files.length > 0 ? (
          <div className="space-y-4">
            {files.map((file: LogFile) => (
              <div key={file.id} className="border border-gray-200 dark:border-gray-700 rounded-lg p-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-3">
                    <DocumentTextIcon className="h-5 w-5 text-gray-400" />
                    <div>
                      <h3 className="text-sm font-medium text-gray-900 dark:text-white">
                        {file.filename}
                      </h3>
                      <p className="text-xs text-gray-500 dark:text-gray-400">
                        {formatFileSize(file.file_size)} • {file.line_count?.toLocaleString() || '0'} lines
                        <br />
                        Uploaded {new Date(file.created_at).toLocaleDateString()}
                      </p>
                    </div>
                  </div>
                  <div className="flex items-center space-x-2">
                    <button
                      type="button"
                      onClick={() => setShowAnalysisOptions(showAnalysisOptions === file.id ? null : file.id)}
                      disabled={analyzingFiles.has(file.id) || settingsLoading || !isConfigured()}
                      className="btn-secondary"
                      title={!isConfigured() ? `Configure ${getProvider()} provider in Settings` : 'Analysis options'}
                    >
                      <svg className="h-4 w-4 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />
                      </svg>
                      Options
                    </button>
                    <button
                      type="button"
                      onClick={() => handleStartAnalysis(file.id, {
                        userContext: userContext || undefined,
                        timeoutSeconds: timeoutSeconds
                      })}
                      disabled={analyzingFiles.has(file.id) || settingsLoading || !isConfigured()}
                      className="btn-primary"
                      title={!isConfigured() ? `Configure ${getProvider()} provider in Settings` : `Analyze with ${getProvider()}`}
                    >
                      <ChartBarIcon className="h-4 w-4 mr-1" />
                      {analyzingFiles.has(file.id)
                        ? 'Analyzing...'
                        : settingsLoading
                        ? 'Loading...'
                        : `Analyze (${getProvider()})`}
                    </button>
                    <button
                      type="button"
                      onClick={() => handleDeleteFile(file.id)}
                      className="btn-secondary"
                    >
                      <TrashIcon className="h-4 w-4 mr-1" />
                      Delete
                    </button>
                  </div>
                </div>

                {/* Analysis Options Panel */}
                {showAnalysisOptions === file.id && (
                  <div className="mt-4 p-4 bg-gray-50 dark:bg-gray-700 rounded-lg space-y-4">
                    <h4 className="text-sm font-medium text-gray-900 dark:text-white">Analysis Options</h4>
                    
                    {/* User Context */}
                    <div>
                      <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">
                        User Context (optional)
                      </label>
                      <textarea
                        className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-800 dark:text-white text-sm"
                        placeholder="Provide additional context for analysis (max 2000 characters)..."
                        value={userContext}
                        onChange={(e) => setUserContext(e.target.value.slice(0, 2000))}
                        rows={3}
                      />
                      <div className="flex justify-between mt-1">
                        <span className="text-xs text-gray-500 dark:text-gray-400">
                          Add context about your application, environment, or specific concerns
                        </span>
                        <span className="text-xs text-gray-500 dark:text-gray-400">
                          {userContext.length}/2000
                        </span>
                      </div>
                    </div>

                    {/* Timeout Configuration */}
                    <div>
                      <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">
                        Timeout (seconds)
                      </label>
                      <input
                        type="number"
                        className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-800 dark:text-white text-sm"
                        min="60"
                        max="1800"
                        value={timeoutSeconds}
                        onChange={(e) => setTimeoutSeconds(parseInt(e.target.value) || 300)}
                      />
                      <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                        Set custom timeout (60-1800 seconds)
                      </p>
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        ) : (
          <div className="text-center py-8">
            <DocumentTextIcon className="mx-auto h-12 w-12 text-gray-400" />
            <h3 className="mt-2 text-sm font-medium text-gray-900 dark:text-white">
              No log files yet
            </h3>
            <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
              Upload a log file to get started with analysis.
            </p>
          </div>
        )}
      </div>

      {/* Analyses Section */}
      <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6">
        <h2 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
          Recent Analyses ({analyses?.analyses?.length || 0})
        </h2>

        {analysesError ? (
          <div className="bg-error-50 dark:bg-error-900/20 border border-error-200 dark:border-error-800 rounded-md p-4">
            <p className="text-sm text-error-700 dark:text-error-300">
              {analysesError instanceof Error ? analysesError.message : 'Failed to load analyses'}
            </p>
          </div>
        ) : analyses && analyses.analyses && analyses.analyses.length > 0 ? (
          <div className="space-y-4">
            {analyses.analyses.map((analysis: Analysis) => (
              <div 
                key={analysis.id} 
                className="border border-gray-200 dark:border-gray-700 rounded-lg p-4 cursor-pointer hover:border-blue-300 dark:hover:border-blue-600 transition-colors"
                onClick={() => handleViewAnalysis(analysis.id)}
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-3">
                    {getStatusIcon(analysis.status)}
                    <div>
                      <h3 className="text-sm font-medium text-gray-900 dark:text-white">
                        {analysis.ai_provider} Analysis
                      </h3>
                      <p className="text-xs text-gray-500 dark:text-gray-400">
                        {analysis.ai_provider} • Started {new Date(analysis.created_at).toLocaleDateString()}
                        {analysis.updated_at && (
                          <span> • Updated {new Date(analysis.updated_at).toLocaleDateString()}</span>
                        )}
                      </p>
                      {analysis.error && (
                        <p className="text-xs text-error-600 dark:text-error-400 mt-1">
                          Error: {analysis.error}
                        </p>
                      )}
                    </div>
                  </div>
                  <div className="flex items-center space-x-2">
                    <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                      analysis.status === 'completed' ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200' :
                      analysis.status === 'failed' ? 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200' :
                      analysis.status === 'running' ? 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200' :
                      'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200'
                    }`}>
                      {getStatusText(analysis.status)}
                    </span>
                    <ArrowRightIcon className="h-4 w-4 text-gray-400" />
                  </div>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="text-center py-8">
            <ChartBarIcon className="mx-auto h-12 w-12 text-gray-400" />
            <h3 className="mt-2 text-sm font-medium text-gray-900 dark:text-white">
              No analyses yet
            </h3>
            <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
              Upload a log file and click "Analyze" to start your first analysis.
            </p>
          </div>
        )}
      </div>

      {analysisError && (
        <div className="fixed bottom-4 right-4 bg-error-50 dark:bg-error-900/20 border border-error-200 dark:border-error-800 rounded-md p-4 max-w-md">
          <p className="text-sm text-error-700 dark:text-error-300">
            Analysis Error: {analysisError}
          </p>
        </div>
      )}

      {deleteError && (
        <div className="fixed bottom-16 right-4 bg-error-50 dark:bg-error-900/20 border border-error-200 dark:border-error-800 rounded-md p-4 max-w-md">
          <div className="flex justify-between items-start">
            <p className="text-sm text-error-700 dark:text-error-300">
              Delete Error: {deleteError}
            </p>
            <button
              type="button"
              onClick={() => setDeleteError(null)}
              className="ml-2 text-error-400 hover:text-error-600"
            >
              ×
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

export default ProjectDetail;