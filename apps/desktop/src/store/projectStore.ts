/**
 * Global state management for ETABS Extension
 * Path: src/store/projectStore.ts
 */

import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import { immer } from 'zustand/middleware/immer';
import { invoke } from '@tauri-apps/api/core';
import type {
    ProjectState,
    BranchInfo,
    VersionInfo,
    TauriResult,
    CreateBranchRequest,
    SaveVersionRequest,
    CheckoutVersionRequest,
    OpenInEtabsRequest
} from '@/types/tauri-commands.ts';

interface ProjectStore {
    // State
    currentProject: ProjectState | null;
    isLoading: boolean;
    error: string | null;

    // ETABS status
    etabsStatus: {
        isRunning: boolean;
        openFilePath?: string;
        canSave: boolean;
    };

    // Actions
    openProject: (projectPath: string) => Promise<void>;
    closeProject: () => void;
    refreshProjectState: () => Promise<void>;

    // Branch operations
    createBranch: (request: CreateBranchRequest) => Promise<void>;
    switchBranch: (branchName: string) => Promise<void>;
    deleteBranch: (branchName: string, forceDelete: boolean) => Promise<void>;

    // Version operations
    saveVersion: (request: SaveVersionRequest) => Promise<void>;
    checkoutVersion: (request: CheckoutVersionRequest) => Promise<void>;

    // ETABS operations
    openInEtabs: (branchName: string, versionId?: string) => Promise<void>;
    closeEtabs: (saveChanges: boolean) => Promise<void>;
    refreshEtabsStatus: () => Promise<void>;

    // Helpers
    getCurrentBranch: () => BranchInfo | null;
    getVersionsForBranch: (branchName: string) => VersionInfo[];
    setError: (error: string | null) => void;
}

