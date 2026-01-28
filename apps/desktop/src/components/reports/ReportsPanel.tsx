import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { Switch } from '@/components/ui/switch';
import {
    FileText,
    Download,
    Eye,
    Plus,
    GitCompare,
    BarChart3,
    FileSpreadsheet,
    Image,
    Settings2
} from 'lucide-react';

// Report templates
const reportTemplates = [
    {
        id: 'comparison',
        name: 'Version Comparison Report',
        description: 'Compare two design versions side-by-side',
        icon: GitCompare,
        fields: ['version1', 'version2', 'includeMaterials', 'includeAnalysis', 'include3D']
    },
    {
        id: 'analysis',
        name: 'Analysis Summary Report',
        description: 'Comprehensive analysis results for a version',
        icon: BarChart3,
        fields: ['version', 'includeForces', 'includeDisplacements', 'includeDesignChecks']
    },
    {
        id: 'bom',
        name: 'Bill of Materials',
        description: 'Material quantities and specifications',
        icon: FileSpreadsheet,
        fields: ['version', 'groupByMaterial', 'includeCosts', 'includeSuppliers']
    },
    {
        id: 'progress',
        name: 'Design Progress Report',
        description: 'Timeline and evolution of the design',
        icon: FileText,
        fields: ['branch', 'includeAllVersions', 'includeImages', 'includeComments']
    }
];

// Mock versions for selection
const mockVersions = [
    { id: 'main/v3', label: 'main/v3 - Final review', analyzed: true },
    { id: 'main/v2', label: 'main/v2 - Updated loads', analyzed: true },
    { id: 'main/v1', label: 'main/v1 - Initial design', analyzed: true },
    { id: 'steel-columns/v1', label: 'steel-columns/v1 - Steel alternative', analyzed: false },
    { id: 'foundation-redesign/v1', label: 'foundation-redesign/v1 - Deep foundation', analyzed: false }
];

// Mock generated reports
const mockReports = [
    {
        id: '1',
        name: 'Comparison: main/v2 vs main/v3',
        type: 'comparison',
        date: '2025-01-22 14:30',
        size: '2.4 MB',
        pages: 12
    },
    {
        id: '2',
        name: 'Analysis Summary: main/v3',
        type: 'analysis',
        date: '2025-01-20 16:15',
        size: '1.8 MB',
        pages: 8
    },
    {
        id: '3',
        name: 'Bill of Materials: steel-columns/v1',
        type: 'bom',
        date: '2025-01-22 10:00',
        size: '856 KB',
        pages: 4
    }
];

