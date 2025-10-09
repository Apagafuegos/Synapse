import { format } from 'date-fns';
import { 
  ClockIcon, 
  CheckCircleIcon, 
  XCircleIcon, 
  PlayIcon,
  ExclamationTriangleIcon
} from '@heroicons/react/24/outline';
import { AnalysisStatus } from '@/types';
import { cn } from '@/utils';

interface AnalysisHeaderProps {
  analysis: {
    id: string;
    status: AnalysisStatus;
    ai_provider: string;
    created_at: string;
    updated_at: string;
    file_id?: string;
  };
}

const statusConfig = {
  pending: {
    icon: ClockIcon,
    color: 'text-yellow-600 bg-yellow-100 dark:text-yellow-400 dark:bg-yellow-900',
    label: 'Pending'
  },
  running: {
    icon: PlayIcon,
    color: 'text-blue-600 bg-blue-100 dark:text-blue-400 dark:bg-blue-900',
    label: 'Running'
  },
  completed: {
    icon: CheckCircleIcon,
    color: 'text-green-600 bg-green-100 dark:text-green-400 dark:bg-green-900',
    label: 'Completed'
  },
  failed: {
    icon: XCircleIcon,
    color: 'text-red-600 bg-red-100 dark:text-red-400 dark:bg-red-900',
    label: 'Failed'
  },
  cancelled: {
    icon: ExclamationTriangleIcon,
    color: 'text-gray-600 bg-gray-100 dark:text-gray-400 dark:bg-gray-900',
    label: 'Cancelled'
  }
};

const providerIcons = {
  openrouter: 'AI',
  openai: 'AI',
  claude: 'AI',
  gemini: 'AI'
};

export function AnalysisHeader({ analysis }: AnalysisHeaderProps) {
  const statusConfigData = statusConfig[analysis.status];
  const StatusIcon = statusConfigData.icon;
  const providerIcon = providerIcons[analysis.ai_provider.toLowerCase() as keyof typeof providerIcons] || 'AI';

  return (
    <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6 mb-6">
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <div className="flex items-center space-x-3 mb-4">
            <div className="text-2xl">{providerIcon}</div>
            <div>
              <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
                Analysis #{analysis.id}
              </h1>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                {analysis.ai_provider} Provider
              </p>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mt-6">
            <div className="flex items-center space-x-2">
              <StatusIcon className="h-5 w-5" />
              <div>
                <p className="text-sm font-medium text-gray-900 dark:text-gray-100">Status</p>
                <span className={cn(
                  'inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium',
                  statusConfigData.color
                )}>
                  {statusConfigData.label}
                </span>
              </div>
            </div>

            <div className="flex items-center space-x-2">
              <ClockIcon className="h-5 w-5 text-gray-400" />
              <div>
                <p className="text-sm font-medium text-gray-900 dark:text-gray-100">Started</p>
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  {format(new Date(analysis.created_at), 'MMM d, yyyy HH:mm')}
                </p>
              </div>
            </div>

            <div className="flex items-center space-x-2">
              <ClockIcon className="h-5 w-5 text-gray-400" />
              <div>
                <p className="text-sm font-medium text-gray-900 dark:text-gray-100">Last Updated</p>
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  {format(new Date(analysis.updated_at), 'MMM d, yyyy HH:mm')}
                </p>
              </div>
            </div>
          </div>
        </div>

        {analysis.file_id && (
          <div className="ml-6">
            <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
              <p className="text-xs text-gray-500 dark:text-gray-400 mb-1">File ID</p>
              <p className="text-sm font-mono text-gray-900 dark:text-gray-100">
                {analysis.file_id}
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}