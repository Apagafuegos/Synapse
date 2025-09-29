import { ScatterChart, Scatter, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Cell } from 'recharts';
import { ExclamationCircleIcon } from '@heroicons/react/24/outline';
import { AnomalyAnalysis } from '@/types';

interface AnomalyDetectionProps {
  anomalies: AnomalyAnalysis[];
  loading: boolean;
}

const anomalyTypeColors = {
  timing: '#ef4444',
  frequency: '#f59e0b',
  pattern: '#8b5cf6',
  threshold: '#06b6d4',
  unknown: '#6b7280'
};

const anomalyTypeIcons = {
  timing: ClockIcon,
  frequency: ArrowsRightLeftIcon,
  pattern: ArrowsRightLeftIcon,
  threshold: ExclamationTriangleIcon,
  unknown: InformationCircleIcon
};

function ClockIcon(props: any) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
    </svg>
  );
}

function ArrowsRightLeftIcon(props: any) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4" />
    </svg>
  );
}

function ExclamationTriangleIcon(props: any) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
    </svg>
  );
}

function InformationCircleIcon(props: any) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
    </svg>
  );
}

export function AnomalyDetection({ anomalies, loading }: AnomalyDetectionProps) {
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

  if (!anomalies || anomalies.length === 0) {
    return (
      <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6 mb-6">
        <div className="flex items-center space-x-2 mb-4">
          <ExclamationCircleIcon className="h-5 w-5 text-gray-400" />
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            Anomaly Detection
          </h2>
        </div>
        <p className="text-gray-500 dark:text-gray-400">
          No anomalies detected in the analyzed logs.
        </p>
      </div>
    );
  }

  // Prepare scatter plot data
  const scatterData = anomalies.map((anomaly, index) => ({
    x: anomaly.line_numbers[0] || index,
    y: anomaly.confidence * 100,
    anomaly: anomaly,
    size: Math.max(10, anomaly.confidence * 30)
  }));

  // Group anomalies by type
  const anomaliesByType = anomalies.reduce((acc, anomaly) => {
    if (!acc[anomaly.anomaly_type]) {
      acc[anomaly.anomaly_type] = [];
    }
    acc[anomaly.anomaly_type].push(anomaly);
    return acc;
  }, {} as Record<string, AnomalyAnalysis[]>);

  return (
    <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6 mb-6">
      <div className="flex items-center space-x-2 mb-6">
        <ExclamationCircleIcon className="h-5 w-5 text-orange-600" />
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
          Anomaly Detection
        </h2>
        <span className="ml-2 bg-orange-100 text-orange-800 text-xs font-medium px-2.5 py-0.5 rounded-full">
          {anomalies.length} anomalies
        </span>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        <div>
          <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-4">
            Anomaly Confidence Scatter Plot
          </h3>
          <div className="h-64">
            <ResponsiveContainer width="100%" height="100%">
              <ScatterChart margin={{ top: 20, right: 20, bottom: 20, left: 20 }}>
                <CartesianGrid />
                <XAxis 
                  dataKey="x" 
                  label={{ value: 'Line Number', position: 'insideBottom', offset: -5 }} 
                />
                <YAxis 
                  label={{ value: 'Confidence (%)', angle: -90, position: 'insideLeft' }}
                  domain={[0, 100]}
                />
                <Tooltip 
                  formatter={(value) => [`${value}%`, 'Confidence']}
                  labelFormatter={(label, payload) => {
                    if (payload && payload[0]) {
                      const anomaly = payload[0].payload.anomaly;
                      return `Line ${label}: ${anomaly.description}`;
                    }
                    return `Line ${label}`;
                  }}
                />
                <Scatter data={scatterData} fill="#f59e0b">
                  {scatterData.map((entry, index) => (
                    <Cell 
                      key={`cell-${index}`} 
                      fill={anomalyTypeColors[entry.anomaly.anomaly_type as keyof typeof anomalyTypeColors] || '#6b7280'} 
                    />
                  ))}
                </Scatter>
              </ScatterChart>
            </ResponsiveContainer>
          </div>
        </div>

        <div>
          <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-4">
            Anomaly Types
          </h3>
          <div className="space-y-3">
            {Object.entries(anomaliesByType).map(([type, typeAnomalies]) => {
              const color = anomalyTypeColors[type as keyof typeof anomalyTypeColors] || '#6b7280';
              const Icon = anomalyTypeIcons[type as keyof typeof anomalyTypeIcons] || InformationCircleIcon;
              
              return (
                <div key={type} className="border border-gray-200 dark:border-gray-700 rounded-lg p-4">
                  <div className="flex items-center space-x-2 mb-2">
                    <Icon className="h-4 w-4" style={{ color }} />
                    <span className="text-sm font-medium text-gray-900 dark:text-gray-100">
                      {type.charAt(0).toUpperCase() + type.slice(1)} Anomalies
                    </span>
                    <span className="text-xs text-gray-500 dark:text-gray-400">
                      ({typeAnomalies.length})
                    </span>
                  </div>
                  <div className="space-y-2">
                    {typeAnomalies.slice(0, 2).map((anomaly, index) => (
                      <div key={index} className="flex items-start space-x-2">
                        <div 
                          className="w-2 h-2 rounded-full mt-2" 
                          style={{ backgroundColor: color }}
                        ></div>
                        <div className="flex-1">
                          <p className="text-xs text-gray-700 dark:text-gray-300">
                            {anomaly.description}
                          </p>
                          <p className="text-xs text-gray-500 dark:text-gray-400">
                            {anomaly.line_numbers.length} lines • {(anomaly.confidence * 100).toFixed(1)}% confidence
                          </p>
                        </div>
                      </div>
                    ))}
                    {typeAnomalies.length > 2 && (
                      <p className="text-xs text-gray-500 dark:text-gray-400 italic">
                        +{typeAnomalies.length - 2} more {type} anomalies
                      </p>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </div>

      <div>
        <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-4">
          All Anomalies
        </h3>
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
            <thead className="bg-gray-50 dark:bg-gray-700">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                  Type
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                  Description
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                  Line Numbers
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                  Confidence
                </th>
              </tr>
            </thead>
            <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
              {anomalies.map((anomaly, index) => {
                const color = anomalyTypeColors[anomaly.anomaly_type as keyof typeof anomalyTypeColors] || '#6b7280';
                const Icon = anomalyTypeIcons[anomaly.anomaly_type as keyof typeof anomalyTypeIcons] || InformationCircleIcon;
                
                return (
                  <tr key={index}>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center space-x-2">
                        <Icon className="h-4 w-4" style={{ color }} />
                        <span className="text-sm text-gray-900 dark:text-gray-100">
                          {anomaly.anomaly_type}
                        </span>
                      </div>
                    </td>
                    <td className="px-6 py-4 text-sm text-gray-900 dark:text-gray-100">
                      {anomaly.description}
                    </td>
                    <td className="px-6 py-4 text-sm text-gray-500 dark:text-gray-400">
                      {anomaly.line_numbers.join(', ')}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center space-x-2">
                        <div className="w-16 bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                          <div 
                            className="h-2 rounded-full" 
                            style={{ 
                              width: `${anomaly.confidence * 100}%`, 
                              backgroundColor: color 
                            }}
                          ></div>
                        </div>
                        <span className="text-sm text-gray-600 dark:text-gray-400">
                          {(anomaly.confidence * 100).toFixed(1)}%
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