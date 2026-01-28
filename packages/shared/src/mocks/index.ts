import type {
  ProjectState,
  BranchData,
  VersionInfo,
  WorkingFileInfo,
  EtabsStatus,
  CliResult,
  ValidationData,
  GenerateE2KData,
  E2KDiffResult,
  E2KChange,
  GeometryDiffResult
} from '../types';

// Mock ETABS Status
export const mockEtabsStatus: EtabsStatus = {
  is_running: false,
  version: 'ETABS v22.0.0',
  current_file: null
};

// Mock Version Info
export const mockVersions: Record<string, VersionInfo[]> = {
  main: [
    {
      id: 'v3',
      message: 'Final review of main design - updated column sections',
      author: 'John Doe',
      timestamp: '2025-01-20T14:30:00Z',
      e2k_path: 'D:\\Projects\\HighRise\\main\\v3\\model.e2k',
      analyzed: true
    },
    {
      id: 'v2',
      message: 'Updated load combinations per new code requirements',
      author: 'Jane Smith',
      timestamp: '2025-01-18T16:45:00Z',
      e2k_path: 'D:\\Projects\\HighRise\\main\\v2\\model.e2k',
      analyzed: true
    },
    {
      id: 'v1',
      message: 'Initial structural design with preliminary member sizing',
      author: 'Jane Smith',
      timestamp: '2025-01-15T10:00:00Z',
      e2k_path: 'D:\\Projects\\HighRise\\main\\v1\\model.e2k',
      analyzed: true
    }
  ],
  'steel-columns': [
    {
      id: 'v2',
      message: 'Optimized connection details for steel columns',
      author: 'John Doe',
      timestamp: '2025-01-23T11:20:00Z',
      e2k_path: 'D:\\Projects\\HighRise\\steel-columns\\v2\\model.e2k',
      analyzed: true
    },
    {
      id: 'v1',
      message: 'Changed columns from concrete to steel W-sections',
      author: 'John Doe',
      timestamp: '2025-01-22T09:15:00Z',
      e2k_path: 'D:\\Projects\\HighRise\\steel-columns\\v1\\model.e2k',
      analyzed: false
    }
  ],
  'foundation-redesign': [
    {
      id: 'v1',
      message: 'Deep foundation design with drilled piers',
      author: 'John Doe',
      timestamp: '2025-01-21T11:20:00Z',
      e2k_path: null,
      analyzed: false
    }
  ],
  'cost-reduction': [
    {
      id: 'v1',
      message: 'Exploring cheaper material alternatives',
      author: 'Jane Smith',
      timestamp: '2025-01-19T14:00:00Z',
      e2k_path: null,
      analyzed: false
    }
  ]
};

// Mock Working Files
const mockWorkingFiles: Record<string, WorkingFileInfo> = {
  main: {
    exists: true,
    path: 'D:\\Projects\\HighRise\\main\\working\\model.edb',
    is_open: false,
    has_unsaved_changes: false,
    source_version: 'v3'
  },
  'steel-columns': {
    exists: true,
    path: 'D:\\Projects\\HighRise\\steel-columns\\working\\model.edb',
    is_open: false,
    has_unsaved_changes: true,
    source_version: 'v2'
  },
  'foundation-redesign': {
    exists: true,
    path: 'D:\\Projects\\HighRise\\foundation-redesign\\working\\model.edb',
    is_open: false,
    has_unsaved_changes: false,
    source_version: 'v1'
  },
  'cost-reduction': {
    exists: false,
    path: 'D:\\Projects\\HighRise\\cost-reduction\\working\\model.edb',
    is_open: false,
    has_unsaved_changes: false,
    source_version: null
  }
};

