import React from 'react';
import { useQuery, useQueryClient, useMutation } from 'react-query';
import { api } from '@services/api';
import { useState } from 'react';
import { CheckCircleIcon, ExclamationTriangleIcon, InformationCircleIcon, ArrowPathIcon } from '@heroicons/react/24/outline';
import type { Settings, ModelListResponse, ModelInfo } from '@/types';

function Settings() {
  const { data: settings, isLoading } = useQuery<Settings>('settings', api.system.getSettings);
  const [isSaving, setIsSaving] = useState(false);
  const [localSettings, setLocalSettings] = useState<Settings | null>(null);
  const [fetchingModels, setFetchingModels] = useState(false);
  const [availableModels, setAvailableModels] = useState<ModelInfo[]>([]);
  const queryClient = useQueryClient();

  // Initialize local settings when data is loaded
  React.useEffect(() => {
    if (settings && !localSettings) {
      setLocalSettings({ ...settings });
      
      // Parse available models if they exist
      if (settings.available_models) {
        try {
          const parsed = JSON.parse(settings.available_models) as ModelInfo[];
          setAvailableModels(parsed);
        } catch (e) {
          console.error('Failed to parse available models:', e);
        }
      }
    }
  }, [settings, localSettings]);

  // Filter models for current provider
  const providerModels = React.useMemo(() => {
    if (!localSettings?.default_provider) return [];
    return availableModels.filter(model => model.provider === localSettings.default_provider);
  }, [availableModels, localSettings?.default_provider]);

  // Mutation for fetching models
  const fetchModelsMutation = useMutation(
    (forceRefresh: boolean = false) => {
      if (!localSettings?.default_provider || !localSettings.api_key) {
        throw new Error('Provider and API key required');
      }
      
      return api.system.fetchModels(localSettings.default_provider, localSettings.api_key, forceRefresh);
    },
    {
      onSuccess: (response: ModelListResponse) => {
        setAvailableModels(response.models);
        // Update cached models in settings
        if (localSettings) {
          const updatedSettings = {
            ...localSettings,
            available_models: JSON.stringify(response.models),
            models_last_fetched: response.cached_at
          };
          setLocalSettings(updatedSettings);
        }
      },
      onError: (error) => {
        console.error('Failed to fetch models:', error);
      }
    }
  );

  // Fetch models when provider or API key changes
  React.useEffect(() => {
    if (localSettings?.default_provider && localSettings.api_key) {
      fetchModelsMutation.mutate(false);
    }
  }, [localSettings?.default_provider, localSettings?.api_key]);

  const handleSave = async () => {
    if (!localSettings) return;
    
    setIsSaving(true);
    try {
      await api.system.updateSettings(localSettings);
      // Invalidate and refetch settings to ensure consistency
      await queryClient.invalidateQueries('settings');
    } catch (error) {
      console.error('Failed to save settings:', error);
    } finally {
      setIsSaving(false);
    }
  };

  const handleRefreshModels = async () => {
    setFetchingModels(true);
    try {
      await fetchModelsMutation.mutateAsync(true);
    } finally {
      setFetchingModels(false);
    }
  };

  const updateSetting = <K extends keyof Settings>(key: K, value: Settings[K]) => {
    setLocalSettings(prev => prev ? { ...prev, [key]: value } : null);
  };

  if (isLoading || !localSettings) {
    return (
      <div className="flex items-center justify-center min-h-96">
        <div className="text-gray-500">Loading settings...</div>
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <h1 className="text-2xl font-bold text-gray-900 dark:text-white">Settings</h1>
      <p className="mt-2 text-gray-600 dark:text-gray-400">
        Configure your Synapse preferences and AI provider settings.
      </p>

      <div className="mt-8 space-y-6">
        {/* AI Provider Settings */}
        <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6">
          <h2 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
            AI Provider Settings
          </h2>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Default Provider
              </label>
              <select 
                className="input"
                value={localSettings?.default_provider || 'openai'}
                onChange={(e) => updateSetting('default_provider', e.target.value)}
              >
                <option value="openai">OpenAI</option>
                <option value="claude">Claude</option>
                <option value="gemini">Gemini</option>
                <option value="openrouter">OpenRouter</option>
                <option value="mock">Mock (for testing)</option>
              </select>
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                API Key
              </label>
              <input
                type="password"
                className="input"
                placeholder="Enter your API key"
                value={localSettings?.api_key || ''}
                onChange={(e) => updateSetting('api_key', e.target.value)}
              />
            </div>

            {/* Provider Status */}
            <div className="mt-4 p-3 rounded-lg border">
              {localSettings?.api_key?.trim() ? (
                <div className="flex items-center space-x-2 text-green-700 dark:text-green-300">
                  <CheckCircleIcon className="h-5 w-5" />
                  <span className="text-sm">
                    {localSettings.default_provider} provider is configured
                  </span>
                </div>
              ) : (
                <div className="flex items-center space-x-2 text-amber-700 dark:text-amber-300">
                  <ExclamationTriangleIcon className="h-5 w-5" />
                  <span className="text-sm">
                    API key required for {localSettings?.default_provider || 'selected'} provider
                  </span>
                </div>
              )}

              {/* Provider-specific setup instructions */}
              {localSettings?.default_provider && (
                <div className="mt-3 p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
                  <div className="flex items-start space-x-2">
                    <InformationCircleIcon className="h-5 w-5 text-blue-500 mt-0.5" />
                    <div className="text-sm text-blue-700 dark:text-blue-300">
                      <strong>{localSettings.default_provider} Setup:</strong>
                      {localSettings.default_provider === 'openai' && (
                        <p className="mt-1">Get your API key from <a href="https://platform.openai.com/api-keys" target="_blank" rel="noopener noreferrer" className="underline">OpenAI Platform</a></p>
                      )}
                      {localSettings.default_provider === 'claude' && (
                        <p className="mt-1">Get your API key from <a href="https://console.anthropic.com/" target="_blank" rel="noopener noreferrer" className="underline">Anthropic Console</a></p>
                      )}
                      {localSettings.default_provider === 'gemini' && (
                        <p className="mt-1">Get your API key from <a href="https://makersuite.google.com/app/apikey" target="_blank" rel="noopener noreferrer" className="underline">Google AI Studio</a></p>
                      )}
                      {localSettings.default_provider === 'openrouter' && (
                        <p className="mt-1">Get your API key from <a href="https://openrouter.ai/keys" target="_blank" rel="noopener noreferrer" className="underline">OpenRouter Keys</a></p>
                      )}
                      {localSettings.default_provider === 'mock' && (
                        <p className="mt-1">Mock provider for testing - no API key required</p>
                      )}
                    </div>
                  </div>
                </div>
              )}
            </div>
            
            {/* Model Selection */}
            {providerModels.length > 0 && (
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  Default Model
                </label>
                <select 
                  className="input"
                  value={localSettings?.selected_model || ''}
                  onChange={(e) => updateSetting('selected_model', e.target.value)}
                >
                  <option value="">Auto-select best model</option>
                  {providerModels.map((model) => (
                    <option key={model.id} value={model.id}>
                      {model.name} ({model.context_limit?.toLocaleString() || 'Unknown'} tokens)
                    </option>
                  ))}
                </select>
                <div className="flex items-center justify-between mt-2">
                  <span className="text-xs text-gray-500 dark:text-gray-400">
                    {localSettings?.models_last_fetched 
                      ? `Models cached: ${new Date(localSettings.models_last_fetched).toLocaleString()}`
                      : 'No models cached'
                    }
                  </span>
                  <button
                    type="button"
                    onClick={handleRefreshModels}
                    disabled={fetchingModels || !localSettings?.api_key}
                    className="text-xs text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300 disabled:opacity-50 flex items-center space-x-1"
                  >
                    <ArrowPathIcon className={`h-3 w-3 ${fetchingModels ? 'animate-spin' : ''}`} />
                    <span>{fetchingModels ? 'Refreshing...' : 'Refresh'}</span>
                  </button>
                </div>
              </div>
            )}

            {/* Timeout Configuration */}
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Analysis Timeout (seconds)
              </label>
              <input
                type="number"
                className="input"
                min="60"
                max="1800"
                value={localSettings?.analysis_timeout_seconds || 300}
                onChange={(e) => updateSetting('analysis_timeout_seconds', parseInt(e.target.value) || undefined)}
              />
              <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                Set analysis timeout (60-1800 seconds, default: 300)
              </p>
            </div>
          </div>
        </div>

        {/* Analysis Settings */}
        <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6">
          <h2 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
            Analysis Settings
          </h2>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Max Log Lines to Analyze
              </label>
              <input
                type="number"
                className="input"
                min="100"
                max="10000"
                value={localSettings?.max_lines || 1000}
                onChange={(e) => updateSetting('max_lines', parseInt(e.target.value))}
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Default Log Level
              </label>
              <select 
                className="input"
                value={localSettings?.default_level || 'INFO'}
                onChange={(e) => updateSetting('default_level', e.target.value)}
              >
                <option value="ERROR">Error</option>
                <option value="WARN">Warning</option>
                <option value="INFO">Info</option>
                <option value="DEBUG">Debug</option>
              </select>
            </div>
          </div>
        </div>

        {/* UI Settings */}
        <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6">
          <h2 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
            UI Settings
          </h2>
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Show timestamps
              </span>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  className="sr-only peer"
                  checked={localSettings?.show_timestamps || true}
                  onChange={(e) => updateSetting('show_timestamps', e.target.checked)}
                />
                <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-blue-600"></div>
              </label>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Show line numbers
              </span>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  className="sr-only peer"
                  checked={localSettings?.show_line_numbers || true}
                  onChange={(e) => updateSetting('show_line_numbers', e.target.checked)}
                />
                <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-blue-600"></div>
              </label>
            </div>
          </div>
        </div>

        {/* Save Button */}
        <div className="flex justify-end">
          <button
            type="button"
            onClick={handleSave}
            disabled={isSaving}
            className="btn-primary"
          >
            {isSaving ? 'Saving...' : 'Save Settings'}
          </button>
        </div>
      </div>
    </div>
  );
}

export default Settings;