import React from 'react';
import { clsx } from 'clsx';
import { AnalysisProgress as Progress, AnalysisStats } from '@/hooks/useWebSocketAnalysis';

interface AnalysisProgressProps {
  progress: Progress;
  stats?: AnalysisStats;
  onCancel?: () => void;
  className?: string;
}

const stageLabels: Record<string, string> = {
  'starting': 'Initializing',
  'reading_file': 'Reading File',
  'parsing': 'Parsing Logs',
  'filtering': 'Filtering',
  'slimming': 'Optimizing',
  'ai_analysis': 'AI Analysis',
  'finalizing': 'Finalizing',
};

const getStageLabel = (stage: string): string => {
  return stageLabels[stage] || stage;
};

export const AnalysisProgress: React.FC<AnalysisProgressProps> = ({
  progress,
  stats,
  onCancel,
  className
}) => {
  const percentage = Math.round(progress.progress * 100);
  const elapsedSeconds = (progress.elapsed_ms / 1000).toFixed(1);

  return (
    <div className={clsx(
      'bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6 space-y-4',
      className
    )}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
          Analysis in Progress
        </h3>
        <span className="text-sm text-gray-500 dark:text-gray-400">
          {elapsedSeconds}s elapsed
        </span>
      </div>

      {/* Stage and Percentage */}
      <div className="flex items-center justify-between">
        <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
          {getStageLabel(progress.stage)}
        </span>
        <span className="text-sm font-bold text-primary-600 dark:text-primary-400">
          {percentage}%
        </span>
      </div>

      {/* Progress Bar */}
      <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3 overflow-hidden">
        <div
          className="bg-gradient-to-r from-primary-500 to-primary-600 h-3 rounded-full transition-all duration-300 ease-out"
          style={{ width: `${percentage}%` }}
          role="progressbar"
          aria-valuenow={percentage}
          aria-valuemin={0}
          aria-valuemax={100}
          aria-label={`Analysis progress: ${percentage}%`}
        />
      </div>

      {/* Progress Message */}
      {progress.message && (
        <p className="text-sm text-gray-600 dark:text-gray-400 italic">
          {progress.message}
        </p>
      )}

      {/* Stats Grid */}
      {stats && (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 pt-4 border-t border-gray-200 dark:border-gray-700">
          <div className="text-center">
            <div className="text-xs text-gray-500 dark:text-gray-400 mb-1">
              Total Lines
            </div>
            <div className="text-lg font-semibold text-gray-900 dark:text-white">
              {stats.total_lines.toLocaleString()}
            </div>
          </div>
          <div className="text-center">
            <div className="text-xs text-gray-500 dark:text-gray-400 mb-1">
              Parsed
            </div>
            <div className="text-lg font-semibold text-gray-900 dark:text-white">
              {stats.parsed_entries.toLocaleString()}
            </div>
          </div>
          <div className="text-center">
            <div className="text-xs text-gray-500 dark:text-gray-400 mb-1">
              Filtered
            </div>
            <div className="text-lg font-semibold text-gray-900 dark:text-white">
              {stats.filtered_entries.toLocaleString()}
            </div>
          </div>
          <div className="text-center">
            <div className="text-xs text-gray-500 dark:text-gray-400 mb-1">
              AI Input
            </div>
            <div className="text-lg font-semibold text-gray-900 dark:text-white">
              {stats.slimmed_entries.toLocaleString()}
            </div>
          </div>
        </div>
      )}

      {/* Cancel Button */}
      {onCancel && (
        <div className="pt-4 flex justify-end">
          <button
            onClick={onCancel}
            className="px-4 py-2 bg-red-600 hover:bg-red-700 text-white font-medium rounded-lg transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-2"
            aria-label="Cancel analysis"
          >
            Cancel Analysis
          </button>
        </div>
      )}
    </div>
  );
};

export default AnalysisProgress;
