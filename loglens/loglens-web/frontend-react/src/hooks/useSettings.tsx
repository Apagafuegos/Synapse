import { useQuery } from 'react-query';
import { api } from '@services/api';
import type { Settings } from '@/types';

/**
 * Custom hook to manage global settings state
 * Provides cached access to user settings with loading and error states
 */
export const useSettings = () => {
  const {
    data: settings,
    isLoading,
    error,
    refetch
  } = useQuery<Settings>(
    'settings',
    api.system.getSettings,
    {
      // Cache settings for 5 minutes to avoid excessive API calls
      staleTime: 5 * 60 * 1000,
      // Keep settings in cache for 10 minutes
      cacheTime: 10 * 60 * 1000,
      // Retry failed requests up to 2 times
      retry: 2,
      // Don't refetch on window focus for settings
      refetchOnWindowFocus: false,
    }
  );

  return {
    settings,
    isLoading,
    error,
    refetch,
    // Helper functions for common settings access
    getProvider: () => settings?.default_provider || 'openrouter',
    getLevel: () => settings?.default_level || 'ERROR',
    getMaxLines: () => settings?.max_lines || 1000,
    hasApiKey: () => Boolean(settings?.api_key?.trim()),
    isConfigured: () => Boolean(settings?.default_provider && settings?.api_key?.trim()),
  };
};

export default useSettings;