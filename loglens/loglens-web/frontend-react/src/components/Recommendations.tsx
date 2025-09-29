import { LightBulbIcon, CheckCircleIcon, ArrowRightIcon } from '@heroicons/react/24/outline';

interface RecommendationsProps {
  recommendations: string[];
  loading: boolean;
}

export function Recommendations({ recommendations, loading }: RecommendationsProps) {
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

  if (!recommendations || recommendations.length === 0) {
    return (
      <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6 mb-6">
        <div className="flex items-center space-x-2 mb-4">
          <LightBulbIcon className="h-5 w-5 text-gray-400" />
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            Recommendations
          </h2>
        </div>
        <p className="text-gray-500 dark:text-gray-400">
          No recommendations available for this analysis.
        </p>
      </div>
    );
  }

  const priorityRecommendations = recommendations.slice(0, 3);
  const additionalRecommendations = recommendations.slice(3);

  return (
    <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6 mb-6">
      <div className="flex items-center space-x-2 mb-6">
        <LightBulbIcon className="h-5 w-5 text-yellow-600" />
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
          Recommendations
        </h2>
        <span className="ml-2 bg-yellow-100 text-yellow-800 text-xs font-medium px-2.5 py-0.5 rounded-full">
          {recommendations.length} suggestions
        </span>
      </div>

      <div className="space-y-4">
        <div>
          <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-4 flex items-center">
            <CheckCircleIcon className="h-4 w-4 mr-2 text-green-600" />
            Priority Recommendations
          </h3>
          <div className="space-y-3">
            {priorityRecommendations.map((recommendation, index) => (
              <div key={index} className="border border-green-200 dark:border-green-700 rounded-lg p-4 bg-green-50 dark:bg-green-900/20">
                <div className="flex items-start space-x-3">
                  <div className="flex-shrink-0">
                    <div className="w-6 h-6 bg-green-100 dark:bg-green-900 rounded-full flex items-center justify-center">
                      <span className="text-xs font-bold text-green-600 dark:text-green-300">
                        {index + 1}
                      </span>
                    </div>
                  </div>
                  <div className="flex-1">
                    <p className="text-sm text-green-800 dark:text-green-200 leading-relaxed">
                      {recommendation}
                    </p>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {additionalRecommendations.length > 0 && (
          <div>
            <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-4 flex items-center">
              <ArrowRightIcon className="h-4 w-4 mr-2 text-blue-600" />
              Additional Recommendations
            </h3>
            <div className="space-y-2">
              {additionalRecommendations.map((recommendation, index) => (
                <div key={index} className="border border-gray-200 dark:border-gray-700 rounded-lg p-3">
                  <div className="flex items-start space-x-2">
                    <div className="flex-shrink-0 w-2 h-2 bg-blue-600 rounded-full mt-2"></div>
                    <p className="text-sm text-gray-700 dark:text-gray-300 leading-relaxed">
                      {recommendation}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-700 rounded-lg p-4">
          <div className="flex items-center space-x-2 mb-2">
            <LightBulbIcon className="h-4 w-4 text-blue-600" />
            <h4 className="text-sm font-semibold text-blue-900 dark:text-blue-100">
              Implementation Tips
            </h4>
          </div>
          <ul className="text-sm text-blue-800 dark:text-blue-200 space-y-1">
            <li>• Review recommendations in order of priority</li>
            <li>• Consider the context of your application when implementing fixes</li>
            <li>• Test changes in a development environment before production deployment</li>
            <li>• Monitor the impact of implemented recommendations</li>
          </ul>
        </div>
      </div>
    </div>
  );
}