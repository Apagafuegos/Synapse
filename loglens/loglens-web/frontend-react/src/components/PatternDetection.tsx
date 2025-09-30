import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import { ArrowsRightLeftIcon, ArrowTrendingUpIcon, ArrowTrendingDownIcon, MinusIcon } from '@heroicons/react/24/outline';
import { PatternAnalysis } from '@/types';
import { CustomTooltip, ChartContainer, chartColors, chartGridProps, chartAxisProps } from './ChartComponents';

interface PatternDetectionProps {
  patterns: PatternAnalysis[];
  loading: boolean;
}

const trendIcons = {
  increasing: ArrowTrendingUpIcon,
  decreasing: ArrowTrendingDownIcon,
  stable: MinusIcon
};

const trendColors = {
  increasing: chartColors.error.main,
  decreasing: chartColors.success.main,
  stable: chartColors.gray.main
};

export function PatternDetection({ patterns, loading }: PatternDetectionProps) {
  if (loading) {
    return (
      <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6 mb-6">
        <div className="animate-pulse">
          <div className="h-8 bg-gray-200 dark:bg-gray-700 rounded mb-4"></div>
          <div className="h-64 bg-gray-200 dark:bg-gray-700 rounded"></div>
        </div>
      </div>
    );
  }

  if (!patterns || patterns.length === 0) {
    return (
      <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6 mb-6">
        <div className="flex items-center space-x-2 mb-4">
          <ArrowsRightLeftIcon className="h-5 w-5 text-gray-400" />
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            Pattern Detection
          </h2>
        </div>
        <p className="text-gray-500 dark:text-gray-400">
          No significant patterns detected in the analyzed logs.
        </p>
      </div>
    );
  }

  // Prepare frequency timeline data
  const timelineData = patterns
    .sort((a, b) => a.first_occurrence - b.first_occurrence)
    .map((pattern) => ({
      pattern: pattern.pattern.substring(0, 30) + (pattern.pattern.length > 30 ? '...' : ''),
      frequency: pattern.frequency,
      trend: pattern.trend,
      firstOccurrence: pattern.first_occurrence
    }));

  return (
    <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6 mb-6">
      <div className="flex items-center space-x-2 mb-6">
        <ArrowsRightLeftIcon className="h-5 w-5 text-blue-600" />
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
          Pattern Detection
        </h2>
        <span className="ml-2 bg-blue-100 text-blue-800 text-xs font-medium px-2.5 py-0.5 rounded-full">
          {patterns.length} patterns
        </span>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        <div>
          <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-4">
            Pattern Frequency Overview
          </h3>
          <ChartContainer>
            <div className="h-64">
              <ResponsiveContainer width="100%" height="100%">
                <AreaChart data={timelineData}>
                  <CartesianGrid {...chartGridProps} />
                  <XAxis dataKey="pattern" {...chartAxisProps} />
                  <YAxis {...chartAxisProps} />
                  <Tooltip
                    content={<CustomTooltip />}
                    formatter={(value) => [value, 'Frequency']}
                    labelFormatter={(label) => `Pattern: ${label}`}
                  />
                  <Area
                    type="monotone"
                    dataKey="frequency"
                    stroke={chartColors.primary.main}
                    fill={chartColors.primary.main}
                    fillOpacity={0.3}
                    name="Frequency"
                  />
                </AreaChart>
              </ResponsiveContainer>
            </div>
          </ChartContainer>
        </div>

        <div>
          <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-4">
            Pattern Trends
          </h3>
          <div className="space-y-3">
            {patterns.slice(0, 5).map((pattern, index) => {
              const TrendIcon = trendIcons[pattern.trend];
              const trendColor = trendColors[pattern.trend];

              return (
                <div key={index} className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-900/50 rounded-xl border border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600 transition-colors">
                  <div className="flex items-center space-x-3">
                    <TrendIcon className="h-5 w-5 flex-shrink-0" style={{ color: trendColor }} />
                    <div>
                      <p className="text-sm font-medium text-gray-900 dark:text-gray-100">
                        {pattern.pattern.substring(0, 40)}{pattern.pattern.length > 40 ? '...' : ''}
                      </p>
                      <p className="text-xs text-gray-500 dark:text-gray-400">
                        Lines {pattern.first_occurrence} - {pattern.last_occurrence}
                      </p>
                    </div>
                  </div>
                  <div className="text-right flex-shrink-0">
                    <p className="text-sm font-bold text-gray-900 dark:text-gray-100">
                      {pattern.frequency}
                    </p>
                    <p className="text-xs capitalize" style={{ color: trendColor }}>
                      {pattern.trend}
                    </p>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </div>

      <div>
        <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-4">
          All Detected Patterns
        </h3>
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
            <thead className="bg-gray-50 dark:bg-gray-700">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                  Pattern
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                  Frequency
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                  First Occurrence
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                  Last Occurrence
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                  Trend
                </th>
              </tr>
            </thead>
            <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
              {patterns.map((pattern, index) => {
                const TrendIcon = trendIcons[pattern.trend];
                const trendColor = trendColors[pattern.trend];
                
                return (
                  <tr key={index}>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100">
                      {pattern.pattern}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100">
                      {pattern.frequency}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                      {pattern.first_occurrence}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                      {pattern.last_occurrence}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm">
                      <div className="flex items-center space-x-1">
                        <TrendIcon className="h-4 w-4" style={{ color: trendColor }} />
                        <span style={{ color: trendColor }}>
                          {pattern.trend}
                        </span>
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}