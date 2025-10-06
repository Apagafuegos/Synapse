import { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { FunnelIcon, MagnifyingGlassIcon } from '@heroicons/react/24/outline';
import LoadingSpinner from '@components/LoadingSpinner';

interface ErrorPattern {
  id: string;
  project_id: string;
  pattern: string;
  description?: string;
  category: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  frequency: number;
  example_lines?: string;
  created_at: string;
  updated_at: string;
}

const severityColors = {
  critical: 'bg-red-100 text-red-800 border-red-300',
  high: 'bg-orange-100 text-orange-800 border-orange-300',
  medium: 'bg-yellow-100 text-yellow-800 border-yellow-300',
  low: 'bg-blue-100 text-blue-800 border-blue-300',
};

const categoryColors = {
  code: 'bg-purple-100 text-purple-800',
  infrastructure: 'bg-green-100 text-green-800',
  configuration: 'bg-indigo-100 text-indigo-800',
  external: 'bg-pink-100 text-pink-800',
};

export default function PatternsPage() {
  const { projectId } = useParams<{ projectId: string }>();
  const [patterns, setPatterns] = useState<ErrorPattern[]>([]);
  const [loading, setLoading] = useState(true);
  const [categoryFilter, setCategoryFilter] = useState<string>('all');
  const [severityFilter, setSeverityFilter] = useState<string>('all');
  const [searchTerm, setSearchTerm] = useState('');

  useEffect(() => {
    fetchPatterns();
  }, [projectId, categoryFilter, severityFilter]);

  const fetchPatterns = async () => {
    try {
      setLoading(true);
      const params = new URLSearchParams();
      if (categoryFilter !== 'all') params.append('category', categoryFilter);
      if (severityFilter !== 'all') params.append('severity', severityFilter);

      const response = await fetch(`/api/projects/${projectId}/patterns?${params}`);
      if (!response.ok) throw new Error('Failed to fetch patterns');

      const data = await response.json();
      setPatterns(data);
    } catch (error) {
      console.error('Error fetching patterns:', error);
    } finally {
      setLoading(false);
    }
  };

  const filteredPatterns = patterns.filter(pattern =>
    searchTerm === '' ||
    pattern.pattern.toLowerCase().includes(searchTerm.toLowerCase()) ||
    pattern.description?.toLowerCase().includes(searchTerm.toLowerCase())
  );

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="mb-6">
        <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">
          Error Patterns
        </h1>
        <p className="text-gray-600 dark:text-gray-400">
          Recurring error patterns detected across your project
        </p>
      </div>

      {/* Filters */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-4 mb-6">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {/* Search */}
          <div className="relative">
            <MagnifyingGlassIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-gray-400" />
            <input
              type="text"
              placeholder="Search patterns..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
            />
          </div>

          {/* Category Filter */}
          <div className="relative">
            <FunnelIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-gray-400" />
            <select
              value={categoryFilter}
              onChange={(e) => setCategoryFilter(e.target.value)}
              className="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
            >
              <option value="all">All Categories</option>
              <option value="code">Code Errors</option>
              <option value="infrastructure">Infrastructure</option>
              <option value="configuration">Configuration</option>
              <option value="external">External Services</option>
            </select>
          </div>

          {/* Severity Filter */}
          <div className="relative">
            <FunnelIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-gray-400" />
            <select
              value={severityFilter}
              onChange={(e) => setSeverityFilter(e.target.value)}
              className="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
            >
              <option value="all">All Severities</option>
              <option value="critical">Critical</option>
              <option value="high">High</option>
              <option value="medium">Medium</option>
              <option value="low">Low</option>
            </select>
          </div>
        </div>
      </div>

      {/* Patterns List */}
      {loading ? (
        <div className="flex justify-center py-12">
          <LoadingSpinner size="lg" />
        </div>
      ) : filteredPatterns.length === 0 ? (
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-8 text-center">
          <p className="text-gray-500 dark:text-gray-400">
            No patterns found matching your filters.
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-4">
          {filteredPatterns.map((pattern) => (
            <div
              key={pattern.id}
              className="bg-white dark:bg-gray-800 rounded-lg shadow p-6 hover:shadow-lg transition-shadow"
            >
              <div className="flex items-start justify-between mb-3">
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-2">
                    <span className={`px-3 py-1 rounded-full text-xs font-medium ${categoryColors[pattern.category as keyof typeof categoryColors] || 'bg-gray-100 text-gray-800'}`}>
                      {pattern.category}
                    </span>
                    <span className={`px-3 py-1 rounded-full text-xs font-medium border ${severityColors[pattern.severity]}`}>
                      {pattern.severity.toUpperCase()}
                    </span>
                  </div>
                  <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                    {pattern.pattern}
                  </h3>
                  {pattern.description && (
                    <p className="text-gray-600 dark:text-gray-400 text-sm mb-2">
                      {pattern.description}
                    </p>
                  )}
                </div>
                <div className="text-right ml-4">
                  <div className="text-2xl font-bold text-gray-900 dark:text-white">
                    {pattern.frequency}
                  </div>
                  <div className="text-xs text-gray-500 dark:text-gray-400">
                    occurrences
                  </div>
                </div>
              </div>

              {pattern.example_lines && (
                <div className="mt-4 bg-gray-50 dark:bg-gray-900 rounded p-3">
                  <p className="text-xs text-gray-500 dark:text-gray-400 mb-1">Example:</p>
                  <code className="text-xs text-gray-700 dark:text-gray-300 font-mono">
                    {pattern.example_lines}
                  </code>
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
