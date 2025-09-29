# LogLens AI Provider Settings Integration Plan

## Problem Summary
Currently, the frontend hardcodes the AI provider as 'openai' in the analysis request, but the system has a complete settings infrastructure that should be used instead. The settings page allows users to configure their preferred AI provider, but this setting is not being used during analysis.

## Current System Analysis

### ✅ **Existing Infrastructure (Already Working)**
- **Database**: Settings table with `default_provider` field
- **Backend API**: Complete settings CRUD endpoints (`/api/settings`)
- **Frontend**: Fully functional Settings page with provider selection
- **Types**: Proper TypeScript interfaces for Settings
- **Validation**: Backend validates provider choices

### 🔧 **Required Changes**

## Phase 1: Frontend Integration (Primary Changes)

### 1.1 Settings Context/Hook
**File**: `loglens-web/frontend-react/src/hooks/useSettings.tsx` (NEW)
- Create custom hook to manage settings state
- Cache settings data globally
- Provide easy access to current provider setting

### 1.2 Update ProjectDetail Component
**File**: `loglens-web/frontend-react/src/pages/ProjectDetail.tsx`
- Remove hardcoded `provider: 'openai'`
- Use settings hook to get default provider
- Add fallback handling for loading states
- Show user-friendly error if no provider configured

### 1.3 Analysis Request Enhancement
**File**: `loglens-web/frontend-react/src/pages/ProjectDetail.tsx`
- Integrate with settings to use `default_provider` and `default_level`
- Add loading state while fetching settings
- Handle case where settings are not yet loaded

## Phase 2: User Experience Improvements

### 2.1 Settings Validation
**File**: `loglens-web/frontend-react/src/pages/Settings.tsx`
- Add API key validation warnings
- Show provider-specific setup instructions
- Display current provider status (configured/not configured)

### 2.2 Analysis UI Enhancement
**File**: `loglens-web/frontend-react/src/pages/ProjectDetail.tsx`
- Show which provider will be used before analysis
- Allow per-analysis provider override (optional)
- Better error messaging for provider issues

## Phase 3: Backend Optimization (Optional)

### 3.1 Settings Fallback
**File**: `loglens-web/src/handlers/analysis.rs`
- If no API key in settings, check environment variables
- Provide clearer error messages about provider configuration

## Implementation Steps

### Step 1: Create Settings Hook
```typescript
// New file: src/hooks/useSettings.tsx
export const useSettings = () => {
  const { data: settings, isLoading, error } = useQuery('settings', api.system.getSettings);
  return { settings, isLoading, error };
};
```

### Step 2: Update ProjectDetail Analysis
```typescript
// In ProjectDetail.tsx - handleStartAnalysis function
const { settings, isLoading: settingsLoading } = useSettings();

// Replace hardcoded provider
const analysis = await api.analysis.create(id, fileId, {
  provider: settings?.default_provider || 'openrouter', // fallback
  level: settings?.default_level || 'ERROR',
  user_context: undefined
});
```

### Step 3: Add Loading States
- Show spinner while settings load
- Disable analyze button until settings are available
- Clear error messaging for missing configuration

### Step 4: Update Default Settings
**File**: `loglens-web/migrations/20240101000003_settings.sql`
- Update default provider from 'openai' to 'openrouter'
- This ensures new installations work out of the box

## Files to Modify

1. **NEW**: `loglens-web/frontend-react/src/hooks/useSettings.tsx`
2. **MODIFY**: `loglens-web/frontend-react/src/pages/ProjectDetail.tsx`
3. **ENHANCE**: `loglens-web/frontend-react/src/pages/Settings.tsx`
4. **UPDATE**: `loglens-web/migrations/20240101000003_settings.sql`

## Testing Plan

1. **Settings Integration**: Verify provider selection flows from settings to analysis
2. **Fallback Behavior**: Test behavior when settings not loaded/available
3. **Error Handling**: Ensure clear error messages for misconfigured providers
4. **User Experience**: Confirm smooth workflow from settings → analysis

## Migration Strategy

1. **Immediate Fix**: Change hardcoded 'openai' to 'openrouter' (current working provider)
2. **Settings Integration**: Implement proper settings-based provider selection
3. **Database Update**: Update default provider in database to match available API keys
4. **User Communication**: Update settings page with setup guidance

## Expected Benefits

- ✅ **User Control**: Users can select their preferred AI provider
- ✅ **Flexibility**: Easy to switch providers without code changes
- ✅ **Maintainability**: Centralized provider configuration
- ✅ **Better UX**: Clear setup flow and error handling

## Implementation Status ✅ COMPLETED

- [x] Problem diagnosed - hardcoded 'openai' provider without API key
- [x] Temporary fix applied - changed to 'openrouter'
- [x] Settings hook implementation - `useSettings.tsx` created
- [x] ProjectDetail integration - Dynamic provider selection implemented
- [x] Settings page enhancements - Provider status and setup instructions added
- [x] Database migration - Default provider updated to 'openrouter'
- [x] Testing and validation - API endpoints verified working

## 🎉 Implementation Complete!

The AI provider settings integration has been successfully implemented. Users can now:
1. **Configure their preferred AI provider** via the Settings page
2. **See real-time provider status** and setup instructions
3. **Analyze logs using their configured provider** automatically
4. **Get helpful error messages** if configuration is incomplete

### Files Modified:
- ✅ **NEW**: `loglens-web/frontend-react/src/hooks/useSettings.tsx` - Settings state management
- ✅ **ENHANCED**: `loglens-web/frontend-react/src/pages/ProjectDetail.tsx` - Settings-based provider selection
- ✅ **ENHANCED**: `loglens-web/frontend-react/src/pages/Settings.tsx` - Provider validation and status
- ✅ **UPDATED**: `loglens-web/migrations/20240101000003_settings.sql` - Default provider changed
- ✅ **UPDATED**: `loglens-web/src/handlers/settings.rs` - Backend defaults updated
- ✅ **UPDATED**: Database settings record - Updated to use OpenRouter with API key

### Testing Results:
- ✅ Settings API endpoint returns correct configuration
- ✅ Analysis workflow now uses settings-based provider selection
- ✅ Frontend shows provider status and configuration guidance
- ✅ Database properly configured with working OpenRouter API key