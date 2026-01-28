/**
 * ETABS Extension - Tauri Command Contract
 *
 * This file defines the interface between Frontend (React) and Backend (Rust).
 * Both agents must agree on these types before implementation.
 *
 * Version: 1.0.0
 * Last Updated: 2025-01-26
 */

// ============================================================================
// PHASE 1: CORE VERSION CONTROL
// ============================================================================

/**
 * Common result wrapper for all Tauri commands
 * Matches Rust's Result<T, String> pattern
 */
export interface TauriResult<T> {
    success: boolean;
    data?: T;
    error?: string;
    timestamp: string; // ISO 8601
}

// ----------------------------------------------------------------------------
// 1. PROJECT MANAGEMENT
// ----------------------------------------------------------------------------

/**
 * Command: create_project
 * Purpose: Initialize a new ETABS project with version control
 */
export interface CreateProjectRequest {
    projectPath: string;      // Directory where project will be created
    projectName: string;      // Human-readable project name
    initialEdbFile?: string;  // Optional: Copy initial .edb file
}

export interface CreateProjectResponse {
    projectPath: string;
    projectName: string;
    initializedGit: boolean;
    createdBranches: string[];  // e.g., ["main"]
    state: ProjectState;
}

/**
 * Command: open_project
 * Purpose: Load an existing ETABS project
 */
export interface OpenProjectRequest {
    projectPath: string;
}

export interface OpenProjectResponse {
    state: ProjectState;
    lastModified: string;
}

/**
 * Command: get_project_state
 * Purpose: Retrieve current project state (polling/refresh)
 */
export interface GetProjectStateRequest {
    projectPath: string;
}

export interface GetProjectStateResponse {
    state: ProjectState;
}

// ----------------------------------------------------------------------------
// 2. BRANCH OPERATIONS
// ----------------------------------------------------------------------------

/**
 * Command: create_branch
 * Purpose: Create a new design alternative branch
 */
export interface CreateBranchRequest {
    projectPath: string;
    branchName: string;
    fromBranch: string;       // Parent branch (e.g., "main")
    fromVersion: string;      // Parent version (e.g., "v3")
    description?: string;     // Optional description
}

export interface CreateBranchResponse {
    branchName: string;
    parentBranch: string;
    parentVersion: string;
    created: string;          // ISO timestamp
    workingFileCreated: boolean;
}

/**
 * Command: switch_branch
 * Purpose: Switch to a different branch
 */
export interface SwitchBranchRequest {
    projectPath: string;
    branchName: string;
    closeCurrentFile: boolean; // Should we close ETABS first?
}

export interface SwitchBranchResponse {
    currentBranch: string;
    workingFileReady: boolean;
    etabsWasClosed: boolean;
}

/**
 * Command: list_branches
 * Purpose: Get all branches in project
 */
export interface ListBranchesRequest {
    projectPath: string;
}

export interface ListBranchesResponse {
    branches: BranchInfo[];
    currentBranch: string;
}

/**
 * Command: delete_branch
 * Purpose: Remove a branch and all its versions
 */
export interface DeleteBranchRequest {
    projectPath: string;
    branchName: string;
    forceDelete: boolean;  // Delete even if has uncommitted changes
}

export interface DeleteBranchResponse {
    deleted: boolean;
    deletedVersions: string[];
    freedSpaceBytes: number;
}

// ----------------------------------------------------------------------------
// 3. VERSION OPERATIONS
// ----------------------------------------------------------------------------

/**
 * Command: save_version
 * Purpose: Save current working file as a new version
 */
export interface SaveVersionRequest {
    projectPath: string;
    branchName: string;
    message: string;          // Commit message
    author?: string;          // Optional author override
    generateE2k: boolean;     // Should we auto-generate E2K?
}

export interface SaveVersionResponse {
    versionId: string;        // e.g., "v4"
    commitHash: string;       // Git SHA
    e2kGenerated: boolean;
    e2kPath?: string;
    fileSize: number;         // Bytes
    timestamp: string;
}