export function ReportsPanel() {
    const [selectedTemplate, setSelectedTemplate] = useState<string | null>(null);
    const [reportConfig, setReportConfig] = useState({
        version1: '',
        version2: '',
        version: '',
        branch: '',
        includeMaterials: true,
        includeAnalysis: true,
        include3D: true,
        includeForces: true,
        includeDisplacements: true,
        includeDesignChecks: true,
        groupByMaterial: true,
        includeCosts: false,
        includeSuppliers: false,
        includeAllVersions: true,
        includeImages: true,
        includeComments: true
    });

    const currentTemplate = reportTemplates.find(t => t.id === selectedTemplate);

    const handleGenerate = () => {
        console.log('Generating report with config:', reportConfig);
        // TODO: Call backend to generate report
    };

    return (
        <div className="h-full overflow-y-auto">
            <div className="max-w-6xl mx-auto p-6 space-y-6">
                {/* Header */}
                <div className="flex items-center justify-between">
                    <div>
                        <h1 className="text-2xl font-bold flex items-center gap-2">
                            <FileText className="w-6 h-6 text-primary" />
                            Reports
                        </h1>
                        <p className="text-muted-foreground mt-1">
                            Generate documentation and comparison reports
                        </p>
                    </div>
                </div>

                <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                    {/* Left: Report Templates */}
                    <div className="lg:col-span-1 space-y-3">
                        <h2 className="text-sm font-semibold text-muted-foreground uppercase">
                            Report Templates
                        </h2>

                        {reportTemplates.map((template) => {
                            const Icon = template.icon;
                            const isSelected = selectedTemplate === template.id;

                            return (
                                <Card
                                    key={template.id}
                                    className={`cursor-pointer transition-all ${
                                        isSelected
                                            ? 'border-primary bg-primary/5'
                                            : 'hover:border-border'
                                    }`}
                                    onClick={() => setSelectedTemplate(template.id)}
                                >
                                    <CardContent className="p-4">
                                        <div className="flex items-start gap-3">
                                            <div className={`p-2 rounded-lg ${
                                                isSelected ? 'bg-primary/10' : 'bg-muted'
                                            }`}>
                                                <Icon className={`w-5 h-5 ${
                                                    isSelected ? 'text-primary' : 'text-muted-foreground'
                                                }`} />
                                            </div>
                                            <div className="flex-1 min-w-0">
                                                <h3 className="font-medium text-sm mb-1">
                                                    {template.name}
                                                </h3>
                                                <p className="text-xs text-muted-foreground">
                                                    {template.description}
                                                </p>
                                            </div>
                                        </div>
                                    </CardContent>
                                </Card>
                            );
                        })}
                    </div>

                    {/* Middle: Configuration */}
                    <div className="lg:col-span-2 space-y-6">
                        {selectedTemplate ? (
                            <>
                                <Card>
                                    <CardHeader>
                                        <CardTitle className="text-base flex items-center gap-2">
                                            <Settings2 className="w-4 h-4" />
                                            Configure Report
                                        </CardTitle>
                                        <CardDescription>
                                            {currentTemplate?.description}
                                        </CardDescription>
                                    </CardHeader>
                                    <CardContent className="space-y-4">
                                        {/* Version Comparison Fields */}
                                        {selectedTemplate === 'comparison' && (
                                            <>
                                                <div className="space-y-2">
                                                    <Label>First Version</Label>
                                                    <select
                                                        value={reportConfig.version1}
                                                        onChange={(e) => setReportConfig({
                                                            ...reportConfig,
                                                            version1: e.target.value
                                                        })}
                                                        className="w-full h-9 rounded-md border border-input bg-background px-3 text-sm"
                                                    >
                                                        <option value="">Select version...</option>
                                                        {mockVersions.map(v => (
                                                            <option key={v.id} value={v.id}>
                                                                {v.label}
                                                            </option>
                                                        ))}
                                                    </select>
                                                </div>

                                                <div className="space-y-2">
                                                    <Label>Second Version</Label>
                                                    <select
                                                        value={reportConfig.version2}
                                                        onChange={(e) => setReportConfig({
                                                            ...reportConfig,
                                                            version2: e.target.value
                                                        })}
                                                        className="w-full h-9 rounded-md border border-input bg-background px-3 text-sm"
                                                    >
                                                        <option value="">Select version...</option>
                                                        {mockVersions.map(v => (
                                                            <option key={v.id} value={v.id}>
                                                                {v.label}
                                                            </option>
                                                        ))}
                                                    </select>
                                                </div>

                                                <div className="space-y-3 pt-2">
                                                    <div className="flex items-center justify-between">
                                                        <Label htmlFor="materials">Include Material Changes</Label>
                                                        <Switch
                                                            id="materials"
                                                            checked={reportConfig.includeMaterials}
                                                            onCheckedChange={(checked) => setReportConfig({
                                                                ...reportConfig,
                                                                includeMaterials: checked
                                                            })}
                                                        />
                                                    </div>

                                                    <div className="flex items-center justify-between">
                                                        <Label htmlFor="analysis">Include Analysis Results</Label>
                                                        <Switch
                                                            id="analysis"
                                                            checked={reportConfig.includeAnalysis}
                                                            onCheckedChange={(checked) => setReportConfig({
                                                                ...reportConfig,
                                                                includeAnalysis: checked
                                                            })}
                                                        />
                                                    </div>

                                                    <div className="flex items-center justify-between">
                                                        <Label htmlFor="3d">Include 3D Screenshots</Label>
                                                        <Switch
                                                            id="3d"
                                                            checked={reportConfig.include3D}
                                                            onCheckedChange={(checked) => setReportConfig({
                                                                ...reportConfig,
                                                                include3D: checked
                                                            })}
                                                        />
                                                    </div>
                                                </div>
                                            </>
                                        )}

                                        {/* Analysis Report Fields */}
                                        {selectedTemplate === 'analysis' && (
                                            <>
                                                <div className="space-y-2">
                                                    <Label>Version</Label>
                                                    <select
                                                        value={reportConfig.version}
                                                        onChange={(e) => setReportConfig({
                                                            ...reportConfig,
                                                            version: e.target.value
                                                        })}
                                                        className="w-full h-9 rounded-md border border-input bg-background px-3 text-sm"
                                                    >
                                                        <option value="">Select version...</option>
                                                        {mockVersions.filter(v => v.analyzed).map(v => (
                                                            <option key={v.id} value={v.id}>
                                                                {v.label}
                                                            </option>
                                                        ))}
                                                    </select>
                                                </div>

                                                <div className="space-y-3 pt-2">
                                                    <div className="flex items-center justify-between">
                                                        <Label htmlFor="forces">Member Forces</Label>
                                                        <Switch
                                                            id="forces"
                                                            checked={reportConfig.includeForces}
                                                            onCheckedChange={(checked) => setReportConfig({
                                                                ...reportConfig,
                                                                includeForces: checked
                                                            })}
                                                        />
                                                    </div>

                                                    <div className="flex items-center justify-between">
                                                        <Label htmlFor="displacements">Displacements</Label>
                                                        <Switch
                                                            id="displacements"
                                                            checked={reportConfig.includeDisplacements}
                                                            onCheckedChange={(checked) => setReportConfig({
                                                                ...reportConfig,
                                                                includeDisplacements: checked
                                                            })}
                                                        />
                                                    </div>

                                                    <div className="flex items-center justify-between">
                                                        <Label htmlFor="design">Design Checks</Label>
                                                        <Switch
                                                            id="design"
                                                            checked={reportConfig.includeDesignChecks}
                                                            onCheckedChange={(checked) => setReportConfig({
                                                                ...reportConfig,
                                                                includeDesignChecks: checked
                                                            })}
                                                        />
                                                    </div>
                                                </div>
                                            </>
                                        )}

                                        <Button
                                            className="w-full mt-4"
                                            onClick={handleGenerate}
                                            disabled={
                                                (selectedTemplate === 'comparison' && (!reportConfig.version1 || !reportConfig.version2)) ||
                                                (selectedTemplate === 'analysis' && !reportConfig.version)
                                            }
                                        >
                                            <FileText className="w-4 h-4" />
                                            Generate Report
                                        </Button>
                                    </CardContent>
                                </Card>

                                {/* Report Preview Info */}
                                <Card>
                                    <CardHeader>
                                        <CardTitle className="text-sm">Report Details</CardTitle>
                                    </CardHeader>
                                    <CardContent className="space-y-2 text-sm">
                                        <div className="flex justify-between">
                                            <span className="text-muted-foreground">Format:</span>
                                            <span className="font-medium">PDF (via Typst)</span>
                                        </div>
                                        <div className="flex justify-between">
                                            <span className="text-muted-foreground">Estimated size:</span>
                                            <span className="font-medium">~2-3 MB</span>
                                        </div>
                                        <div className="flex justify-between">
                                            <span className="text-muted-foreground">Generation time:</span>
                                            <span className="font-medium">~10-15 seconds</span>
                                        </div>
                                    </CardContent>
                                </Card>
                            </>
                        ) : (
                            <Card className="h-full min-h-[400px] flex items-center justify-center">
                                <CardContent>
                                    <div className="text-center">
                                        <FileText className="w-12 h-12 mx-auto text-muted-foreground mb-4" />
                                        <p className="text-muted-foreground">
                                            Select a report template to begin
                                        </p>
                                    </div>
                                </CardContent>
                            </Card>
                        )}
                    </div>
                </div>

                {/* Generated Reports History */}
                <div className="space-y-3">
                    <h2 className="text-sm font-semibold text-muted-foreground uppercase">
                        Recent Reports
                    </h2>

                    {mockReports.map((report) => (
                        <Card key={report.id}>
                            <CardContent className="p-4">
                                <div className="flex items-center justify-between">
                                    <div className="flex items-center gap-3">
                                        <div className="p-2 rounded-lg bg-muted">
                                            <FileText className="w-5 h-5 text-muted-foreground" />
                                        </div>
                                        <div>
                                            <h3 className="font-medium text-sm">{report.name}</h3>
                                            <div className="flex items-center gap-3 text-xs text-muted-foreground mt-1">
                                                <span>{report.date}</span>
                                                <span>•</span>
                                                <span>{report.size}</span>
                                                <span>•</span>
                                                <span>{report.pages} pages</span>
                                            </div>
                                        </div>
                                    </div>
                                    <div className="flex gap-2">
                                        <Button variant="ghost" size="sm">
                                            <Eye className="w-3 h-3" />
                                            View
                                        </Button>
                                        <Button variant="ghost" size="sm">
                                            <Download className="w-3 h-3" />
                                            Download
                                        </Button>
                                    </div>
                                </div>
                            </CardContent>
                        </Card>
                    ))}
                </div>
            </div>
        </div>
    );
}