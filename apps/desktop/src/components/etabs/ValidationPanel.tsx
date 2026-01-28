import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '@/components/ui/button.tsx';
import { Input } from '@/components/ui/input.tsx';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card.tsx';
import { Alert, AlertDescription } from '@/components/ui/alert.tsx';
import { Badge } from '@/components/ui/badge.tsx';
import { CheckCircle2, XCircle, AlertCircle, Loader2, FileCheck } from 'lucide-react';

// Type definitions matching your C# CLI output
interface ValidationData {
    etabsInstalled: boolean;
    etabsVersion?: string;
    fileValid?: boolean;
    filePath?: string;
    fileExists?: boolean;
    fileExtension?: string;
    isAnalyzed?: boolean;
    validationMessages: string[];
}

interface CliResult<T> {
    success: boolean;
    error?: string;
    timestamp: string;
    data?: T;
}

export function ValidationPanel() {
    const [filePath, setFilePath] = useState('');
    const [result, setResult] = useState<CliResult<ValidationData> | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const handleValidate = async () => {
        if (!filePath.trim()) {
            setError('Please enter a file path');
            return;
        }

        setLoading(true);
        setError(null);
        setResult(null);

        try {
            // Call Rust command which returns CliResult<ValidationData>
            const response = await invoke<CliResult<ValidationData>>('validate_etabs_file', {
                filePath: filePath.trim()
            });

            setResult(response);

            if (!response.success) {
                setError(response.error || 'Validation failed');
            }
        } catch (err) {
            const errorMessage = err instanceof Error ? err.message : String(err);
            setError(`Failed to execute command: ${errorMessage}`);
        } finally {
            setLoading(false);
        }
    };

    const getStatusIcon = (success: boolean) => {
        return success ? (
            <CheckCircle2 className="h-5 w-5 text-green-500" />
        ) : (
            <XCircle className="h-5 w-5 text-red-500" />
        );
    };

    return (
        <Card className="w-full">
            <CardHeader>
                <CardTitle className="flex items-center gap-2">
                    <FileCheck className="h-5 w-5" />
                    ETABS File Validation
                </CardTitle>
                <CardDescription>
                    Validate ETABS installation and check file status
                </CardDescription>
            </CardHeader>

            <CardContent className="space-y-4">
                {/* Input Section */}
                <div className="space-y-2">
                    <label className="text-sm font-medium">File Path</label>
                    <div className="flex gap-2">
                        <Input
                            type="text"
                            value={filePath}
                            onChange={(e) => setFilePath(e.target.value)}
                            placeholder="D:\path\to\model.edb"
                            disabled={loading}
                            onKeyDown={(e) => e.key === 'Enter' && handleValidate()}
                        />
                        <Button
                            onClick={handleValidate}
                            disabled={loading || !filePath.trim()}
                        >
                            {loading ? (
                                <>
                                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                                    Validating...
                                </>
                            ) : (
                                'Validate'
                            )}
                        </Button>
                    </div>
                </div>

                {/* Error Display */}
                {error && (
                    <Alert variant="destructive">
                        <AlertCircle className="h-4 w-4" />
                        <AlertDescription>{error}</AlertDescription>
                    </Alert>
                )}

                {/* Results Display */}
                {result && result.data && (
                    <div className="space-y-4 mt-6">
                        {/* Status Badge */}
                        <div className="flex items-center gap-2">
                            {getStatusIcon(result.success)}
                            <Badge variant={result.success ? "default" : "destructive"}>
                                {result.success ? 'Validation Passed' : 'Validation Failed'}
                            </Badge>
                        </div>

                        {/* ETABS Installation Info */}
                        <Card>
                            <CardHeader className="pb-3">
                                <CardTitle className="text-sm">ETABS Installation</CardTitle>
                            </CardHeader>
                            <CardContent className="space-y-2 text-sm">
                                <div className="flex justify-between">
                                    <span className="text-muted-foreground">Status:</span>
                                    <span className="font-medium">
                    {result.data.etabsInstalled ? (
                        <span className="text-green-600">Installed & Running</span>
                    ) : (
                        <span className="text-red-600">Not Found</span>
                    )}
                  </span>
                                </div>
                                {result.data.etabsVersion && (
                                    <div className="flex justify-between">
                                        <span className="text-muted-foreground">Version:</span>
                                        <span className="font-medium">{result.data.etabsVersion}</span>
                                    </div>
                                )}
                            </CardContent>
                        </Card>

                        {/* File Info */}
                        {result.data.filePath && (
                            <Card>
                                <CardHeader className="pb-3">
                                    <CardTitle className="text-sm">File Information</CardTitle>
                                </CardHeader>
                                <CardContent className="space-y-2 text-sm">
                                    <div className="flex justify-between">
                                        <span className="text-muted-foreground">Exists:</span>
                                        <span className="font-medium">
                      {result.data.fileExists ? '✓ Yes' : '✗ No'}
                    </span>
                                    </div>
                                    {result.data.fileExtension && (
                                        <div className="flex justify-between">
                                            <span className="text-muted-foreground">Type:</span>
                                            <span className="font-medium">{result.data.fileExtension}</span>
                                        </div>
                                    )}
                                    <div className="flex justify-between">
                                        <span className="text-muted-foreground">Valid:</span>
                                        <span className="font-medium">
                      {result.data.fileValid ? (
                          <span className="text-green-600">✓ Valid</span>
                      ) : (
                          <span className="text-red-600">✗ Invalid</span>
                      )}
                    </span>
                                    </div>
                                    {result.data.isAnalyzed !== undefined && (
                                        <div className="flex justify-between">
                                            <span className="text-muted-foreground">Analyzed:</span>
                                            <span className="font-medium">
                        {result.data.isAnalyzed ? (
                            <span className="text-green-600">✓ Yes</span>
                        ) : (
                            <span className="text-yellow-600">⚠ Not Analyzed</span>
                        )}
                      </span>
                                        </div>
                                    )}
                                </CardContent>
                            </Card>
                        )}

                        {/* Validation Messages */}
                        {result.data.validationMessages.length > 0 && (
                            <Card>
                                <CardHeader className="pb-3">
                                    <CardTitle className="text-sm">Process Log</CardTitle>
                                </CardHeader>
                                <CardContent>
                                    <div className="space-y-1 text-sm font-mono">
                                        {result.data.validationMessages.map((msg, idx) => (
                                            <div key={idx} className="text-muted-foreground">
                                                {msg}
                                            </div>
                                        ))}
                                    </div>
                                </CardContent>
                            </Card>
                        )}

                        {/* Timestamp */}
                        <div className="text-xs text-muted-foreground text-right">
                            Validated at: {new Date(result.timestamp).toLocaleString()}
                        </div>
                    </div>
                )}
            </CardContent>
        </Card>
    );
}