// Mock Branch Data
export const mockBranches: Record<string, BranchData> = {
  main: {
    name: 'main',
    description: 'Primary design branch',
    versions: mockVersions.main,
    latest_version: 'v3',
    parent_branch: null,
    parent_version: null,
    created: '2025-01-15T09:00:00Z',
    working_file: mockWorkingFiles.main
  },
  'steel-columns': {
    name: 'steel-columns',
    description: 'Exploring steel column alternative to reduce cost',
    versions: mockVersions['steel-columns'],
    latest_version: 'v2',
    parent_branch: 'main',
    parent_version: 'v2',
    created: '2025-01-22T09:00:00Z',
    working_file: mockWorkingFiles['steel-columns']
  },
  'foundation-redesign': {
    name: 'foundation-redesign',
    description: 'Deep foundation design for poor soil conditions',
    versions: mockVersions['foundation-redesign'],
    latest_version: 'v1',
    parent_branch: 'main',
    parent_version: 'v2',
    created: '2025-01-21T10:00:00Z',
    working_file: mockWorkingFiles['foundation-redesign']
  },
  'cost-reduction': {
    name: 'cost-reduction',
    description: 'Value engineering to reduce overall project cost',
    versions: mockVersions['cost-reduction'],
    latest_version: 'v1',
    parent_branch: 'main',
    parent_version: 'v3',
    created: '2025-01-19T13:30:00Z',
    working_file: mockWorkingFiles['cost-reduction']
  }
};

// Mock Project State
export const mockProjectState: ProjectState = {
  project_name: 'HighRise Tower',
  project_path: 'D:\\Projects\\HighRise',
  current_branch: 'main',
  branches: mockBranches
};

// Mock Validation Result
export const mockValidationSuccess: CliResult<ValidationData> = {
  success: true,
  error: null,
  timestamp: new Date().toISOString(),
  data: {
    etabs_installed: true,
    etabs_version: 'ETABS v22.0.0 Build 3400',
    file_valid: true,
    file_path: 'D:\\Projects\\HighRise\\main\\working\\model.edb',
    file_exists: true,
    file_extension: '.edb',
    is_analyzed: true,
    validation_messages: [
      '✓ ETABS installation detected',
      '✓ File exists and is accessible',
      '✓ File is valid ETABS database',
      '✓ Model has been analyzed',
      '✓ Ready for processing'
    ]
  }
};

export const mockValidationFailure: CliResult<ValidationData> = {
  success: false,
  error: 'File not found',
  timestamp: new Date().toISOString(),
  data: {
    etabs_installed: true,
    etabs_version: 'ETABS v22.0.0 Build 3400',
    file_valid: false,
    file_path: 'D:\\Projects\\NonExistent\\model.edb',
    file_exists: false,
    file_extension: null,
    is_analyzed: null,
    validation_messages: [
      '✓ ETABS installation detected',
      '✗ File does not exist at specified path',
      '⚠ Cannot validate file properties'
    ]
  }
};

// Mock E2K Generation Result
export const mockE2KGenerationSuccess: CliResult<GenerateE2KData> = {
  success: true,
  error: null,
  timestamp: new Date().toISOString(),
  data: {
    input_file: 'D:\\Projects\\HighRise\\main\\working\\model.edb',
    output_file: 'D:\\Projects\\HighRise\\main\\working\\model.e2k',
    file_exists: true,
    file_extension: '.edb',
    output_exists: true,
    generation_successful: true,
    file_size_bytes: 2458624,
    generation_time_ms: 8450,
    messages: [
      '✓ Input file validated',
      '✓ ETABS instance started',
      '✓ Model opened successfully',
      '✓ E2K export initiated',
      '✓ E2K file generated: model.e2k',
      '✓ File size: 2.35 MB',
      '✓ Generation completed in 8.45 seconds'
    ]
  }
};

