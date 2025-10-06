import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { ChartBarIcon } from '@heroicons/react/24/outline';
import { useWebSocketAnalysis } from '@/hooks/useWebSocketAnalysis';
import AnalysisProgress from '@/components/AnalysisProgress';
import { useSettings } from '@/hooks/useSettings';

interface WebSocketAnalysisPanelProps {
  projectId: string;
  fileId: string;
  fileName: string;
  onComplete?: () => void;
}

export const WebSocketAnalysisPanel: React.FC<WebSocketAnalysisPanelProps> = ({
  projectId,
  fileId,
  fileName,
  onComplete,
}) => {
  const navigate = useNavigate();
  const { getProvider, getLevel, isConfigured } = useSettings();
  const [userContext, setUserContext] = useState('');
  const [showOptions, setShowOptions] = useState(false);

  const {
    isConnected,
    isAnalyzing,
    progress,
    result,
    error,
    connect,
    cancel,
  } = useWebSocketAnalysis({
    projectId,
    fileId,
    provider: getProvider(),
    level: getLevel(),
    userContext: userContext || undefined,
    onProgress: (prog) => {
      console.log('Analysis progress:', prog);
    },
    onComplete: (res) => {
      console.log('Analysis complete:', res);
      onComplete?.();
      // Navigate to the completed analysis
      setTimeout(() => {
        navigate(`/projects/${projectId}/analyses/${res.analysis_id}`);
      }, 2000);
    },
    onError: (err) => {
      console.error('Analysis error:', err);
    },
    onCancel: (reason) => {
      console.log('Analysis cancelled:', reason);
    },
  });

  const handleStartAnalysis = () => {
    if (!isConfigured()) {
      alert(`Please configure your AI provider (${getProvider()}) in Settings before starting analysis.`);
      return;
    }
    connect();
  };

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium text-gray-900 dark:text-white">
            WebSocket Analysis
          </h3>
          <p className="text-sm text-gray-500 dark:text-gray-400">
            Real-time progress tracking for {fileName}
          </p>
        </div>

        {!isAnalyzing && !result && (
          <button
            onClick={() => setShowOptions(!showOptions)}
            className="text-sm text-primary-600 hover:text-primary-700 dark:text-primary-400 dark:hover:text-primary-300"
          >
            {showOptions ? 'Hide Options' : 'Show Options'}
          </button>
        )}
      </div>

      {/* Options Panel */}
      {showOptions && !isAnalyzing && !result && (
        <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 space-y-3">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              User Context (optional)
            </label>
            <textarea
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-primary-500 focus:border-primary-500 dark:bg-gray-800 dark:text-white text-sm"
              placeholder="Provide additional context for analysis..."
              value={userContext}
              onChange={(e) => setUserContext(e.target.value.slice(0, 2000))}
              rows={3}
            />
            <div className="flex justify-between mt-1">
              <span className="text-xs text-gray-500 dark:text-gray-400">
                Add context about your application or environment
              </span>
              <span className="text-xs text-gray-500 dark:text-gray-400">
                {userContext.length}/2000
              </span>
            </div>
          </div>
        </div>
      )}

      {/* Start Analysis Button */}
      {!isAnalyzing && !result && (
        <button
          onClick={handleStartAnalysis}
          disabled={!isConfigured()}
          className="w-full btn-primary flex items-center justify-center"
          title={!isConfigured() ? `Configure ${getProvider()} provider in Settings` : undefined}
        >
          <ChartBarIcon className="h-5 w-5 mr-2" />
          Start Analysis with Real-time Progress
        </button>
      )}

      {/* Connection Status */}
      {isConnected && !isAnalyzing && !result && (
        <div className="text-sm text-green-600 dark:text-green-400 flex items-center">
          <span className="inline-block w-2 h-2 bg-green-500 rounded-full mr-2 animate-pulse" />
          Connected - Starting analysis...
        </div>
      )}

      {/* Progress Display */}
      {isAnalyzing && progress && (
        <AnalysisProgress
          progress={progress}
          stats={result?.stats}
          onCancel={cancel}
        />
      )}

      {/* Completion Message */}
      {result && (
        <div className="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-4">
          <div className="flex items-start">
            <div className="flex-shrink-0">
              <svg className="h-5 w-5 text-green-400" viewBox="0 0 20 20" fill="currentColor">
                <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
              </svg>
            </div>
            <div className="ml-3 flex-1">
              <h3 className="text-sm font-medium text-green-800 dark:text-green-200">
                Analysis Complete!
              </h3>
              <div className="mt-2 text-sm text-green-700 dark:text-green-300">
                <p>Analysis ID: {result.analysis_id}</p>
                <p>Time taken: {(result.elapsed_ms / 1000).toFixed(2)}s</p>
                <p className="mt-2">Redirecting to results...</p>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Error Display */}
      {error && (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
          <div className="flex items-start">
            <div className="flex-shrink-0">
              <svg className="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
              </svg>
            </div>
            <div className="ml-3">
              <h3 className="text-sm font-medium text-red-800 dark:text-red-200">
                Analysis Failed
              </h3>
              <div className="mt-2 text-sm text-red-700 dark:text-red-300">
                <p>{error}</p>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default WebSocketAnalysisPanel;
