import { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import {
  SignalIcon,
  PlusIcon,
  XMarkIcon,
  StopIcon,
} from '@heroicons/react/24/outline';
import LoadingSpinner from '@components/LoadingSpinner';

interface StreamingSource {
  source_id: string;
  name: string;
  source_type: string;
  project_id: string;
  status: 'active' | 'inactive' | 'error';
  created_at: string;
}

interface StreamingStats {
  active_sources: number;
  active_connections: number;
  total_logs_processed: number;
}

interface CreateSourceModalProps {
  projectId: string;
  onSubmit: (config: any) => void;
  onCancel: () => void;
}

function CreateSourceModal({ projectId, onSubmit, onCancel }: CreateSourceModalProps) {
  const [sourceType, setSourceType] = useState('file');
  const [name, setName] = useState('');
  const [config, setConfig] = useState<any>({});

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSubmit({
      project_id: projectId,
      name,
      source_type: sourceType,
      config,
      parser_config: {
        log_format: 'text',
      },
      buffer_size: 100,
      batch_timeout_seconds: 2,
      restart_on_error: true,
    });
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-lg w-full">
        <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
            Create Streaming Source
          </h2>
          <button
            onClick={onCancel}
            className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
          >
            <XMarkIcon className="h-6 w-6" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-6 space-y-4">
          {/* Source Name */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Source Name *
            </label>
            <input
              type="text"
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g., Application Logs"
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
            />
          </div>

          {/* Source Type */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Source Type *
            </label>
            <select
              required
              value={sourceType}
              onChange={(e) => {
                setSourceType(e.target.value);
                setConfig({});
              }}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
            >
              <option value="file">File (tail -f)</option>
              <option value="command">Command Output</option>
              <option value="tcp">TCP Listener</option>
              <option value="http">HTTP Endpoint</option>
            </select>
          </div>

          {/* Source-specific config */}
          {sourceType === 'file' && (
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                File Path *
              </label>
              <input
                type="text"
                required
                value={config.path || ''}
                onChange={(e) => setConfig({ ...config, path: e.target.value })}
                placeholder="/var/log/app.log"
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
              />
            </div>
          )}

          {sourceType === 'command' && (
            <>
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  Command *
                </label>
                <input
                  type="text"
                  required
                  value={config.command || ''}
                  onChange={(e) => setConfig({ ...config, command: e.target.value })}
                  placeholder="journalctl -f"
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  Arguments (comma-separated)
                </label>
                <input
                  type="text"
                  value={config.args?.join(', ') || ''}
                  onChange={(e) =>
                    setConfig({
                      ...config,
                      args: e.target.value.split(',').map((s) => s.trim()).filter(Boolean),
                    })
                  }
                  placeholder="-u, myapp.service"
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
                />
              </div>
            </>
          )}

          {sourceType === 'tcp' && (
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                TCP Port *
              </label>
              <input
                type="number"
                required
                value={config.port || ''}
                onChange={(e) => setConfig({ ...config, port: parseInt(e.target.value) })}
                placeholder="5140"
                min="1"
                max="65535"
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
              />
            </div>
          )}

          {sourceType === 'http' && (
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Endpoint Path *
              </label>
              <input
                type="text"
                required
                value={config.path || ''}
                onChange={(e) => setConfig({ ...config, path: e.target.value })}
                placeholder="/ingest/logs"
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
              />
            </div>
          )}

          {/* Actions */}
          <div className="flex justify-end gap-3 pt-4 border-t border-gray-200 dark:border-gray-700">
            <button
              type="button"
              onClick={onCancel}
              className="px-4 py-2 text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={!name || Object.keys(config).length === 0}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
            >
              Create Source
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

export default function StreamingPage() {
  const { projectId } = useParams<{ projectId: string }>();
  const [sources, setSources] = useState<StreamingSource[]>([]);
  const [stats, setStats] = useState<StreamingStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);

  useEffect(() => {
    if (projectId) {
      fetchSources();
      fetchStats();

      // Poll stats every 5 seconds
      const interval = setInterval(fetchStats, 5000);
      return () => clearInterval(interval);
    }
  }, [projectId]);

  const fetchSources = async () => {
    try {
      setLoading(true);
      const response = await fetch(`/api/projects/${projectId}/streaming/sources`);
      if (!response.ok) throw new Error('Failed to fetch sources');

      const data = await response.json();
      setSources(data);
    } catch (error) {
      console.error('Error fetching sources:', error);
    } finally {
      setLoading(false);
    }
  };

  const fetchStats = async () => {
    try {
      const response = await fetch(`/api/projects/${projectId}/streaming/stats`);
      if (!response.ok) throw new Error('Failed to fetch stats');

      const data = await response.json();
      setStats(data);
    } catch (error) {
      console.error('Error fetching stats:', error);
    }
  };

  const createSource = async (sourceConfig: any) => {
    try {
      const response = await fetch(`/api/projects/${projectId}/streaming/sources`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(sourceConfig),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.error || 'Failed to create source');
      }

      setShowCreateModal(false);
      fetchSources();
    } catch (error) {
      console.error('Error creating source:', error);
      alert(error instanceof Error ? error.message : 'Failed to create source');
    }
  };

  const stopSource = async (sourceId: string) => {
    if (!confirm('Are you sure you want to stop this streaming source?')) {
      return;
    }

    try {
      const response = await fetch(
        `/api/projects/${projectId}/streaming/sources/${sourceId}`,
        { method: 'DELETE' }
      );

      if (!response.ok) throw new Error('Failed to stop source');

      fetchSources();
    } catch (error) {
      console.error('Error stopping source:', error);
      alert('Failed to stop source');
    }
  };

  const statusColors = {
    active: 'bg-green-100 text-green-800 border-green-300',
    inactive: 'bg-gray-100 text-gray-800 border-gray-300',
    error: 'bg-red-100 text-red-800 border-red-300',
  };

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="flex items-center justify-between mb-6">
        <div>
          <div className="flex items-center gap-3 mb-2">
            <SignalIcon className="h-8 w-8 text-blue-600" />
            <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
              Real-Time Log Streaming
            </h1>
          </div>
          <p className="text-gray-600 dark:text-gray-400">
            Monitor and analyze logs as they arrive in real-time
          </p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
        >
          <PlusIcon className="h-5 w-5" />
          New Source
        </button>
      </div>

      {/* Stats Cards */}
      {stats && (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
            <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2">
              Active Sources
            </h3>
            <div className="text-3xl font-bold text-gray-900 dark:text-white">
              {stats.active_sources}
            </div>
          </div>
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
            <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2">
              Live Connections
            </h3>
            <div className="text-3xl font-bold text-gray-900 dark:text-white">
              {stats.active_connections}
            </div>
          </div>
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
            <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2">
              Logs Processed
            </h3>
            <div className="text-3xl font-bold text-gray-900 dark:text-white">
              {stats.total_logs_processed.toLocaleString()}
            </div>
          </div>
        </div>
      )}

      {/* Sources List */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
        <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            Active Sources
          </h2>
        </div>

        {loading ? (
          <div className="flex justify-center py-12">
            <LoadingSpinner size="lg" />
          </div>
        ) : sources.length === 0 ? (
          <div className="p-8 text-center text-gray-500 dark:text-gray-400">
            <p>No streaming sources configured.</p>
            <p className="text-sm mt-1">Create one to get started!</p>
          </div>
        ) : (
          <div className="divide-y divide-gray-200 dark:divide-gray-700">
            {sources.map((source) => (
              <div key={source.source_id} className="p-6 hover:bg-gray-50 dark:hover:bg-gray-700/50">
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-2">
                      <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                        {source.name}
                      </h3>
                      <span className={`px-3 py-1 rounded-full text-xs font-medium border ${statusColors[source.status]}`}>
                        {source.status.toUpperCase()}
                      </span>
                    </div>
                    <p className="text-sm text-gray-600 dark:text-gray-400 mb-1">
                      Type: <span className="font-medium">{source.source_type}</span>
                    </p>
                    <p className="text-xs text-gray-500 dark:text-gray-500">
                      Created: {new Date(source.created_at).toLocaleString()}
                    </p>
                  </div>
                  <button
                    onClick={() => stopSource(source.source_id)}
                    className="flex items-center gap-2 px-3 py-2 text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
                  >
                    <StopIcon className="h-4 w-4" />
                    Stop
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Create Source Modal */}
      {showCreateModal && projectId && (
        <CreateSourceModal
          projectId={projectId}
          onSubmit={createSource}
          onCancel={() => setShowCreateModal(false)}
        />
      )}
    </div>
  );
}
