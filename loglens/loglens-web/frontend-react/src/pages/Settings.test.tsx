import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from 'react-query';
import Settings from './Settings';

// Mock the API calls
jest.mock('../services/api', () => ({
  projects: {},
  files: {},
  analysis: {},
  system: {
    getSettings: jest.fn(() => Promise.resolve({
      default_provider: 'openai',
      api_key: 'test-key',
      max_lines: 1000,
      default_level: 'INFO',
      show_timestamps: true,
      show_line_numbers: true
    })),
    updateSettings: jest.fn(() => Promise.resolve({
      default_provider: 'openai',
      api_key: 'test-key',
      max_lines: 1000,
      default_level: 'INFO',
      show_timestamps: true,
      show_line_numbers: true
    }))
  }
}));

const queryClient = new QueryClient();

const renderSettings = () => {
  return render(
    <QueryClientProvider client={queryClient}>
      <Settings />
    </QueryClientProvider>
  );
};

describe('Settings Component', () => {
  test('renders settings form correctly', async () => {
    renderSettings();
    
    // Wait for the component to load
    await waitFor(() => {
      expect(screen.getByText('Settings')).toBeInTheDocument();
    });
  });

  test('allows changing AI provider', async () => {
    renderSettings();
    
    // Wait for the component to load
    await waitFor(() => {
      expect(screen.getByText('Default Provider')).toBeInTheDocument();
    });
    
    const providerSelect = screen.getByLabelText('Default Provider');
    
    // Test changing the provider
    fireEvent.change(providerSelect, { target: { value: 'claude' } });
    
    await waitFor(() => {
      expect(providerSelect).toHaveValue('claude');
    });
  });

  test('allows changing max log lines', async () => {
    renderSettings();
    
    // Wait for the component to load
    await waitFor(() => {
      expect(screen.getByText('Max Log Lines to Analyze')).toBeInTheDocument();
    });
    
    const maxLogLinesInput = screen.getByLabelText('Max Log Lines to Analyze');
    
    // Test changing the value
    fireEvent.change(maxLogLinesInput, { target: { value: '2000' } });
    
    await waitFor(() => {
      expect(maxLogLinesInput).toHaveValue(2000);
    });
  });

  test('allows changing default log level', async () => {
    renderSettings();
    
    // Wait for the component to load
    await waitFor(() => {
      expect(screen.getByText('Default Log Level')).toBeInTheDocument();
    });
    
    const levelSelect = screen.getByLabelText('Default Log Level');
    
    // Test changing the value
    fireEvent.change(levelSelect, { target: { value: 'ERROR' } });
    
    await waitFor(() => {
      expect(levelSelect).toHaveValue('ERROR');
    });
  });

  test('allows toggling show timestamps', async () => {
    renderSettings();
    
    // Wait for the component to load
    await waitFor(() => {
      expect(screen.getByText('Show timestamps')).toBeInTheDocument();
    });
    
    const timestampToggle = screen.getByLabelText('Show timestamps');
    
    // Test toggling the value
    fireEvent.click(timestampToggle);
    
    await waitFor(() => {
      expect(timestampToggle).not.toBeChecked();
    });
  });

  test('allows toggling show line numbers', async () => {
    renderSettings();
    
    // Wait for the component to load
    await waitFor(() => {
      expect(screen.getByText('Show line numbers')).toBeInTheDocument();
    });
    
    const lineNumbersToggle = screen.getByLabelText('Show line numbers');
    
    // Test toggling the value
    fireEvent.click(lineNumbersToggle);
    
    await waitFor(() => {
      expect(lineNumbersToggle).not.toBeChecked();
    });
  });
});