import { TooltipProps } from 'recharts';

// Theme-aware colors matching Tailwind config
export const chartColors = {
  primary: {
    main: '#3b82f6',
    light: '#60a5fa',
    dark: '#2563eb',
  },
  error: {
    main: '#ef4444',
    light: '#f87171',
    dark: '#dc2626',
  },
  warning: {
    main: '#f59e0b',
    light: '#fbbf24',
    dark: '#d97706',
  },
  success: {
    main: '#22c55e',
    light: '#4ade80',
    dark: '#16a34a',
  },
  gray: {
    main: '#6b7280',
    light: '#9ca3af',
    dark: '#4b5563',
  },
  blue: {
    main: '#3b82f6',
    light: '#60a5fa',
    dark: '#2563eb',
  },
  purple: {
    main: '#8b5cf6',
    light: '#a78bfa',
    dark: '#7c3aed',
  },
  cyan: {
    main: '#06b6d4',
    light: '#22d3ee',
    dark: '#0891b2',
  },
};

// Custom Tooltip with dark mode support
interface CustomTooltipProps extends TooltipProps<number, string> {
  labelFormatter?: (label: string) => string;
}

export function CustomTooltip({
  active,
  payload,
  label,
  labelFormatter
}: CustomTooltipProps) {
  if (active && payload && payload.length) {
    return (
      <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg p-3">
        {label && (
          <p className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-2">
            {labelFormatter ? labelFormatter(label) : label}
          </p>
        )}
        {payload.map((entry, index) => (
          <div key={index} className="flex items-center space-x-2">
            <div
              className="w-3 h-3 rounded-full"
              style={{ backgroundColor: entry.color }}
            />
            <span className="text-xs text-gray-600 dark:text-gray-400">
              {entry.name}:
            </span>
            <span className="text-xs font-semibold text-gray-900 dark:text-gray-100">
              {entry.value}
            </span>
          </div>
        ))}
      </div>
    );
  }
  return null;
}

// Chart container wrapper with consistent styling
interface ChartContainerProps {
  children: React.ReactNode;
  className?: string;
}

export function ChartContainer({ children, className = '' }: ChartContainerProps) {
  return (
    <div className={`bg-gray-50 dark:bg-gray-900/50 rounded-xl p-4 border border-gray-200 dark:border-gray-700 shadow-sm ${className}`}>
      {children}
    </div>
  );
}

// Grid styling for dark mode
export const chartGridProps = {
  strokeDasharray: '3 3',
  stroke: 'currentColor',
  className: 'text-gray-300 dark:text-gray-600',
};

// Axis styling for dark mode
export const chartAxisProps = {
  tick: { fill: 'currentColor' },
  className: 'text-gray-600 dark:text-gray-400 text-xs',
};
