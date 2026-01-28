import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { GitBranch, Plus, GitMerge, Trash2, AlertCircle, Loader2 } from 'lucide-react';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { useProjectStore } from '@/store/projectStore';

export function BranchesPanel() {
    const {
        currentProject,
        isLoading,
        error,
        createBranch,
        deleteBranch,
        setError
    } = useProjectStore();

    const [formData, setFormData] = useState({
        branchName: '',
        fromBranch: 'main',
        fromVersion: '',
        description: ''
    });

    const [deleteConfirm, setDeleteConfirm] = useState<string | null>(null);

    const branches = currentProject ? Object.entries(currentProject.branches) : [];

    // Get available versions for selected parent branch
    const availableVersions = currentProject && formData.fromBranch
        ? currentProject.branches[formData.fromBranch]?.versions || []
        : [];

    const handleCreateBranch = async () => {
        if (!currentProject || !formData.branchName.trim() || !formData.fromVersion) {
            return;
        }

        await createBranch({
            projectPath: currentProject.projectPath,
            branchName: formData.branchName.trim(),
            fromBranch: formData.fromBranch,
            fromVersion: formData.fromVersion,
            description: formData.description.trim() || undefined
        });

        // Reset form on success
        if (!error) {
            setFormData({
                branchName: '',
                fromBranch: 'main',
                fromVersion: '',
                description: ''
            });
        }
    };

    const handleDeleteBranch = async (branchName: string, forceDelete: boolean = false) => {
        if (!currentProject) return;

        await deleteBranch(branchName, forceDelete);
        setDeleteConfirm(null);
    };

    if (!currentProject) {
        return (
            <div className="h-full flex items-center justify-center">
                <div className="text-center max-w-md">
                    <GitBranch className="w-16 h-16 mx-auto text-muted-foreground mb-4" />
                    <h2 className="text-xl font-semibold mb-2">No Project Open</h2>
                    <p className="text-muted-foreground mb-6">
                        Open a project to manage branches
                    </p>
                </div>
            </div>
        );
    }

    return (
        <div className="h-full overflow-y-auto">
            <div className="max-w-6xl mx-auto p-6 space-y-6">
                {/* Header */}
                <div className="flex items-center justify-between">
                    <div>
                        <h1 className="text-2xl font-bold">Branches</h1>
                        <p className="text-muted-foreground mt-1">
                            Manage design alternatives and explorations
                        </p>
                    </div>
                </div>

                {/* Error Display */}
                {error && (
                    <Alert variant="destructive">
                        <AlertCircle className="h-4 w-4" />
                        <AlertDescription className="flex items-center justify-between">
                            <span>{error}</span>
                            <Button variant="ghost" size="sm" onClick={() => setError(null)}>
                                Dismiss
                            </Button>
                        </AlertDescription>
                    </Alert>
                )}

                {/* Loading Overlay */}
                {isLoading && (
                    <div className="fixed inset-0 bg-background/80 flex items-center justify-center z-40">
                        <div className="text-center">
                            <Loader2 className="w-8 h-8 animate-spin mx-auto text-primary mb-4" />
                            <p className="text-sm text-muted-foreground">Processing...</p>
                        </div>
                    </div>
                )}

                {/* Create Branch Card */}
                <Card>
                    <CardHeader>
                        <CardTitle className="text-base flex items-center gap-2">
                            <Plus className="w-4 h-4" />
                            Create New Branch
                        </CardTitle>
                        <CardDescription>
                            Start a new design exploration based on an existing version
                        </CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        <div className="grid grid-cols-2 gap-4">
                            <div className="space-y-2">
                                <Label htmlFor="branchName">Branch Name *</Label>
                                <Input
                                    id="branchName"
                                    placeholder="e.g., cost-reduction"
                                    value={formData.branchName}
                                    onChange={(e) => setFormData({
                                        ...formData,
                                        branchName: e.target.value
                                    })}
                                />
                                <p className="text-xs text-muted-foreground">
                                    Use lowercase with hyphens (no spaces)
                                </p>
                            </div>

                            <div className="space-y-2">
                                <Label htmlFor="fromBranch">Based On Branch *</Label>
                                <select
                                    id="fromBranch"
                                    value={formData.fromBranch}
                                    onChange={(e) => setFormData({
                                        ...formData,
                                        fromBranch: e.target.value,
                                        fromVersion: '' // Reset version when branch changes
                                    })}
                                    className="w-full h-9 rounded-md border border-input bg-background px-3 text-sm"
                                >
                                    {branches.map(([branchName]) => (
                                        <option key={branchName} value={branchName}>
                                            {branchName}
                                        </option>
                                    ))}
                                </select>
                            </div>
                        </div>

                        <div className="space-y-2">
                            <Label htmlFor="fromVersion">Version *</Label>
                            <select
                                id="fromVersion"
                                value={formData.fromVersion}
                                onChange={(e) => setFormData({
                                    ...formData,
                                    fromVersion: e.target.value
                                })}
                                className="w-full h-9 rounded-md border border-input bg-background px-3 text-sm"
                                disabled={!formData.fromBranch}
                            >
                                <option value="">Select version...</option>
                                {availableVersions.map((version) => (
                                    <option key={version.id} value={version.id}>
                                        {version.id} - {version.message}
                                    </option>
                                ))}
                            </select>
                        </div>

                        <div className="space-y-2">
                            <Label htmlFor="description">Description (optional)</Label>
                            <Input
                                id="description"
                                placeholder="What are you exploring in this branch?"
                                value={formData.description}
                                onChange={(e) => setFormData({
                                    ...formData,
                                    description: e.target.value
                                })}
                            />
                        </div>

                        <Button
                            className="w-full"
                            onClick={handleCreateBranch}
                            disabled={!formData.branchName.trim() || !formData.fromVersion || isLoading}
                        >
                            <GitBranch className="w-4 h-4 mr-2" />
                            Create Branch
                        </Button>
                    </CardContent>
                </Card>

                {/* Branches List */}
                <div className="space-y-3">
                    <h2 className="text-sm font-semibold text-muted-foreground uppercase">
                        All Branches ({branches.length})
                    </h2>

                    {branches.map(([branchName, branchData]) => {
                        const isMainBranch = branchName === 'main';
                        const isCurrentBranch = branchName === currentProject.currentBranch;
                        const showingDeleteConfirm = deleteConfirm === branchName;

                        return (
                            <Card key={branchName}>
                                <CardContent className="p-4">
                                    <div className="flex items-start justify-between">
                                        <div className="flex-1">
                                            <div className="flex items-center gap-2">
                                                <GitBranch className="w-4 h-4 text-muted-foreground" />
                                                <span className="font-semibold">{branchName}</span>
                                                {isMainBranch && <Badge variant="default">Main</Badge>}
                                                {isCurrentBranch && <Badge variant="outline">Active</Badge>}
                                            </div>

                                            {branchData.description && (
                                                <p className="text-sm text-muted-foreground mt-2">
                                                    {branchData.description}
                                                </p>
                                            )}

                                            <div className="flex items-center gap-4 mt-3 text-xs text-muted-foreground">
                                                <span>{branchData.versions.length} version{branchData.versions.length !== 1 ? 's' : ''}</span>
                                                <span>•</span>
                                                <span>Latest: {branchData.latestVersion}</span>
                                                {branchData.parentBranch && (
                                                    <>
                                                        <span>•</span>
                                                        <span>From {branchData.parentBranch}/{branchData.parentVersion}</span>
                                                    </>
                                                )}
                                            </div>

                                            {branchData.created && (
                                                <div className="text-xs text-muted-foreground mt-1">
                                                    Created {new Date(branchData.created).toLocaleDateString()}
                                                </div>
                                            )}
                                        </div>

                                        <div className="flex gap-2">
                                            {!isMainBranch && (
                                                <>
                                                    {showingDeleteConfirm ? (
                                                        <div className="flex gap-2">
                                                            <Button
                                                                variant="destructive"
                                                                size="sm"
                                                                onClick={() => handleDeleteBranch(branchName, false)}
                                                            >
                                                                Confirm Delete
                                                            </Button>
                                                            <Button
                                                                variant="outline"
                                                                size="sm"
                                                                onClick={() => setDeleteConfirm(null)}
                                                            >
                                                                Cancel
                                                            </Button>
                                                        </div>
                                                    ) : (
                                                        <>
                                                            <Button variant="outline" size="sm">
                                                                <GitMerge className="w-3 h-3 mr-2" />
                                                                Merge
                                                            </Button>
                                                            <Button
                                                                variant="ghost"
                                                                size="icon-sm"
                                                                onClick={() => setDeleteConfirm(branchName)}
                                                            >
                                                                <Trash2 className="w-3 h-3 text-destructive" />
                                                            </Button>
                                                        </>
                                                    )}
                                                </>
                                            )}
                                        </div>
                                    </div>

                                    {showingDeleteConfirm && (
                                        <Alert variant="destructive" className="mt-4">
                                            <AlertCircle className="h-4 w-4" />
                                            <AlertDescription>
                                                This will permanently delete the branch and all its versions. This action cannot be undone.
                                            </AlertDescription>
                                        </Alert>
                                    )}
                                </CardContent>
                            </Card>
                        );
                    })}

                    {branches.length === 0 && (
                        <Card>
                            <CardContent className="p-12 text-center">
                                <GitBranch className="w-12 h-12 mx-auto text-muted-foreground mb-4" />
                                <p className="text-muted-foreground">No branches yet</p>
                                <p className="text-sm text-muted-foreground mt-1">
                                    Create your first branch to get started
                                </p>
                            </CardContent>
                        </Card>
                    )}
                </div>
            </div>
        </div>
    );
}