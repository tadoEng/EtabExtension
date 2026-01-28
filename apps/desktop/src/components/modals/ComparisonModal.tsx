import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { MonacoDiffViewer } from '@/components/editor/MonacoDiffViewer';
import {
    X,
    GitCompare,
    FileText,
    Box,
    Loader2,
    Download,
    AlertCircle
} from 'lucide-react';
import type { TauriResult, E2KDiffResult, GeometryDiffResult } from '@/types/tauri-commands';

interface ComparisonModalProps {
    version1: { branch: string; versionId: string; label: string };
    version2: { branch: string; versionId: string; label: string };
    projectPath: string;
    onClose: () => void;
}

export function ComparisonModal({
                                    version1,
                                    version2,
                                    projectPath,
                                    onClose
                                }: ComparisonModalProps) {
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [e2kDiff, setE2kDiff] = useState<E2KDiffResult | null>(null);
    const [geometryDiff, setGeometryDiff] = useState<GeometryDiffResult | null>(null);

    const loadComparison = async (diffType: 'e2k' | 'geometry' | 'both') => {
        setLoading(true);
        setError(null);

        try {
            const result = await invoke<TauriResult<{
                e2kDiff?: E2KDiffResult;
                geometryDiff?: GeometryDiffResult;
            }>>('compare_versions', {
                projectPath,
                version1: {
                    branch: version1.branch,
                    versionId: version1.versionId
                },
                version2: {
                    branch: version2.branch,
                    versionId: version2.versionId
                },
                diffType
            });

            if (result.success && result.data) {
                if (result.data.e2kDiff) setE2kDiff(result.data.e2kDiff);
                if (result.data.geometryDiff) setGeometryDiff(result.data.geometryDiff);
            } else {
                setError(result.error || 'Failed to load comparison');
            }
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Unknown error');
        } finally {
            setLoading(false);
        }
    };

    // Auto-load E2K diff on mount
    useState(() => {
        loadComparison('e2k');
    });

    return (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
            <Card className="w-full max-w-6xl h-[90vh] flex flex-col">
                {/* Header */}
                <CardHeader className="border-b">
                    <div className="flex items-start justify-between">
                        <div className="flex-1">
                            <CardTitle className="text-lg flex items-center gap-2">
                                <GitCompare className="w-5 h-5 text-primary" />
                                Version Comparison
                            </CardTitle>
                            <div className="flex items-center gap-4 mt-2 text-sm">
                                <div className="flex items-center gap-2">
                                    <Badge variant="outline">{version1.label}</Badge>
                                    <span className="text-muted-foreground">vs</span>
                                    <Badge variant="outline">{version2.label}</Badge>
                                </div>
                            </div>
                        </div>
                        <Button
                            variant="ghost"
                            size="icon"
                            onClick={onClose}
                        >
                            <X className="w-4 h-4" />
                        </Button>
                    </div>
                </CardHeader>

                {/* Content */}
                <CardContent className="flex-1 overflow-hidden p-0">
                    {loading ? (
                        <div className="h-full flex items-center justify-center">
                            <div className="text-center">
                                <Loader2 className="w-8 h-8 animate-spin mx-auto text-primary mb-4" />
                                <p className="text-muted-foreground">Loading comparison...</p>
                            </div>
                        </div>
                    ) : error ? (
                        <div className="h-full flex items-center justify-center">
                            <div className="text-center max-w-md">
                                <AlertCircle className="w-12 h-12 mx-auto text-destructive mb-4" />
                                <p className="text-destructive font-medium mb-2">Comparison Failed</p>
                                <p className="text-sm text-muted-foreground mb-4">{error}</p>
                                <Button onClick={() => loadComparison('e2k')}>
                                    Try Again
                                </Button>
                            </div>
                        </div>
                    ) : (
                        <Tabs defaultValue="e2k" className="h-full flex flex-col">
                            <div className="border-b px-6 pt-4">
                                <TabsList>
                                    <TabsTrigger
                                        value="e2k"
                                        onClick={() => !e2kDiff && loadComparison('e2k')}
                                    >
                                        <FileText className="w-4 h-4 mr-2" />
                                        E2K Changes
                                    </TabsTrigger>
                                    <TabsTrigger
                                        value="geometry"
                                        onClick={() => !geometryDiff && loadComparison('geometry')}
                                    >
                                        <Box className="w-4 h-4 mr-2" />
                                        3D Geometry
                                    </TabsTrigger>
                                    <TabsTrigger value="summary">
                                        <GitCompare className="w-4 h-4 mr-2" />
                                        Summary
                                    </TabsTrigger>
                                </TabsList>
                            </div>

                            {/* E2K Diff Tab */}
                            <TabsContent value="e2k" className="flex-1 overflow-hidden m-0">
                                {e2kDiff ? (
                                    <div className="h-full flex flex-col">
                                        {/* Stats Bar */}
                                        <div className="border-b p-4 bg-muted/30">
                                            <div className="flex items-center gap-6 text-sm">
                                                <div className="flex items-center gap-2">
                          <span className="text-green-600 font-medium">
                            +{e2kDiff.added}
                          </span>
                                                    <span className="text-muted-foreground">added</span>
                                                </div>
                                                <div className="flex items-center gap-2">
                          <span className="text-red-600 font-medium">
                            -{e2kDiff.removed}
                          </span>
                                                    <span className="text-muted-foreground">removed</span>
                                                </div>
                                                <div className="flex items-center gap-2">
                          <span className="text-yellow-600 font-medium">
                            ~{e2kDiff.modified}
                          </span>
                                                    <span className="text-muted-foreground">modified</span>
                                                </div>
                                                <div className="flex-1" />
                                                <Button variant="outline" size="sm">
                                                    <Download className="w-3 h-3 mr-2" />
                                                    Export Diff
                                                </Button>
                                            </div>
                                        </div>

                                        {/* Diff Viewer */}
                                        <div className="flex-1 overflow-hidden">
                                            <MonacoDiffViewer
                                                original={e2kDiff.rawDiff.split('\n--- ')[0]}
                                                modified={e2kDiff.rawDiff.split('\n+++ ')[1] || ''}
                                            />
                                        </div>

                                        {/* Changes List */}
                                        <div className="border-t p-4 max-h-48 overflow-y-auto bg-muted/30">
                                            <h3 className="text-sm font-medium mb-3">Detailed Changes</h3>
                                            <div className="space-y-2">
                                                {e2kDiff.changes.map((change, idx) => (
                                                    <div
                                                        key={idx}
                                                        className="flex items-start gap-3 text-sm p-2 rounded bg-background"
                                                    >
                                                        <Badge
                                                            variant={
                                                                change.type === 'add' ? 'default' :
                                                                    change.type === 'remove' ? 'destructive' :
                                                                        'outline'
                                                            }
                                                            className="mt-0.5"
                                                        >
                                                            {change.type}
                                                        </Badge>
                                                        <div className="flex-1">
                                                            <div className="font-medium text-xs text-muted-foreground uppercase mb-1">
                                                                {change.category}
                                                            </div>
                                                            <div>{change.description}</div>
                                                            {change.oldValue && change.newValue && (
                                                                <div className="text-xs text-muted-foreground mt-1">
                                                                    <span className="text-red-600">{change.oldValue}</span>
                                                                    {' → '}
                                                                    <span className="text-green-600">{change.newValue}</span>
                                                                </div>
                                                            )}
                                                        </div>
                                                    </div>
                                                ))}
                                            </div>
                                        </div>
                                    </div>
                                ) : (
                                    <div className="h-full flex items-center justify-center">
                                        <p className="text-muted-foreground">Loading E2K comparison...</p>
                                    </div>
                                )}
                            </TabsContent>

                            {/* Geometry Diff Tab */}
                            <TabsContent value="geometry" className="flex-1 m-0">
                                {geometryDiff ? (
                                    <div className="h-full flex flex-col">
                                        <div className="border-b p-4 bg-muted/30">
                                            <div className="flex items-center gap-6 text-sm">
                                                <div className="flex items-center gap-2">
                          <span className="text-green-600 font-medium">
                            {geometryDiff.membersAdded.length}
                          </span>
                                                    <span className="text-muted-foreground">added</span>
                                                </div>
                                                <div className="flex items-center gap-2">
                          <span className="text-red-600 font-medium">
                            {geometryDiff.membersRemoved.length}
                          </span>
                                                    <span className="text-muted-foreground">removed</span>
                                                </div>
                                                <div className="flex items-center gap-2">
                          <span className="text-yellow-600 font-medium">
                            {geometryDiff.membersModified.length}
                          </span>
                                                    <span className="text-muted-foreground">modified</span>
                                                </div>
                                            </div>
                                        </div>

                                        {/* 3D Viewer would go here */}
                                        <div className="flex-1 flex items-center justify-center bg-muted/20">
                                            <div className="text-center">
                                                <Box className="w-16 h-16 mx-auto text-muted-foreground mb-4" />
                                                <p className="text-muted-foreground">3D geometry viewer</p>
                                                <p className="text-sm text-muted-foreground">
                                                    Interactive visualization coming soon
                                                </p>
                                            </div>
                                        </div>
                                    </div>
                                ) : (
                                    <div className="h-full flex items-center justify-center">
                                        <Button onClick={() => loadComparison('geometry')}>
                                            Load 3D Comparison
                                        </Button>
                                    </div>
                                )}
                            </TabsContent>

                            {/* Summary Tab */}
                            <TabsContent value="summary" className="flex-1 p-6 overflow-y-auto">
                                <div className="max-w-3xl mx-auto space-y-6">
                                    <Card>
                                        <CardHeader>
                                            <CardTitle className="text-base">Comparison Summary</CardTitle>
                                        </CardHeader>
                                        <CardContent className="space-y-4">
                                            <div className="grid grid-cols-2 gap-4">
                                                <div>
                                                    <h3 className="text-sm font-medium mb-2">Version 1</h3>
                                                    <div className="space-y-1 text-sm">
                                                        <div className="flex justify-between">
                                                            <span className="text-muted-foreground">Branch:</span>
                                                            <span className="font-medium">{version1.branch}</span>
                                                        </div>
                                                        <div className="flex justify-between">
                                                            <span className="text-muted-foreground">Version:</span>
                                                            <span className="font-medium">{version1.versionId}</span>
                                                        </div>
                                                    </div>
                                                </div>

                                                <div>
                                                    <h3 className="text-sm font-medium mb-2">Version 2</h3>
                                                    <div className="space-y-1 text-sm">
                                                        <div className="flex justify-between">
                                                            <span className="text-muted-foreground">Branch:</span>
                                                            <span className="font-medium">{version2.branch}</span>
                                                        </div>
                                                        <div className="flex justify-between">
                                                            <span className="text-muted-foreground">Version:</span>
                                                            <span className="font-medium">{version2.versionId}</span>
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>

                                            {e2kDiff && (
                                                <div className="border-t pt-4">
                                                    <h3 className="text-sm font-medium mb-2">E2K Changes</h3>
                                                    <div className="grid grid-cols-3 gap-4 text-sm">
                                                        <div className="text-center p-3 bg-green-50 dark:bg-green-950 rounded">
                                                            <div className="text-2xl font-bold text-green-600">
                                                                {e2kDiff.added}
                                                            </div>
                                                            <div className="text-xs text-muted-foreground">Added</div>
                                                        </div>
                                                        <div className="text-center p-3 bg-red-50 dark:bg-red-950 rounded">
                                                            <div className="text-2xl font-bold text-red-600">
                                                                {e2kDiff.removed}
                                                            </div>
                                                            <div className="text-xs text-muted-foreground">Removed</div>
                                                        </div>
                                                        <div className="text-center p-3 bg-yellow-50 dark:bg-yellow-950 rounded">
                                                            <div className="text-2xl font-bold text-yellow-600">
                                                                {e2kDiff.modified}
                                                            </div>
                                                            <div className="text-xs text-muted-foreground">Modified</div>
                                                        </div>
                                                    </div>
                                                </div>
                                            )}

                                            {geometryDiff && (
                                                <div className="border-t pt-4">
                                                    <h3 className="text-sm font-medium mb-2">Geometry Changes</h3>
                                                    <div className="text-sm">
                                                        <p className="text-muted-foreground">
                                                            Total changes: <span className="font-medium text-foreground">
                                {geometryDiff.totalChanges}
                              </span>
                                                        </p>
                                                    </div>
                                                </div>
                                            )}
                                        </CardContent>
                                    </Card>

                                    <Button className="w-full">
                                        <Download className="w-4 h-4 mr-2" />
                                        Generate Comparison Report
                                    </Button>
                                </div>
                            </TabsContent>
                        </Tabs>
                    )}
                </CardContent>
            </Card>
        </div>
    );
}