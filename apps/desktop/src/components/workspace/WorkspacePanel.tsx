import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import {
    FolderOpen,
    GitBranch,
    Plus,
    ExternalLink,
    Save,
    GitMerge,
    Lock,
    Unlock,
    FileCode,
    Loader2,
    AlertCircle,
    CheckCircle2,
    X
} from 'lucide-react';
import { useProjectStore } from '@/store/projectStore';
import { open } from '@tauri-apps/plugin-dialog';
import { Alert, AlertDescription } from '@/components/ui/alert';

// Save Version Modal
function SaveVersionModal({
                              onClose,
                              onSave
                          }: {
    onClose: () => void;
    onSave: (message: string, generateE2k: boolean) => void;
}) {
    const [message, setMessage] = useState('');
    const [generateE2k, setGenerateE2k] = useState(true);

    const handleSubmit = () => {
        if (message.trim()) {
            onSave(message, generateE2k);
            onClose();
        }
    };

    return (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
            <Card className="w-full max-w-md">
                <CardHeader>
                    <div className="flex items-center justify-between">
                        <CardTitle className="text-lg">Save New Version</CardTitle>
                        <Button variant="ghost" size="icon" onClick={onClose}>
                            <X className="w-4 h-4" />
                        </Button>
                    </div>
                    <CardDescription>
                        Create a new version of the current design
                    </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                    <div className="space-y-2">
                        <Label htmlFor="message">Commit Message *</Label>
                        <Input
                            id="message"
                            value={message}
                            onChange={(e) => setMessage(e.target.value)}
                            placeholder="Describe what changed..."
                            onKeyDown={(e) => e.key === 'Enter' && handleSubmit()}
                            autoFocus
                        />
                    </div>

                    <div className="flex items-center justify-between">
                        <Label htmlFor="e2k">Generate E2K file</Label>
                        <input
                            type="checkbox"
                            id="e2k"
                            checked={generateE2k}
                            onChange={(e) => setGenerateE2k(e.target.checked)}
                            className="w-4 h-4"
                        />
                    </div>

                    <div className="flex gap-2 pt-2">
                        <Button variant="outline" onClick={onClose} className="flex-1">
                            Cancel
                        </Button>
                        <Button onClick={handleSubmit} disabled={!message.trim()} className="flex-1">
                            <Save className="w-4 h-4 mr-2" />
                            Save Version
                        </Button>
                    </div>
                </CardContent>
            </Card>
        </div>
    );
}