export const useProjectStore = create<ProjectStore>()(
    devtools(
        immer((set, get) => ({
            // Initial state
            currentProject: null,
            isLoading: false,
            error: null,
            etabsStatus: {
                isRunning: false,
                canSave: false
            },

            // Open project
            openProject: async (projectPath: string) => {
                set({ isLoading: true, error: null });

                try {
                    const result = await invoke<TauriResult<ProjectState>>('open_project', {
                        projectPath
                    });

                    if (result.success && result.data) {
                        set({ currentProject: result.data, isLoading: false });
                        await get().refreshEtabsStatus();
                    } else {
                        set({
                            error: result.error || 'Failed to open project',
                            isLoading: false
                        });
                    }
                } catch (error) {
                    set({
                        error: error instanceof Error ? error.message : 'Unknown error',
                        isLoading: false
                    });
                }
            },

            // Close project
            closeProject: () => {
                set({ currentProject: null, error: null });
            },

            // Refresh project state
            refreshProjectState: async () => {
                const project = get().currentProject;
                if (!project) return;

                try {
                    const result = await invoke<TauriResult<ProjectState>>('get_project_state', {
                        projectPath: project.projectPath
                    });

                    if (result.success && result.data) {
                        set({ currentProject: result.data });
                    }
                } catch (error) {
                    console.error('Failed to refresh project state:', error);
                }
            },

            // Create branch
            createBranch: async (request: CreateBranchRequest) => {
                set({ isLoading: true, error: null });

                try {
                    const result = await invoke<TauriResult<any>>('create_branch', request);

                    if (result.success) {
                        await get().refreshProjectState();
                        set({ isLoading: false });
                    } else {
                        set({
                            error: result.error || 'Failed to create branch',
                            isLoading: false
                        });
                    }
                } catch (error) {
                    set({
                        error: error instanceof Error ? error.message : 'Unknown error',
                        isLoading: false
                    });
                }
            },

            // Switch branch
            switchBranch: async (branchName: string) => {
                const project = get().currentProject;
                if (!project) return;

                set({ isLoading: true, error: null });

                try {
                    const result = await invoke<TauriResult<any>>('switch_branch', {
                        projectPath: project.projectPath,
                        branchName,
                        closeCurrentFile: get().etabsStatus.isRunning
                    });

                    if (result.success) {
                        await get().refreshProjectState();
                        await get().refreshEtabsStatus();
                        set({ isLoading: false });
                    } else {
                        set({
                            error: result.error || 'Failed to switch branch',
                            isLoading: false
                        });
                    }
                } catch (error) {
                    set({
                        error: error instanceof Error ? error.message : 'Unknown error',
                        isLoading: false
                    });
                }
            },

            // Delete branch
            deleteBranch: async (branchName: string, forceDelete: boolean) => {
                const project = get().currentProject;
                if (!project) return;

                set({ isLoading: true, error: null });

                try {
                    const result = await invoke<TauriResult<any>>('delete_branch', {
                        projectPath: project.projectPath,
                        branchName,
                        forceDelete
                    });

                    if (result.success) {
                        await get().refreshProjectState();
                        set({ isLoading: false });
                    } else {
                        set({
                            error: result.error || 'Failed to delete branch',
                            isLoading: false
                        });
                    }
                } catch (error) {
                    set({
                        error: error instanceof Error ? error.message : 'Unknown error',
                        isLoading: false
                    });
                }
            },

            // Save version
            saveVersion: async (request: SaveVersionRequest) => {
                set({ isLoading: true, error: null });

                try {
                    const result = await invoke<TauriResult<any>>('save_version', request);

                    if (result.success) {
                        await get().refreshProjectState();
                        set({ isLoading: false });
                    } else {
                        set({
                            error: result.error || 'Failed to save version',
                            isLoading: false
                        });
                    }
                } catch (error) {
                    set({
                        error: error instanceof Error ? error.message : 'Unknown error',
                        isLoading: false
                    });
                }
            },

            // Checkout version
            checkoutVersion: async (request: CheckoutVersionRequest) => {
                set({ isLoading: true, error: null });

                try {
                    const result = await invoke<TauriResult<any>>('checkout_version', request);

                    if (result.success) {
                        await get().refreshProjectState();
                        if (request.openInEtabs) {
                            await get().refreshEtabsStatus();
                        }
                        set({ isLoading: false });
                    } else {
                        set({
                            error: result.error || 'Failed to checkout version',
                            isLoading: false
                        });
                    }
                } catch (error) {
                    set({
                        error: error instanceof Error ? error.message : 'Unknown error',
                        isLoading: false
                    });
                }
            },

            // Open in ETABS
            openInEtabs: async (branchName: string, versionId?: string) => {
                const project = get().currentProject;
                if (!project) return;

                set({ isLoading: true, error: null });

                try {
                    const result = await invoke<TauriResult<any>>('open_in_etabs', {
                        projectPath: project.projectPath,
                        branchName,
                        versionId
                    } as OpenInEtabsRequest);

                    if (result.success) {
                        await get().refreshEtabsStatus();
                        set({ isLoading: false });
                    } else {
                        set({
                            error: result.error || 'Failed to open in ETABS',
                            isLoading: false
                        });
                    }
                } catch (error) {
                    set({
                        error: error instanceof Error ? error.message : 'Unknown error',
                        isLoading: false
                    });
                }
            },

            // Close ETABS
            closeEtabs: async (saveChanges: boolean) => {
                set({ isLoading: true, error: null });

                try {
                    const result = await invoke<TauriResult<any>>('close_etabs', {
                        saveChanges
                    });

                    if (result.success) {
                        await get().refreshEtabsStatus();
                        if (saveChanges) {
                            await get().refreshProjectState();
                        }
                        set({ isLoading: false });
                    } else {
                        set({
                            error: result.error || 'Failed to close ETABS',
                            isLoading: false
                        });
                    }
                } catch (error) {
                    set({
                        error: error instanceof Error ? error.message : 'Unknown error',
                        isLoading: false
                    });
                }
            },

            // Refresh ETABS status
            refreshEtabsStatus: async () => {
                const project = get().currentProject;
                if (!project) return;

                try {
                    const result = await invoke<TauriResult<any>>('get_etabs_status', {
                        projectPath: project.projectPath
                    });

                    if (result.success && result.data) {
                        set({
                            etabsStatus: {
                                isRunning: result.data.isRunning,
                                openFilePath: result.data.openFilePath,
                                canSave: result.data.canSave
                            }
                        });
                    }
                } catch (error) {
                    console.error('Failed to refresh ETABS status:', error);
                }
            },

            // Get current branch
            getCurrentBranch: () => {
                const project = get().currentProject;
                if (!project) return null;

                return project.branches[project.currentBranch] || null;
            },

            // Get versions for branch
            getVersionsForBranch: (branchName: string) => {
                const project = get().currentProject;
                if (!project) return [];

                const branch = project.branches[branchName];
                return branch?.versions || [];
            },

            // Set error
            setError: (error: string | null) => {
                set({ error });
            }
        }))
    )
);

// Selectors
export const selectCurrentBranch = (state: ProjectStore) => state.getCurrentBranch();
export const selectIsLoading = (state: ProjectStore) => state.isLoading;
export const selectError = (state: ProjectStore) => state.error;
export const selectEtabsStatus = (state: ProjectStore) => state.etabsStatus;