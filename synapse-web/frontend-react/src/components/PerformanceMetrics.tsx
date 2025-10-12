import { RadarChart, PolarGrid, PolarAngleAxis, PolarRadiusAxis, Radar, ResponsiveContainer, Tooltip } from 'recharts';
import { ClockIcon } from '@heroicons/react/24/outline';
import { PerformanceAnalysis } from '@/types';

interface PerformanceMetricsProps {
  performance: PerformanceAnalysis | null;
  loading: boolean;
}

export function PerformanceMetrics({ performance, loading }: PerformanceMetricsProps) {
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

  if (!performance) {
    return (
      <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6 mb-6">
        <div className="flex items-center space-x-2 mb-4">
          <ClockIcon className="h-5 w-5 text-gray-400" />
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            Performance Metrics
          </h2>
        </div>
        <p className="text-gray-500 dark:text-gray-400">
          No performance metrics available for this analysis.
        </p>
      </div>
    );
  }

  // Prepare metrics data for radar chart
  const metricsData = Object.entries(performance.metrics).map(([key, value]) => ({
    metric: key.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase()),
    value: value,
    fullMark: 100
  }));

  const formatProcessingTime = (time: number) => {
    if (time < 1000) {
      return `${time.toFixed(2)}ms`;
    } else if (time < 60000) {
      return `${(time / 1000).toFixed(2)}s`;
    } else {
      return `${(time / 60000).toFixed(2)}min`;
    }
  };

  return (
    <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6 mb-6">
      <div className="flex items-center space-x-2 mb-6">
        <ClockIcon className="h-5 w-5 text-green-600" />
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
          Performance Metrics
        </h2>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        <div>
          <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-4">
            Processing Time
          </h3>
          <div className="bg-gradient-to-r from-green-50 to-blue-50 dark:from-green-900/20 dark:to-blue-900/20 rounded-lg p-6">
            <div className="flex items-center space-x-3">
              <ClockIcon className="h-8 w-8 text-green-600" />
              <div>
                <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                  {formatProcessingTime(performance.total_processing_time)}
                </p>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  Total Processing Time
                </p>
              </div>
            </div>
          </div>
        </div>

        <div>
          <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-4">
            Performance Score
          </h3>
          <div className="h-40">
            <ResponsiveContainer width="100%" height="100%">
              <RadarChart cx="50%" cy="50%" outerRadius="80%" data={metricsData}>
                <PolarGrid />
                <PolarAngleAxis dataKey="metric" />
                <PolarRadiusAxis angle={90} domain={[0, 100]} />
                <Radar
                  name="Performance"
                  dataKey="value"
                  stroke="#10b981"
                  fill="#10b981"
                  fillOpacity={0.3}
                />
                <Tooltip />
              </RadarChart>
            </ResponsiveContainer>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        <div>
          <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-4">
            Performance Bottlenecks
          </h3>
          <div className="space-y-3">
            {performance.bottlenecks.map((bottleneck, index) => (
              <div key={index} className="flex items-start space-x-3 p-3 bg-red-50 dark:bg-red-900/20 rounded-lg">
                <div className="flex-shrink-0">
                  <div className="w-6 h-6 bg-red-100 dark:bg-red-900 rounded-full flex items-center justify-center">
                    <span className="text-xs font-bold text-red-600 dark:text-red-300">
                      {index + 1}
                    </span>
                  </div>
                </div>
                <div className="flex-1">
                  <p className="text-sm font-medium text-red-800 dark:text-red-200">
                    Bottleneck Detected
                  </p>
                  <p className="text-sm text-red-600 dark:text-red-300">
                    {bottleneck}
                  </p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      <div>
        <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-4">
          Performance Recommendations
        </h3>
        <div className="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-4">
          <ul className="space-y-2">
            {performance.recommendations.map((recommendation, index) => (
              <li key={index} className="flex items-start space-x-2">
                <div className="flex-shrink-0 w-2 h-2 bg-blue-600 rounded-full mt-2"></div>
                <p className="text-sm text-blue-800 dark:text-blue-200">
                  {recommendation}
                </p>
              </li>
            ))}
          </ul>
        </div>
      </div>
    </div>
  );
}