import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '@/components/ui/button.tsx';
import { Input } from '@/components/ui/input.tsx';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card.tsx';
import { Alert, AlertDescription } from '@/components/ui/alert.tsx';
import { Badge } from '@/components/ui/badge.tsx';
import { Switch } from '@/components/ui/switch.tsx';
import { Label } from '@/components/ui/label.tsx';
import { CheckCircle2, XCircle, AlertCircle, Loader2, FileOutput, Clock, HardDrive } from 'lucide-react';

// Type definitions matching your C# CLI output
interface GenerateE2KData {
    inputFile: string;
    outputFile?: string;
    fileExists: boolean;
    fileExtension?: string;
    outputExists?: boolean;
    generationSuccessful?: boolean;
    fileSizeBytes?: number;
    generationTimeMs?: number;
    messages: string[];
}

interface CliResult<T> {
    success: boolean;
    error?: string;
    timestamp: string;
    data?: T;
}

export function GenerateE2KPanel() {
    const [inputFile, setInputFile] = useState('');
    const [outputFile, setOutputFile] = useState('');
    const [overwrite, setOverwrite] = useState(false);
    const [result, setResult] = useState<CliResult<GenerateE2KData> | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const handleGenerate = async () => {
        if (!inputFile.trim()) {
            setError('Please enter an input file path');
            return;
        }

        setLoading(true);
        setError(null);
        setResult(null);

        try {
            // Call Rust command which returns CliResult<GenerateE2KData>
            const response = await invoke<CliResult<GenerateE2KData>>('generate_e2k', {
                inputFile: inputFile.trim(),
                outputFile: outputFile.trim() || null,
                overwrite
            });

            setResult(response);

            if (!response.success) {
                setError(response.error || 'Generation failed');
            }
        } catch (err) {
            const errorMessage = err instanceof Error ? err.message : String(err);
            setError(`Failed to execute command: ${errorMessage}`);
        } finally {
            setLoading(false);
        }
    };

    const formatFileSize = (bytes?: number): string => {
        if (!bytes) return 'Unknown';

        const units = ['B', 'KB', 'MB', 'GB'];
        let size = bytes;
        let unitIndex = 0;

        while (size >= 1024 && unitIndex < units.length - 1) {
            size /= 1024;
            unitIndex++;
        }

        return `${size.toFixed(2)} ${units[unitIndex]}`;
    };

    const formatDuration = (ms?: number): string => {
        if (!ms) return 'Unknown';

        if (ms < 1000) return `${ms}ms`;
        return `${(ms / 1000).toFixed(2)}s`;
    };

    return (
        <Card className="w-full">
            <CardHeader>
                <CardTitle className="flex items-center gap-2">
                    <FileOutput className="h-5 w-5" />
                    Generate E2K File
                </CardTitle>
                <CardDescription>
                    Convert ETABS .edb file to .e2k text format
                </CardDescription>
            </CardHeader>

            <CardContent className="space-y-4">
                {/* Input File */}
                <div className="space-y-2">
                    <Label htmlFor="input-file">Input File (.edb)</Label>
                    <Input
                        id="input-file"
                        type="text"
                        value={inputFile}
                        onChange={(e) => setInputFile(e.target.value)}
                        placeholder="D:\path\to\model.edb"
                        disabled={loading}
                    />
                </div>

                {/* Output File */}
                <div className="space-y-2">
                    <Label htmlFor="output-file">Output File (.e2k) - Optional</Label>
                    <Input
                        id="output-file"
                        type="text"
                        value={outputFile}
                        onChange={(e) => setOutputFile(e.target.value)}
                        placeholder="Leave empty to use default location"
                        disabled={loading}
                    />
                    <p className="text-xs text-muted-foreground">
                        If empty, will use same directory as input file
                    </p>
                </div>

                {/* Overwrite Option */}
                <div className="flex items-center space-x-2">
                    <Switch
                        id="overwrite"
                        checked={overwrite}
                        onCheckedChange={setOverwrite}
                        disabled={loading}
                    />
                    <Label htmlFor="overwrite" className="cursor-pointer">
                        Overwrite existing file
                    </Label>
                </div>

                {/* Generate Button */}
                <Button
                    onClick={handleGenerate}
                    disabled={loading || !inputFile.trim()}
                    className="w-full"
                >
                    {loading ? (
                        <>
                            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                            Generating E2K...
                        </>
                    ) : (
                        <>
                            <FileOutput className="mr-2 h-4 w-4" />
                            Generate E2K
                        </>
                    )}
                </Button>

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
                            {result.data.generationSuccessful ? (
                                <>
                                    <CheckCircle2 className="h-5 w-5 text-green-500" />
                                    <Badge variant="default">Generation Successful</Badge>
                                </>
                            ) : (
                                <>
                                    <XCircle className="h-5 w-5 text-red-500" />
                                    <Badge variant="destructive">Generation Failed</Badge>
                                </>
                            )}
                        </div>

                        {/* Output Details */}
                        {result.data.generationSuccessful && (
                            <Card>
                                <CardHeader className="pb-3">
                                    <CardTitle className="text-sm">Output File</CardTitle>
                                </CardHeader>
                                <CardContent className="space-y-3">
                                    {/* File Path */}
                                    <div className="p-3 bg-muted rounded-md">
                                        <p className="text-xs text-muted-foreground mb-1">Path:</p>
                                        <p className="text-sm font-mono break-all">
                                            {result.data.outputFile}
                                        </p>
                                    </div>

                                    {/* File Stats */}
                                    <div className="grid grid-cols-2 gap-4">
                                        {result.data.fileSizeBytes !== undefined && (
                                            <div className="flex items-center gap-2">
                                                <HardDrive className="h-4 w-4 text-muted-foreground" />
                                                <div>
                                                    <p className="text-xs text-muted-foreground">Size</p>
                                                    <p className="text-sm font-medium">
                                                        {formatFileSize(result.data.fileSizeBytes)}
                                                    </p>
                                                </div>
                                            </div>
                                        )}

                                        {result.data.generationTimeMs !== undefined && (
                                            <div className="flex items-center gap-2">
                                                <Clock className="h-4 w-4 text-muted-foreground" />
                                                <div>
                                                    <p className="text-xs text-muted-foreground">Duration</p>
                                                    <p className="text-sm font-medium">
                                                        {formatDuration(result.data.generationTimeMs)}
                                                    </p>
                                                </div>
                                            </div>
                                        )}
                                    </div>
                                </CardContent>
                            </Card>
                        )}

                        {/* Process Log */}
                        {result.data.messages.length > 0 && (
                            <Card>
                                <CardHeader className="pb-3">
                                    <CardTitle className="text-sm">Process Log</CardTitle>
                                </CardHeader>
                                <CardContent>
                                    <div className="space-y-1 max-h-60 overflow-y-auto">
                                        {result.data.messages.map((msg, idx) => (
                                            <div
                                                key={idx}
                                                className={`text-sm font-mono p-2 rounded ${
                                                    msg.includes('✓')
                                                        ? 'bg-green-50 text-green-700'
                                                        : msg.includes('✗')
                                                            ? 'bg-red-50 text-red-700'
                                                            : msg.includes('⚠')
                                                                ? 'bg-yellow-50 text-yellow-700'
                                                                : 'bg-muted text-muted-foreground'
                                                }`}
                                            >
                                                {msg}
                                            </div>
                                        ))}
                                    </div>
                                </CardContent>
                            </Card>
                        )}

                        {/* Timestamp */}
                        <div className="text-xs text-muted-foreground text-right">
                            Generated at: {new Date(result.timestamp).toLocaleString()}
                        </div>
                    </div>
                )}
            </CardContent>
        </Card>
    );
}