/**
 * Command: list_versions
 * Purpose: Get all versions for a branch
 */
export interface ListVersionsRequest {
    projectPath: string;
    branchName: string;
}

export interface ListVersionsResponse {
    versions: VersionInfo[];
    workingFile: WorkingFileInfo | null;
}

/**
 * Command: checkout_version
 * Purpose: Load a specific version into working directory
 */
export interface CheckoutVersionRequest {
    projectPath: string;
    branchName: string;
    versionId: string;
    openInEtabs: boolean;     // Auto-open after checkout?
}

export interface CheckoutVersionResponse {
    checkedOut: boolean;
    workingFilePath: string;
    etabsOpened: boolean;
}

/**
 * Command: compare_versions
 * Purpose: Get diff between two versions
 */
export interface CompareVersionsRequest {
    projectPath: string;
    version1: {
        branch: string;
        versionId: string;
    };
    version2: {
        branch: string;
        versionId: string;
    };
    diffType: 'e2k' | 'geometry' | 'both';
}

export interface CompareVersionsResponse {
    e2kDiff?: E2KDiffResult;
    geometryDiff?: GeometryDiffResult;
}

// ----------------------------------------------------------------------------
// 4. ETABS INTEGRATION
// ----------------------------------------------------------------------------

/**
 * Command: open_in_etabs
 * Purpose: Open a file in ETABS application
 */
export interface OpenInEtabsRequest {
    projectPath: string;
    branchName: string;
    versionId?: string;  // If null, open working file
}

export interface OpenInEtabsResponse {
    opened: boolean;
    filePath: string;
    etabsProcessId?: number;
}

/**
 * Command: close_etabs
 * Purpose: Close ETABS application
 */
export interface CloseEtabsRequest {
    saveChanges: boolean;
    processId?: number;  // Optional: close specific instance
}

export interface CloseEtabsResponse {
    closed: boolean;
    changesSaved: boolean;
}

/**
 * Command: get_etabs_status
 * Purpose: Check if ETABS is running and which file is open
 */
export interface GetEtabsStatusRequest {
    projectPath: string;
}

export interface GetEtabsStatusResponse {
    isRunning: boolean;
    openFilePath?: string;
    processId?: number;
    canSave: boolean;
}

/**
 * Command: generate_e2k
 * Purpose: Generate E2K file from EDB (using CLI)
 */
export interface GenerateE2kRequest {
    edbPath: string;
    outputPath?: string;  // Optional: auto-generate if null
    overwrite: boolean;
}

export interface GenerateE2kResponse {
    success: boolean;
    e2kPath: string;
    fileSize: number;
    generationTimeMs: number;
    messages: string[];
}

// ----------------------------------------------------------------------------
// 5. GIT OPERATIONS
// ----------------------------------------------------------------------------

/**
 * Command: git_commit
 * Purpose: Create a Git commit (low-level, usually called by save_version)
 */
export interface GitCommitRequest {
    projectPath: string;
    message: string;
    author?: string;
    files?: string[];  // Specific files to commit, or all if empty
}

export interface GitCommitResponse {
    commitHash: string;
    committed: boolean;
    filesCommitted: number;
}

/**
 * Command: git_log
 * Purpose: Get Git history for a branch
 */
export interface GitLogRequest {
    projectPath: string;
    branchName: string;
    limit?: number;  // Max commits to return
}

export interface GitLogResponse {
    commits: GitCommit[];
}

/**
 * Command: git_diff
 * Purpose: Get Git diff between two commits/refs
 */
export interface GitDiffRequest {
    projectPath: string;
    ref1: string;  // Commit hash or branch name
    ref2: string;
    filePath?: string;  // Optional: diff specific file
}

export interface GitDiffResponse {
    diff: string;  // Unified diff format
    filesChanged: number;
    insertions: number;
    deletions: number;
}

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/**
 * Complete project state
 * This is the single source of truth
 */
