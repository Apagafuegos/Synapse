import { 
  ChartBarIcon,
  LightBulbIcon,
  ClockIcon,
  ArrowTrendingUpIcon
} from '@heroicons/react/24/outline';
import { AnalysisResult } from '@/types';

interface ExecutiveSummaryProps {
  result: AnalysisResult | null;
  loading: boolean;
}

export function ExecutiveSummary({ result, loading }: ExecutiveSummaryProps) {
  if (loading) {
    return (
      <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6 mb-6">
        <div className="animate-pulse">
          <div className="h-8 bg-gray-200 dark:bg-gray-700 rounded mb-4"></div>
          <div className="space-y-3">
            <div className="h-4 bg-gray-200 dark:bg-gray-700 rounded"></div>
            <div className="h-4 bg-gray-200 dark:bg-gray-700 rounded w-3/4"></div>
            <div className="h-4 bg-gray-200 dark:bg-gray-700 rounded w-1/2"></div>
          </div>
        </div>
      </div>
    );
  }

  if (!result) {
    return (
      <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6 mb-6">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          Executive Summary
        </h2>
        <p className="text-gray-500 dark:text-gray-400">
          No analysis results available yet. The analysis may still be processing.
        </p>
      </div>
    );
  }

  return (
    <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6 mb-6">
      <div className="flex items-center space-x-2 mb-4">
        <ChartBarIcon className="h-5 w-5 text-blue-600" />
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
          Executive Summary
        </h2>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
        <div className="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-4">
          <div className="flex items-center space-x-2">
            <ChartBarIcon className="h-4 w-4 text-blue-600" />
            <span className="text-sm font-medium text-blue-900 dark:text-blue-100">
              Confidence
            </span>
          </div>
          <p className="text-2xl font-bold text-blue-900 dark:text-blue-100 mt-1">
            {((result.root_cause?.confidence || 0) * 100).toFixed(1)}%
          </p>
        </div>

        <div className="bg-green-50 dark:bg-green-900/20 rounded-lg p-4">
          <div className="flex items-center space-x-2">
            <LightBulbIcon className="h-4 w-4 text-green-600" />
            <span className="text-sm font-medium text-green-900 dark:text-green-100">
              Recommendations
            </span>
          </div>
          <p className="text-2xl font-bold text-green-900 dark:text-green-100 mt-1">
            {result.recommendations?.length || 0}
          </p>
        </div>

        <div className="bg-purple-50 dark:bg-purple-900/20 rounded-lg p-4">
          <div className="flex items-center space-x-2">
            <ArrowTrendingUpIcon className="h-4 w-4 text-purple-600" />
            <span className="text-sm font-medium text-purple-900 dark:text-purple-100">
              Errors Found
            </span>
          </div>
          <p className="text-2xl font-bold text-purple-900 dark:text-purple-100 mt-1">
            {((result.related_errors?.length || 0) + (result.unrelated_errors?.length || 0))}
          </p>
        </div>

        <div className="bg-orange-50 dark:bg-orange-900/20 rounded-lg p-4">
          <div className="flex items-center space-x-2">
            <ClockIcon className="h-4 w-4 text-orange-600" />
            <span className="text-sm font-medium text-orange-900 dark:text-orange-100">
              Patterns
            </span>
          </div>
          <p className="text-2xl font-bold text-orange-900 dark:text-orange-100 mt-1">
            {0}
          </p>
        </div>
      </div>

      <div className="space-y-4">
        <div>
          <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-2">
            Analysis Overview
          </h3>
          <p className="text-gray-700 dark:text-gray-300 text-sm leading-relaxed">
            {result.sequence_of_events || "No analysis summary available."}
          </p>
        </div>

        <div>
          <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-2">
            Key Findings
          </h3>
          <ul className="space-y-2">
            {(result.recommendations || []).slice(0, 3).map((recommendation, index) => (
              <li key={index} className="flex items-start space-x-2">
                <div className="flex-shrink-0 w-2 h-2 bg-blue-600 rounded-full mt-2"></div>
                <p className="text-sm text-gray-700 dark:text-gray-300">
                  {recommendation}
                </p>
              </li>
            ))}
            {(result.recommendations?.length || 0) > 3 && (
              <li className="text-sm text-gray-500 dark:text-gray-400 italic">
                +{(result.recommendations?.length || 0) - 3} more recommendations available
              </li>
            )}
          </ul>
        </div>
      </div>
    </div>
  );
}