export function WorkspacePanel() {
    const {
        currentProject,
        isLoading,
        error,
        etabsStatus,
        openProject,
        switchBranch,
        saveVersion,
        openInEtabs,
        closeEtabs,
        getCurrentBranch,
        setError
    } = useProjectStore();

    const [showSaveModal, setShowSaveModal] = useState(false);

    const currentBranch = getCurrentBranch();
    const branches = currentProject ? Object.entries(currentProject.branches) : [];

    const handleOpenProject = async () => {
        const selected = await open({
            directory: true,
            title: 'Select ETABS Project Folder',
            multiple: false
        });

        if (selected && typeof selected === 'string') {
            await openProject(selected);
        }
    };

    const handleSaveVersion = async (message: string, generateE2k: boolean) => {
        if (!currentProject || !currentBranch) return;

        await saveVersion({
            projectPath: currentProject.projectPath,
            branchName: currentBranch.name,
            message,
            generateE2k
        });
    };

    const handleOpenInEtabs = async () => {
        if (!currentBranch) return;
        await openInEtabs(currentBranch.name);
    };

    const handleCloseEtabs = async (saveChanges: boolean) => {
        await closeEtabs(saveChanges);
    };

    if (!currentProject) {
        return (
            <div className="h-full flex items-center justify-center">
                <div className="text-center max-w-md">
                    <FolderOpen className="w-16 h-16 mx-auto text-muted-foreground mb-4" />
                    <h2 className="text-xl font-semibold mb-2">No Project Open</h2>
                    <p className="text-muted-foreground mb-6">
                        Open an existing ETABS project to start managing versions
                    </p>
                    <Button onClick={handleOpenProject} size="lg">
                        <FolderOpen className="w-4 h-4 mr-2" />
                        Open Project
                    </Button>
                </div>
            </div>
        );
    }

    return (
        <div className="h-full flex flex-col">
            {error && (
                <Alert variant="destructive" className="m-4">
                    <AlertCircle className="h-4 w-4" />
                    <AlertDescription className="flex items-center justify-between">
                        <span>{error}</span>
                        <Button variant="ghost" size="sm" onClick={() => setError(null)}>
                            Dismiss
                        </Button>
                    </AlertDescription>
                </Alert>
            )}

            {isLoading && (
                <div className="absolute inset-0 bg-background/80 flex items-center justify-center z-40">
                    <div className="text-center">
                        <Loader2 className="w-8 h-8 animate-spin mx-auto text-primary mb-4" />
                        <p className="text-sm text-muted-foreground">Loading...</p>
                    </div>
                </div>
            )}

            <div className="flex-1 flex overflow-hidden">
                {/* Left Panel - Branches */}
                <div className="w-80 border-r border-border/40 bg-background/50 overflow-y-auto">
                    <div className="p-4 border-b border-border/40">
                        <div className="flex items-center justify-between mb-3">
                            <h2 className="text-sm font-semibold text-muted-foreground uppercase">
                                Branches
                            </h2>
                            <Button size="icon-sm" variant="ghost">
                                <Plus className="w-4 h-4" />
                            </Button>
                        </div>

                        <div className="p-3 rounded-lg bg-muted/50 mb-3">
                            <div className="text-sm font-medium truncate">
                                {currentProject.projectName}
                            </div>
                            <div className="text-xs text-muted-foreground mt-1">
                                {branches.length} branch{branches.length !== 1 ? 'es' : ''}
                            </div>
                        </div>

                        <Button
                            variant="outline"
                            className="w-full justify-start gap-2"
                            size="sm"
                            onClick={handleOpenProject}
                        >
                            <FolderOpen className="w-4 h-4" />
                            Open Different Project
                        </Button>
                    </div>

                    <div className="p-2">
                        {branches.map(([branchName, branchData]) => {
                            const isActive = branchName === currentProject.currentBranch;
                            const isMainBranch = branchName === 'main';

                            return (
                                <button
                                    key={branchName}
                                    onClick={() => !isActive && switchBranch(branchName)}
                                    disabled={isActive}
                                    className={`w-full text-left p-3 rounded-lg mb-1 transition-colors ${
                                        isActive
                                            ? 'bg-primary/10 border border-primary/30'
                                            : 'hover:bg-accent border border-transparent'
                                    }`}
                                >
                                    <div className="flex items-start gap-2">
                                        <GitBranch className={`w-4 h-4 mt-0.5 ${
                                            isActive ? 'text-primary' : 'text-muted-foreground'
                                        }`} />
                                        <div className="flex-1 min-w-0">
                                            <div className="flex items-center gap-2">
                        <span className={`text-sm font-medium ${
                            isActive ? 'text-primary' : 'text-foreground'
                        }`}>
                          {branchName}
                        </span>
                                                {isActive && <Badge variant="default" className="text-xs">Active</Badge>}
                                                {isMainBranch && <Badge variant="outline" className="text-xs">Main</Badge>}
                                            </div>
                                            <div className="text-xs text-muted-foreground mt-1">
                                                {branchData.versions.length} version{branchData.versions.length !== 1 ? 's' : ''}
                                                {' • '}Latest: {branchData.latestVersion}
                                            </div>
                                            {branchData.parentBranch && (
                                                <div className="text-xs text-muted-foreground mt-1">
                                                    From {branchData.parentBranch}/{branchData.parentVersion}
                                                </div>
                                            )}
                                            {branchData.workingFile?.isOpen && (
                                                <div className="flex items-center gap-1 mt-1">
                                                    <Unlock className="w-3 h-3 text-green-500" />
                                                    <span className="text-xs text-green-600">Open in ETABS</span>
                                                </div>
                                            )}
                                        </div>
                                    </div>
                                </button>
                            );
                        })}
                    </div>
                </div>

                {/* Middle Panel - Versions */}
                <div className="w-96 border-r border-border/40 bg-background/50 overflow-y-auto">
                    <div className="p-4 border-b border-border/40">
                        <div className="flex items-center justify-between mb-2">
                            <div>
                                <h2 className="text-sm font-semibold flex items-center gap-2">
                                    <GitBranch className="w-4 h-4 text-primary" />
                                    {currentProject.currentBranch}
                                </h2>
                                {currentBranch?.parentBranch && (
                                    <p className="text-xs text-muted-foreground mt-1">
                                        From {currentBranch.parentBranch}/{currentBranch.parentVersion}
                                    </p>
                                )}
                            </div>
                        </div>

                        {etabsStatus.isRunning ? (
                            <Alert className="mb-3 border-green-200 bg-green-50 dark:border-green-900 dark:bg-green-950">
                                <CheckCircle2 className="h-4 w-4 text-green-600 dark:text-green-400" />
                                <AlertDescription className="text-green-900 dark:text-green-100">
                                    <div className="flex items-center justify-between">
                                        <span className="text-sm">ETABS is open</span>
                                        <Button variant="ghost" size="sm" onClick={() => handleCloseEtabs(true)}>
                                            Close & Save
                                        </Button>
                                    </div>
                                </AlertDescription>
                            </Alert>
                        ) : (
                            <div className="flex gap-2 mb-3">
                                <Button size="sm" variant="default" className="flex-1" onClick={handleOpenInEtabs}>
                                    <ExternalLink className="w-3 h-3 mr-2" />
                                    Open in ETABS
                                </Button>
                                <Button
                                    size="sm"
                                    variant="outline"
                                    onClick={() => setShowSaveModal(true)}
                                    disabled={!currentBranch?.workingFile?.exists}
                                >
                                    <Save className="w-3 h-3 mr-2" />
                                    Save Version
                                </Button>
                            </div>
                        )}
                    </div>

                    <div className="p-2">
                        <div className="text-xs font-semibold text-muted-foreground uppercase px-2 py-2">
                            Versions
                        </div>

                        {currentBranch?.versions.map((version, idx) => (
                            <div
                                key={version.id}
                                className="p-3 rounded-lg mb-1 hover:bg-accent border border-transparent hover:border-border/50 transition-colors cursor-pointer"
                            >
                                <div className="flex items-start justify-between">
                                    <div className="flex-1 min-w-0">
                                        <div className="flex items-center gap-2">
                                            <span className="text-sm font-medium">{version.id}</span>
                                            {idx === 0 && <Badge variant="default" className="text-xs">Latest</Badge>}
                                            {version.analyzed && (
                                                <Badge variant="outline" className="text-xs text-green-600 border-green-600">
                                                    Analyzed
                                                </Badge>
                                            )}
                                        </div>
                                        <p className="text-xs text-muted-foreground mt-1 line-clamp-2">
                                            {version.message}
                                        </p>
                                        <div className="flex items-center gap-2 text-xs text-muted-foreground mt-2">
                                            <span>{new Date(version.timestamp).toLocaleDateString()}</span>
                                            {version.author && (
                                                <>
                                                    <span>•</span>
                                                    <span>{version.author}</span>
                                                </>
                                            )}
                                        </div>
                                        {version.e2kPath && (
                                            <div className="flex items-center gap-1 mt-1">
                                                <FileCode className="w-3 h-3 text-muted-foreground" />
                                                <span className="text-xs text-muted-foreground">E2K available</span>
                                            </div>
                                        )}
                                    </div>
                                </div>
                            </div>
                        ))}

                        {currentBranch?.workingFile?.exists && (
                            <div className="p-3 rounded-lg mb-1 bg-primary/5 border border-primary/20">
                                <div className="flex items-center gap-2">
                                    <FileCode className="w-4 h-4 text-primary" />
                                    <div className="flex-1">
                                        <span className="text-sm font-medium text-primary">Working File</span>
                                        {currentBranch.workingFile.sourceVersion && (
                                            <p className="text-xs text-muted-foreground mt-0.5">
                                                Based on {currentBranch.workingFile.sourceVersion}
                                            </p>
                                        )}
                                        {currentBranch.workingFile.hasUnsavedChanges && (
                                            <p className="text-xs text-yellow-600 mt-0.5">Unsaved changes</p>
                                        )}
                                    </div>
                                    {currentBranch.workingFile.isOpen ? (
                                        <Unlock className="w-4 h-4 text-green-500" />
                                    ) : (
                                        <Lock className="w-4 h-4 text-muted-foreground" />
                                    )}
                                </div>
                            </div>
                        )}

                        {currentBranch?.versions.length === 0 && !currentBranch.workingFile?.exists && (
                            <div className="p-8 text-center">
                                <GitBranch className="w-12 h-12 mx-auto text-muted-foreground mb-3" />
                                <p className="text-sm text-muted-foreground">No versions yet</p>
                                <p className="text-xs text-muted-foreground mt-1">
                                    Create your first version to get started
                                </p>
                            </div>
                        )}
                    </div>

                    {currentProject.currentBranch !== 'main' && (
                        <div className="p-4 border-t border-border/40 mt-auto">
                            <Button variant="outline" size="sm" className="w-full">
                                <GitMerge className="w-3 h-3 mr-2" />
                                Merge to Main
                            </Button>
                        </div>
                    )}
                </div>

                {/* Right Panel - Details */}
                <div className="flex-1 overflow-y-auto bg-background">
                    <div className="p-6">
                        <Card>
                            <CardHeader>
                                <CardTitle className="text-base">Version Details</CardTitle>
                                <CardDescription>
                                    Select a version to view details and changes
                                </CardDescription>
                            </CardHeader>
                            <CardContent>
                                <div className="text-sm text-muted-foreground text-center py-12">
                                    <FileCode className="w-12 h-12 mx-auto text-muted-foreground mb-4" />
                                    <p>No version selected</p>
                                    <p className="text-xs mt-1">Click on a version to see its details</p>
                                </div>
                            </CardContent>
                        </Card>
                    </div>
                </div>
            </div>

            {showSaveModal && (
                <SaveVersionModal
                    onClose={() => setShowSaveModal(false)}
                    onSave={handleSaveVersion}
                />
            )}
        </div>
    );
}