export interface ProjectState {
    projectPath: string;
    projectName: string;
    currentBranch: string;
    created: string;          // ISO timestamp
    lastModified: string;

    branches: Record<string, BranchInfo>;

    // Git info
    gitInitialized: boolean;
    gitRemoteUrl?: string;
}

/**
 * Branch information
 */
export interface BranchInfo {
    name: string;
    latestVersion: string;
    parentBranch?: string;
    parentVersion?: string;
    created: string;
    description?: string;

    versions: VersionInfo[];
    workingFile: WorkingFileInfo | null;
}

/**
 * Version information
 */
export interface VersionInfo {
    id: string;               // e.g., "v3"
    timestamp: string;
    message: string;
    author?: string;
    commitHash: string;

    // File info
    edbPath: string;
    e2kPath?: string;
    fileSize: number;

    // Analysis status
    analyzed: boolean;
    analysisResults?: AnalysisResults;
}

/**
 * Working file information
 */
export interface WorkingFileInfo {
    exists: boolean;
    sourceVersion: string | null;  // Which version it's based on
    isOpen: boolean;               // Open in ETABS?
    hasUnsavedChanges: boolean;
    lastModified: string | null;
    path: string;
}

/**
 * E2K diff result
 */
export interface E2KDiffResult {
    added: number;
    removed: number;
    modified: number;

    changes: E2KChange[];

    // Raw diff text (unified format)
    rawDiff: string;
}

export interface E2KChange {
    type: 'add' | 'remove' | 'modify';
    category: 'material' | 'section' | 'member' | 'load' | 'analysis' | 'design';
    description: string;
    lineNumber: number;
    oldValue?: string;
    newValue?: string;
}

/**
 * Geometry diff result (for 3D visualization)
 */
export interface GeometryDiffResult {
    membersAdded: GeometryElement[];
    membersRemoved: GeometryElement[];
    membersModified: GeometryElement[];

    // Summary stats
    totalChanges: number;
}

export interface GeometryElement {
    id: string;
    type: 'column' | 'beam' | 'slab' | 'wall' | 'foundation';
    coordinates: number[][];  // 3D coordinates
    properties: Record<string, any>;
}

/**
 * Analysis results (extracted from ETABS)
 */
export interface AnalysisResults {
    timestamp: string;

    // Structural performance
    maxDisplacement: number;      // mm
    maxDrift: number;             // %
    baseShear: number;            // kN
    overturningMoment: number;    // kN·m

    // Member forces
    maxColumnForce: number;       // kN
    maxBeamMoment: number;        // kN·m
    maxShellStress: number;       // MPa

    // Design checks
    passedMembers: number;
    failedMembers: number;
    utilizationRatio: number;     // %

    // File paths
    reportPaths: string[];
}

/**
 * Git commit info
 */
export interface GitCommit {
    hash: string;
    message: string;
    author: string;
    timestamp: string;
    files: string[];
}

// ============================================================================
// HELPER TYPES
// ============================================================================

export type CommandName =
    | 'create_project'
    | 'open_project'
    | 'get_project_state'
    | 'create_branch'
    | 'switch_branch'
    | 'list_branches'
    | 'delete_branch'
    | 'save_version'
    | 'list_versions'
    | 'checkout_version'
    | 'compare_versions'
    | 'open_in_etabs'
    | 'close_etabs'
    | 'get_etabs_status'
    | 'generate_e2k'
    | 'git_commit'
    | 'git_log'
    | 'git_diff';

// ============================================================================
// TYPE-SAFE INVOKE WRAPPER
// ============================================================================

/**
 * Type-safe wrapper for Tauri invoke
 * This ensures compile-time type checking
 */
export async function invokeCommand<T>(
    command: CommandName,
    args?: any
): Promise<TauriResult<T>> {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke(command, args);
}
