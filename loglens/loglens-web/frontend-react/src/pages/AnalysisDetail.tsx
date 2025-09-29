import { useParams } from 'react-router-dom';
import { ErrorBoundary } from 'react-error-boundary';
import { useAnalysisDetail, useAnalysisResult } from '@/hooks/useAnalysisDetail';
import { AnalysisHeader } from '@/components/AnalysisHeader';
import { ExecutiveSummary } from '@/components/ExecutiveSummary';
import { ErrorAnalysisDashboard } from '@/components/ErrorAnalysisDashboard';
import { PatternDetection } from '@/components/PatternDetection';
import { PerformanceMetrics } from '@/components/PerformanceMetrics';
import { AnomalyDetection } from '@/components/AnomalyDetection';
import { Recommendations } from '@/components/Recommendations';
import { ExportOptions } from '@/components/ExportOptions';
import { AnalysisErrorBoundary } from '@/components/AnalysisErrorBoundary';

function AnalysisDetailContent() {
  const { id } = useParams<{ id: string }>();
  const { data: analysis, isLoading, error, isError, isSuccess } = useAnalysisDetail({ 
    id: id || '', 
    enabled: !!id 
  });
  const analysisResult = useAnalysisResult(analysis);

  // Debug information
  console.log('Analysis Detail Debug:', {
    id,
    isLoading,
    isError,
    isSuccess,
    error,
    analysis,
    analysisResult
  });

  if (isLoading) {
    return (
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="animate-pulse">
          <div className="h-12 bg-gray-200 dark:bg-gray-700 rounded mb-6"></div>
          <div className="space-y-6">
            <div className="h-64 bg-gray-200 dark:bg-gray-700 rounded"></div>
            <div className="h-64 bg-gray-200 dark:bg-gray-700 rounded"></div>
            <div className="h-64 bg-gray-200 dark:bg-gray-700 rounded"></div>
          </div>
        </div>
      </div>
    );
  }

  if (isError) {
    return (
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6">
          <div className="text-center">
            <div className="mx-auto h-12 w-12 bg-red-100 dark:bg-red-900/20 rounded-full flex items-center justify-center mb-4">
              <svg className="h-6 w-6 text-red-600 dark:text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
            </div>
            <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
              Analysis Not Found
            </h2>
            <p className="text-gray-600 dark:text-gray-400 mb-4">
              The analysis you're looking for doesn't exist or you don't have permission to view it.
            </p>
            {error && (
              <div className="mb-4 p-3 bg-red-50 dark:bg-red-900/20 rounded-md">
                <p className="text-sm text-red-600 dark:text-red-400">
                  Error: {error.message}
                </p>
              </div>
            )}
            <button
              onClick={() => window.history.back()}
              className="inline-flex items-center px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-lg hover:bg-blue-700 transition-colors"
            >
              Go Back
            </button>
          </div>
        </div>
      </div>
    );
  }

  if (!analysis) {
    return (
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6">
          <div className="text-center">
            <p className="text-gray-500 dark:text-gray-400 mb-4">
              No analysis data available.
            </p>
            <div className="bg-gray-50 dark:bg-gray-900/20 rounded-md p-4 text-left">
              <p className="text-sm text-gray-600 dark:text-gray-400 mb-2">
                Debug Information:
              </p>
              <ul className="text-xs text-gray-500 dark:text-gray-400 space-y-1">
                <li>• Analysis ID: {id}</li>
                <li>• Loading: {isLoading ? 'true' : 'false'}</li>
                <li>• Has Error: {isError ? 'true' : 'false'}</li>
                <li>• Has Data: {analysis ? 'true' : 'false'}</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    );
  }

  // Check if analysis has meaningful results
  console.log('hasMeaningfulResults Debug:', {
    analysisResult,
    hasSequenceOfEvents: !!(analysisResult?.sequence_of_events && analysisResult.sequence_of_events !== "No log entries found matching the specified level."),
    hasRelatedErrors: !!(analysisResult?.related_errors && analysisResult.related_errors.length > 0),
    hasUnrelatedErrors: !!(analysisResult?.unrelated_errors && analysisResult.unrelated_errors.length > 0),
    hasConfidence: !!(analysisResult?.confidence && analysisResult.confidence > 0),
    hasPatterns: !!(analysisResult?.patterns && analysisResult.patterns.length > 0)
  });
  
  const hasMeaningfulResults = analysisResult && (
    (analysisResult.sequence_of_events && analysisResult.sequence_of_events !== "No log entries found matching the specified level.") ||
    (analysisResult.related_errors && Array.isArray(analysisResult.related_errors) && analysisResult.related_errors.length > 0) ||
    (analysisResult.unrelated_errors && Array.isArray(analysisResult.unrelated_errors) && analysisResult.unrelated_errors.length > 0) ||
    (analysisResult.root_cause?.confidence && analysisResult.root_cause.confidence > 0) ||
    (analysisResult.recommendations && Array.isArray(analysisResult.recommendations) && analysisResult.recommendations.length > 0)
  );

  console.log('hasMeaningfulResults result:', hasMeaningfulResults);

  if (!hasMeaningfulResults) {
    return (
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6">
          <div className="text-center">
            <div className="mx-auto h-12 w-12 bg-yellow-100 dark:bg-yellow-900/20 rounded-full flex items-center justify-center mb-4">
              <svg className="h-6 w-6 text-yellow-600 dark:text-yellow-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
            </div>
            <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
              No Analysis Results Found
            </h2>
            <p className="text-gray-600 dark:text-gray-400 mb-4">
              This analysis didn't find any log entries matching the selected filter level.
            </p>
            <div className="bg-blue-50 dark:bg-blue-900/20 rounded-md p-4 text-left mb-6">
              <h3 className="text-sm font-medium text-blue-800 dark:text-blue-200 mb-2">
                Analysis Details:
              </h3>
              <ul className="text-sm text-blue-700 dark:text-blue-300 space-y-1">
                <li>• Provider: {analysis.ai_provider}</li>
                <li>• Status: {analysis.status}</li>
                <li>• Created: {new Date(analysis.created_at).toLocaleString()}</li>
                <li>• Updated: {new Date(analysis.updated_at).toLocaleString()}</li>
                {analysis.error && (
                  <li>• Error: {analysis.error}</li>
                )}
              </ul>
            </div>
            <div className="space-y-3">
              <button
                onClick={() => window.history.back()}
                className="inline-flex items-center px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-lg hover:bg-blue-700 transition-colors"
              >
                Go Back to Project
              </button>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Tip: Try analyzing with a different level filter (like "INFO" or "All") or upload a log file with more entries.
              </p>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-6">
      <AnalysisHeader analysis={analysis} />
      
      <ExecutiveSummary 
        result={analysisResult} 
        loading={isLoading} 
      />
      
      <ErrorAnalysisDashboard 
        errors={[...(analysisResult?.related_errors || []), ...(analysisResult?.unrelated_errors || [])]} 
        loading={isLoading} 
      />
      
      <PatternDetection 
        patterns={[]} 
        loading={isLoading} 
      />
      
      <PerformanceMetrics 
        performance={null} 
        loading={isLoading} 
      />
      
      <AnomalyDetection 
        anomalies={[]} 
        loading={isLoading} 
      />
      
      <Recommendations 
        recommendations={analysisResult?.recommendations || []} 
        loading={isLoading} 
      />
      
      <ExportOptions 
        analysis={analysis} 
        loading={isLoading} 
      />
    </div>
  );
}

export default function AnalysisDetail() {
  return (
    <AnalysisErrorBoundary>
      <ErrorBoundary fallback={null}>
        <AnalysisDetailContent />
      </ErrorBoundary>
    </AnalysisErrorBoundary>
  );
}