// Mock E2K Changes
export const mockE2KChanges: E2KChange[] = [
  {
    change_type: 'modify',
    category: 'Frame Section',
    description: 'Column C1 section changed',
    old_value: 'W14X90',
    new_value: 'W14X120'
  },
  {
    change_type: 'add',
    category: 'Load Pattern',
    description: 'Added seismic load pattern EQX',
    old_value: null,
    new_value: 'EQX: Seismic X-Direction'
  },
  {
    change_type: 'modify',
    category: 'Load Combination',
    description: 'Updated load factors for LRFD-1',
    old_value: '1.2D + 1.6L',
    new_value: '1.2D + 1.6L + 0.5Lr'
  },
  {
    change_type: 'remove',
    category: 'Frame Element',
    description: 'Removed temporary bracing member B45',
    old_value: 'B45: Bracing at Grid 5',
    new_value: null
  },
  {
    change_type: 'add',
    category: 'Joint',
    description: 'Added joint at intersection of Grid A and Grid 8',
    old_value: null,
    new_value: 'J128 (X: 120.5, Y: 80.0, Z: 45.0)'
  },
  {
    change_type: 'modify',
    category: 'Material',
    description: 'Concrete strength updated for mat foundation',
    old_value: 'fc = 4000 psi',
    new_value: 'fc = 5000 psi'
  }
];

// Mock E2K Diff Result
export const mockE2KDiff: E2KDiffResult = {
  added: 12,
  removed: 5,
  modified: 18,
  changes: mockE2KChanges,
  raw_diff: `--- main/v2/model.e2k
+++ main/v3/model.e2k
@@ -145,7 +145,7 @@
 $ FRAME SECTION PROPERTIES
-   FSEC=C1   SECTION=W14X90   MATERIAL=STEEL   
+   FSEC=C1   SECTION=W14X120  MATERIAL=STEEL   
@@ -892,6 +892,8 @@
 $ LOAD PATTERNS
    LOADPAT=DEAD  TYPE=DEAD
    LOADPAT=LIVE  TYPE=LIVE
+   LOADPAT=EQX   TYPE=QUAKE  DIR=X
@@ -1203,7 +1205,7 @@
 $ LOAD COMBINATIONS
-   COMBO=LRFD-1  LOAD=DEAD  SF=1.2  LOAD=LIVE  SF=1.6
+   COMBO=LRFD-1  LOAD=DEAD  SF=1.2  LOAD=LIVE  SF=1.6  LOAD=LROOF  SF=0.5`
};

// Mock Geometry Diff Result
export const mockGeometryDiff: GeometryDiffResult = {
  members_added: ['C45', 'C46', 'B78', 'B79', 'B80'],
  members_removed: ['B45', 'B67'],
  members_modified: ['C1', 'C2', 'C3', 'B12', 'B13', 'B14', 'B15', 'B16'],
  total_changes: 15
};

// Mock E2K Diff with Comparison
export const mockComparisonResult: CliResult<{
  e2kDiff: E2KDiffResult;
  geometryDiff: GeometryDiffResult;
}> = {
  success: true,
  error: null,
  timestamp: new Date().toISOString(),
  data: {
    e2kDiff: mockE2KDiff,
    geometryDiff: mockGeometryDiff
  }
};

// Helper to create mock CLI results
export function createMockCliResult<T>(
    data: T,
    options: { success?: boolean; error?: string } = {}
): CliResult<T> {
  return {
    success: options.success ?? true,
    error: options.error ?? null,
    timestamp: new Date().toISOString(),
    data: options.success === false ? null : data
  };
}

// Helper to simulate async operations
export function mockAsync<T>(data: T, delay: number = 1000): Promise<T> {
  return new Promise((resolve) => {
    setTimeout(() => resolve(data), delay);
  });
}

// Helper to simulate random failures
export function mockAsyncWithFailure<T>(
    data: T,
    options: { delay?: number; failureRate?: number; errorMessage?: string } = {}
): Promise<CliResult<T>> {
  const { delay = 1000, failureRate = 0, errorMessage = 'Operation failed' } = options;

  return new Promise((resolve) => {
    setTimeout(() => {
      const shouldFail = Math.random() < failureRate;
      resolve(
          createMockCliResult(data, {
            success: !shouldFail,
            error: shouldFail ? errorMessage : undefined
          })
      );
    }, delay);
  });
}