import { useState, useEffect } from 'react';
import { MagnifyingGlassIcon, GlobeAltIcon, CheckCircleIcon } from '@heroicons/react/24/outline';
import LoadingSpinner from '@components/LoadingSpinner';

interface KnowledgeBaseEntry {
  id: string;
  project_id: string;
  title: string;
  problem_description: string;
  solution: string;
  tags?: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  usage_count: number;
  is_public: boolean;
  created_at: string;
  updated_at: string;
}

const severityColors = {
  critical: 'bg-red-100 text-red-800 border-red-300',
  high: 'bg-orange-100 text-orange-800 border-orange-300',
  medium: 'bg-yellow-100 text-yellow-800 border-yellow-300',
  low: 'bg-blue-100 text-blue-800 border-blue-300',
};

export default function PublicKnowledgePage() {
  const [knowledge, setKnowledge] = useState<KnowledgeBaseEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchTerm, setSearchTerm] = useState('');
  const [expandedId, setExpandedId] = useState<string | null>(null);

  useEffect(() => {
    fetchPublicKnowledge();
  }, [searchTerm]);

  const fetchPublicKnowledge = async () => {
    try {
      setLoading(true);
      const params = new URLSearchParams();
      if (searchTerm) params.append('search', searchTerm);

      const response = await fetch(`/api/knowledge/public?${params}`);
      if (!response.ok) throw new Error('Failed to fetch knowledge base');

      const data = await response.json();
      setKnowledge(data);
    } catch (error) {
      console.error('Error fetching knowledge:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="mb-6">
        <div className="flex items-center gap-3 mb-2">
          <GlobeAltIcon className="h-8 w-8 text-blue-600" />
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
            Public Knowledge Base
          </h1>
        </div>
        <p className="text-gray-600 dark:text-gray-400">
          Community-shared solutions for common problems
        </p>
      </div>

      {/* Search */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-4 mb-6">
        <div className="relative">
          <MagnifyingGlassIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-gray-400" />
          <input
            type="search"
            placeholder="Search solutions..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
          />
        </div>
      </div>

      {/* Knowledge Grid */}
      {loading ? (
        <div className="flex justify-center py-12">
          <LoadingSpinner size="lg" />
        </div>
      ) : knowledge.length === 0 ? (
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-8 text-center">
          <p className="text-gray-500 dark:text-gray-400">
            No public knowledge entries found.
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {knowledge.map((entry) => (
            <div
              key={entry.id}
              className="bg-white dark:bg-gray-800 rounded-lg shadow hover:shadow-lg transition-shadow"
            >
              <div className="p-6">
                <div className="flex items-start justify-between mb-3">
                  <h3 className="text-lg font-semibold text-gray-900 dark:text-white flex-1">
                    {entry.title}
                  </h3>
                  <span className={`px-3 py-1 rounded-full text-xs font-medium border ${severityColors[entry.severity]}`}>
                    {entry.severity.toUpperCase()}
                  </span>
                </div>

                <p className="text-gray-600 dark:text-gray-400 text-sm mb-4">
                  {entry.problem_description}
                </p>

                {/* Solution */}
                <details
                  open={expandedId === entry.id}
                  onToggle={() => setExpandedId(expandedId === entry.id ? null : entry.id)}
                  className="mb-4"
                >
                  <summary className="cursor-pointer text-blue-600 dark:text-blue-400 font-medium text-sm hover:text-blue-700 dark:hover:text-blue-300">
                    View Solution
                  </summary>
                  <div className="mt-3 bg-gray-50 dark:bg-gray-900 rounded-lg p-4">
                    <pre className="text-sm text-gray-700 dark:text-gray-300 whitespace-pre-wrap font-sans">
                      {entry.solution}
                    </pre>
                  </div>
                </details>

                {/* Metadata */}
                <div className="flex items-center justify-between text-xs text-gray-500 dark:text-gray-400 border-t border-gray-200 dark:border-gray-700 pt-3">
                  <div className="flex items-center gap-4">
                    <span className="flex items-center gap-1">
                      <CheckCircleIcon className="h-4 w-4" />
                      Used {entry.usage_count} times
                    </span>
                    {entry.tags && (
                      <span className="flex items-center gap-1">
                        Tags: {entry.tags}
                      </span>
                    )}
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
