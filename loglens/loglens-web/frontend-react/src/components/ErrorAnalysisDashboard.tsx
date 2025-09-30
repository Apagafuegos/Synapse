import { PieChart, Pie, Cell, BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from 'recharts';
import { ExclamationTriangleIcon } from '@heroicons/react/24/outline';
import { ErrorAnalysis, ErrorCategory } from '@/types';
import { CustomTooltip, ChartContainer, chartColors, chartGridProps, chartAxisProps } from './ChartComponents';

interface ErrorAnalysisDashboardProps {
  errors: ErrorAnalysis[];
  loading: boolean;
}

// Updated colors using Tailwind theme
const categoryColors: Record<ErrorCategory | 'unknown', string> = {
  code_related: chartColors.error.main,
  infrastructure_related: chartColors.warning.main,
  configuration_related: chartColors.purple.main,
  external_service_related: chartColors.cyan.main,
  unknown_related: chartColors.gray.main,
  unknown: chartColors.gray.main, // Fallback for truly unknown categories
};

const severityColors: Record<string, string> = {
  low: chartColors.success.main,
  medium: chartColors.warning.main,
  high: chartColors.error.main,
  critical: chartColors.error.dark,
};

const categoryLabels: Record<ErrorCategory | 'unknown', string> = {
  code_related: 'Code Related',
  infrastructure_related: 'Infrastructure',
  configuration_related: 'Configuration',
  external_service_related: 'External Service',
  unknown_related: 'Unknown',
  unknown: 'Unknown', // Fallback label
};

const severityLabels: Record<string, string> = {
  low: 'Low',
  medium: 'Medium',
  high: 'High',
  critical: 'Critical',
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

  // Prepare data for charts with proper fallback handling
  const categoryData = Object.entries(
    errors.reduce((acc, error) => {
      // Ensure category exists and is valid, otherwise use 'unknown'
      const category = error.category || 'unknown';
      acc[category] = (acc[category] || 0) + error.frequency;
      return acc;
    }, {} as Record<string, number>)
  ).map(([category, frequency]) => {
    // Safe category lookup with fallback
    const categoryKey = category as ErrorCategory | 'unknown';
    const label = categoryLabels[categoryKey] || categoryLabels.unknown || 'Unknown';
    const color = categoryColors[categoryKey] || categoryColors.unknown;

    return {
      name: label, // Use 'name' for better Recharts compatibility
      category: label,
      frequency,
      color
    };
  }).filter(item => item.frequency > 0); // Filter out zero-frequency items

  const severityData = Object.entries(
    errors.reduce((acc, error) => {
      const severity = error.severity || 'unknown';
      acc[severity] = (acc[severity] || 0) + 1;
      return acc;
    }, {} as Record<string, number>)
  ).map(([severity, count]) => {
    const label = severityLabels[severity] || severity || 'Unknown';
    const color = severityColors[severity] || chartColors.gray.main;

    return {
      name: label, // Use 'name' for better Recharts compatibility
      severity: label,
      count,
      color
    };
  }).filter(item => item.count > 0); // Filter out zero-count items

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
          <ChartContainer>
            <div className="h-64">
              <ResponsiveContainer width="100%" height="100%">
                <PieChart>
                  <Pie
                    data={categoryData}
                    cx="50%"
                    cy="50%"
                    outerRadius={80}
                    dataKey="frequency"
                    label={({ name, frequency }) => `${name}: ${frequency}`}
                    labelLine={{ stroke: 'currentColor', className: 'text-gray-400 dark:text-gray-500' }}
                  >
                    {categoryData.map((entry, index) => (
                      <Cell key={`cell-${index}`} fill={entry.color} />
                    ))}
                  </Pie>
                  <Tooltip content={<CustomTooltip />} />
                  <Legend
                    verticalAlign="bottom"
                    height={36}
                    iconType="circle"
                    wrapperStyle={{ fontSize: '12px' }}
                  />
                </PieChart>
              </ResponsiveContainer>
            </div>
          </ChartContainer>
        </div>

        <div>
          <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-4">
            Error Severity Distribution
          </h3>
          <ChartContainer>
            <div className="h-64">
              <ResponsiveContainer width="100%" height="100%">
                <BarChart data={severityData}>
                  <CartesianGrid {...chartGridProps} />
                  <XAxis dataKey="severity" {...chartAxisProps} />
                  <YAxis {...chartAxisProps} />
                  <Tooltip content={<CustomTooltip />} />
                  <Bar dataKey="count" radius={[8, 8, 0, 0]}>
                    {severityData.map((entry, index) => (
                      <Cell key={`cell-${index}`} fill={entry.color} />
                    ))}
                  </Bar>
                </BarChart>
              </ResponsiveContainer>
            </div>
          </ChartContainer>
        </div>
      </div>

      <div>
        <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-4">
          Top Errors by Frequency
        </h3>
        <div className="space-y-3">
          {topErrors.map((error, index) => {
            const categoryKey = (error.category || 'unknown') as ErrorCategory | 'unknown';
            const categoryLabel = categoryLabels[categoryKey] || 'Unknown';
            const categoryColor = categoryColors[categoryKey] || categoryColors.unknown;
            const severityLabel = severityLabels[error.severity] || error.severity || 'Unknown';
            const severityColor = severityColors[error.severity] || chartColors.gray.main;

            return (
              <div key={index} className="border border-gray-200 dark:border-gray-700 rounded-xl p-4 hover:border-gray-300 dark:hover:border-gray-600 transition-colors">
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center space-x-2 mb-2">
                      <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium"
                            style={{ backgroundColor: `${categoryColor}20`, color: categoryColor }}>
                        {categoryLabel}
                      </span>
                      <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium"
                            style={{ backgroundColor: `${severityColor}20`, color: severityColor }}>
                        {severityLabel}
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
            );
          })}
        </div>
      </div>
    </div>
  );
}