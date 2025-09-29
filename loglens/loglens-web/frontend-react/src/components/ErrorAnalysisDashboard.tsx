import { PieChart, Pie, Cell, BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import { ExclamationTriangleIcon } from '@heroicons/react/24/outline';
import { ErrorAnalysis, ErrorCategory } from '@/types';

interface ErrorAnalysisDashboardProps {
  errors: ErrorAnalysis[];
  loading: boolean;
}

const categoryColors = {
  code_related: '#ef4444',
  infrastructure_related: '#f59e0b',
  configuration_related: '#8b5cf6',
  external_service_related: '#06b6d4',
  unknown_related: '#6b7280'
};

const severityColors = {
  low: '#10b981',
  medium: '#f59e0b',
  high: '#ef4444',
  critical: '#dc2626'
};

const categoryLabels: Record<ErrorCategory, string> = {
  code_related: 'Code Related',
  infrastructure_related: 'Infrastructure',
  configuration_related: 'Configuration',
  external_service_related: 'External Service',
  unknown_related: 'Unknown'
};

const severityLabels = {
  low: 'Low',
  medium: 'Medium',
  high: 'High',
  critical: 'Critical'
};

export function ErrorAnalysisDashboard({ errors, loading }: ErrorAnalysisDashboardProps) {
  if (loading) {
    return (
      <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6 mb-6">
        <div className="animate-pulse">
          <div className="h-8 bg-gray-200 dark:bg-gray-700 rounded mb-4"></div>
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <div className="h-64 bg-gray-200 dark:bg-gray-700 rounded"></div>
            <div className="h-64 bg-gray-200 dark:bg-gray-700 rounded"></div>
          </div>
        </div>
      </div>
    );
  }

  if (!errors || errors.length === 0) {
    return (
      <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6 mb-6">
        <div className="flex items-center space-x-2 mb-4">
          <ExclamationTriangleIcon className="h-5 w-5 text-gray-400" />
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            Error Analysis
          </h2>
        </div>
        <p className="text-gray-500 dark:text-gray-400">
          No errors detected in the analyzed logs.
        </p>
      </div>
    );
  }

  // Prepare data for charts
  const categoryData = Object.entries(
    errors.reduce((acc, error) => {
      const category = error.category;
      acc[category] = (acc[category] || 0) + error.frequency;
      return acc;
    }, {} as Record<ErrorCategory, number>)
  ).map(([category, frequency]) => ({
    category: categoryLabels[category as ErrorCategory],
    frequency,
    color: categoryColors[category as ErrorCategory]
  }));

  const severityData = Object.entries(
    errors.reduce((acc, error) => {
      const severity = error.severity;
      acc[severity] = (acc[severity] || 0) + 1;
      return acc;
    }, {} as Record<string, number>)
  ).map(([severity, count]) => ({
    severity: severityLabels[severity as keyof typeof severityLabels],
    count,
    color: severityColors[severity as keyof typeof severityColors]
  }));

  const topErrors = errors
    .sort((a, b) => b.frequency - a.frequency)
    .slice(0, 5);

  return (
    <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6 mb-6">
      <div className="flex items-center space-x-2 mb-6">
        <ExclamationTriangleIcon className="h-5 w-5 text-red-600" />
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
          Error Analysis
        </h2>
        <span className="ml-2 bg-red-100 text-red-800 text-xs font-medium px-2.5 py-0.5 rounded-full">
          {errors.length} errors
        </span>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        <div>
          <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-4">
            Error Categories
          </h3>
          <div className="h-64">
            <ResponsiveContainer width="100%" height="100%">
              <PieChart>
                <Pie
                  data={categoryData}
                  cx="50%"
                  cy="50%"
                  outerRadius={80}
                  fill="#8884d8"
                  dataKey="frequency"
                  label={({ category, frequency }) => `${category}: ${frequency}`}
                >
                  {categoryData.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={entry.color} />
                  ))}
                </Pie>
                <Tooltip />
              </PieChart>
            </ResponsiveContainer>
          </div>
        </div>

        <div>
          <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-4">
            Error Severity Distribution
          </h3>
          <div className="h-64">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={severityData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="severity" />
                <YAxis />
                <Tooltip />
                <Bar dataKey="count" fill="#8884d8">
                  {severityData.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={entry.color} />
                  ))}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          </div>
        </div>
      </div>

      <div>
        <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-4">
          Top Errors by Frequency
        </h3>
        <div className="space-y-3">
          {topErrors.map((error, index) => (
            <div key={index} className="border border-gray-200 dark:border-gray-700 rounded-lg p-4">
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center space-x-2 mb-2">
                    <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium"
                          style={{ backgroundColor: `${categoryColors[error.category]}20`, color: categoryColors[error.category] }}>
                      {categoryLabels[error.category]}
                    </span>
                    <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium"
                          style={{ backgroundColor: `${severityColors[error.severity]}20`, color: severityColors[error.severity] }}>
                      {severityLabels[error.severity]}
                    </span>
                    <span className="text-xs text-gray-500 dark:text-gray-400">
                      {error.frequency} occurrences
                    </span>
                  </div>
                  <h4 className="text-sm font-medium text-gray-900 dark:text-gray-100 mb-1">
                    {error.description}
                  </h4>
                  {error.file_location && (
                    <p className="text-xs text-gray-500 dark:text-gray-400 mb-2">
                      File: {error.file_location}
                    </p>
                  )}
                  {error.context && error.context.length > 0 && (
                    <div className="mt-2">
                      <p className="text-xs text-gray-600 dark:text-gray-300 mb-1">Context:</p>
                      <div className="bg-gray-50 dark:bg-gray-700 rounded p-2">
                        <p className="text-xs text-gray-700 dark:text-gray-300">
                          {error.context[0]}
                        </p>
                      </div>
                    </div>
                  )}
                </div>
                <div className="flex-shrink-0 ml-4">
                  <div className="w-8 h-8 bg-gray-100 dark:bg-gray-700 rounded-full flex items-center justify-center">
                    <span className="text-sm font-bold text-gray-600 dark:text-gray-400">
                      {index + 1}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}