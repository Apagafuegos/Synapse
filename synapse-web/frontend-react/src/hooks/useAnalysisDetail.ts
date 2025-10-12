import { useQuery } from 'react-query';
import { analysisApi, ApiError } from '@/services/api';
import { Analysis } from '@/types';

interface UseAnalysisDetailOptions {
  id: string;
  enabled?: boolean;
}

export const useAnalysisDetail = ({ id, enabled = true }: UseAnalysisDetailOptions) => {
  const query = useQuery<Analysis, ApiError>({
    queryKey: ['analysis', id],
    queryFn: () => {
      console.log(`Fetching analysis with ID: ${id}`);
      return analysisApi.getById(id);
    },
    enabled: enabled && !!id,
    staleTime: 5 * 60 * 1000, // 5 minutes
    retry: (failureCount, error) => {
      // Don't retry for 404 errors
      if (error?.status === 404) return false;
      return failureCount < 3;
    },
    onError: (error) => {
      console.error('Error fetching analysis:', error);
    },
    onSuccess: (data) => {
      console.log('Successfully fetched analysis:', data);
    },
    // Add refetch interval for running analyses
    refetchInterval: (data) => {
      // If analysis is running, poll every 2 seconds
      if (data?.status === 'running') {
        return 2000;
      }
      // If analysis is pending, poll every 3 seconds
      if (data?.status === 'pending') {
        return 3000;
      }
      // Otherwise, no polling needed
      return false;
    },
  });

  return query;
};

export const useAnalysisResult = (analysis: Analysis | undefined): any => {
  const result = analysis?.result;
  
  if (!result) {
    return null;
  }

  try {
    // Parse the result as AnalysisResponse from core library
    const parsedResult: any = JSON.parse(result as unknown as string);
    return parsedResult;
  } catch (error) {
    console.error('Failed to parse analysis result:', error);
    return null;
  }
};

export const useAnalysisWebSocket = (analysisId: string) => {
  // Placeholder for WebSocket functionality
  const connect = () => {
    console.log('WebSocket connection for analysis:', analysisId);
  };

  const disconnect = () => {
    console.log('WebSocket disconnected for analysis:', analysisId);
  };

  return { connect, disconnect };
};