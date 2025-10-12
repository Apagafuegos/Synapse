import type {
  Project,
  CreateProjectRequest,
  LogFile,
  Analysis,
  AnalysisRequest,
  AnalysisListResponse,
  Settings,
  DashboardStats,
  ModelListResponse
} from '@/types';

const API_BASE = '/api';

console.log('API Service: API_BASE is set to:', API_BASE);

class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
    public code?: string
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

async function fetchApi<T>(
  url: string,
  options: RequestInit = {}
): Promise<T> {
  console.log(`API Request: ${API_BASE}${url}`, options);
  
  const response = await fetch(`${API_BASE}${url}`, {
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    },
    ...options,
  });

  console.log(`API Response: ${response.status} ${response.statusText}`);

  if (!response.ok) {
    let errorMessage = `HTTP ${response.status}`;
    let errorCode: string | undefined;

    try {
      const errorData = await response.json();
      errorMessage = errorData.message || errorData.error || errorMessage;
      errorCode = errorData.code;
      console.error('API Error:', errorData);
    } catch {
      // If we can't parse the error response, use the status text
      errorMessage = response.statusText || errorMessage;
    }

    throw new ApiError(errorMessage, response.status, errorCode);
  }

  // Handle empty responses (like DELETE operations)
  if (response.status === 204 || response.headers.get('content-length') === '0') {
    return undefined as unknown as T;
  }

  try {
    const data = await response.json();
    console.log('API Success Response:', data);
    return data;
  } catch (error) {
    throw new ApiError('Invalid JSON response', response.status);
  }
}

// Project management
export const projectsApi = {
  async getAll(): Promise<Project[]> {
    return fetchApi<Project[]>('/projects');
  },

  async getById(id: string): Promise<Project> {
    return fetchApi<Project>(`/projects/${id}`);
  },

  async create(request: CreateProjectRequest): Promise<Project> {
    return fetchApi<Project>('/projects', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  },

  async delete(id: string): Promise<void> {
    return fetchApi<void>(`/projects/${id}`, {
      method: 'DELETE',
    });
  },

  async update(id: string, updates: Partial<CreateProjectRequest>): Promise<Project> {
    return fetchApi<Project>(`/projects/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(updates),
    });
  },
};

// File management
export const filesApi = {
  async getByProject(projectId: string): Promise<LogFile[]> {
    return fetchApi<LogFile[]>(`/projects/${projectId}/files`);
  },

  async upload(projectId: string, file: File): Promise<LogFile> {
    const formData = new FormData();
    formData.append('file', file);

    return fetchApi<LogFile>(`/projects/${projectId}/files`, {
      method: 'POST',
      headers: {}, // Remove Content-Type header to let browser set it for FormData
      body: formData,
    });
  },

  async delete(projectId: string, fileId: string): Promise<void> {
    return fetchApi<void>(`/projects/${projectId}/files/${fileId}`, {
      method: 'DELETE',
    });
  },

  async getContent(projectId: string, fileId: string, lines?: number): Promise<string> {
    const params = new URLSearchParams();
    if (lines) params.append('lines', lines.toString());

    const url = `/projects/${projectId}/files/${fileId}/content${params.toString() ? `?${params}` : ''}`;
    return fetchApi<string>(url);
  },
};

// Analysis management
export const analysisApi = {
  async create(
    projectId: string,
    fileId: string,
    request: AnalysisRequest
  ): Promise<Analysis> {
    return fetchApi<Analysis>(`/projects/${projectId}/files/${fileId}/analyze`, {
      method: 'POST',
      body: JSON.stringify(request),
    });
  },

  async getById(id: string): Promise<Analysis> {
    return fetchApi<Analysis>(`/analyses/${id}`);
  },

  async getByProject(
    projectId: string,
    page: number = 1,
    perPage: number = 20
  ): Promise<AnalysisListResponse> {
    const params = new URLSearchParams({
      page: page.toString(),
      per_page: perPage.toString(),
    });

    return fetchApi<AnalysisListResponse>(`/projects/${projectId}/analyses?${params}`);
  },

  async cancel(id: string): Promise<void> {
    return fetchApi<void>(`/analyses/${id}/cancel`, {
      method: 'POST',
    });
  },

  async delete(id: string): Promise<void> {
    return fetchApi<void>(`/analyses/${id}`, {
      method: 'DELETE',
    });
  },

  async export(analysisId: string, projectId: string, format: 'json' | 'html' | 'md' | 'csv' | 'pdf' = 'json'): Promise<Blob> {
    const response = await fetch(`${API_BASE}/projects/${projectId}/analyses/${analysisId}/export/${format}`, {
      method: 'GET',
    });

    if (!response.ok) {
      throw new ApiError(`Export failed: HTTP ${response.status}`, response.status);
    }

    return response.blob();
  },
};

// Dashboard management
export const dashboardApi = {
  async getStats(): Promise<DashboardStats> {
    return fetchApi<DashboardStats>('/dashboard/stats');
  },
};

// System management
export const systemApi = {
  async getHealth(): Promise<{ status: string; version: string }> {
    return fetchApi<{ status: string; version: string }>('/health');
  },

  async getSettings(): Promise<Settings> {
    return fetchApi<Settings>('/settings');
  },

  async updateSettings(settings: Settings): Promise<Settings> {
    return fetchApi<Settings>('/settings', {
      method: 'PATCH',
      body: JSON.stringify(settings),
    });
  },

  async fetchModels(provider: string, apiKey: string, forceRefresh: boolean = false): Promise<ModelListResponse> {
    return fetchApi<ModelListResponse>('/models/available', {
      method: 'POST',
      body: JSON.stringify({
        provider,
        api_key: apiKey,
        force_refresh: forceRefresh
      }),
    });
  },
};

// Export the main API object
export const api = {
  projects: projectsApi,
  files: filesApi,
  analysis: analysisApi,
  dashboard: dashboardApi,
  system: systemApi,
  
  // Generic methods for custom requests
  post: <T = any>(url: string, data?: any) => fetchApi<T>(url, {
    method: 'POST',
    body: data ? JSON.stringify(data) : undefined,
  }),
  
  get: <T = any>(url: string) => fetchApi<T>(url),
};

export { ApiError };