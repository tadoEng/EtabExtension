import { useState } from 'react';
import { Card, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import {
    Clock,
    GitCommit,
    Search,
    GitCompare,
    FileText,
    User,
    Calendar,
    Filter
} from 'lucide-react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';

// Mock commit data
const mockCommits = [
    {
        hash: 'a1b2c3d',
        message: 'Final review of main design',
        author: 'John Doe',
        date: '2025-01-20 14:30',
        branch: 'main',
        version: 'v3',
        filesChanged: 3,
        analyzed: true
    },
    {
        hash: 'e4f5g6h',
        message: 'Changed columns to steel sections',
        author: 'John Doe',
        date: '2025-01-22 09:15',
        branch: 'steel-columns',
        version: 'v1',
        filesChanged: 5,
        analyzed: false
    },
    {
        hash: 'i7j8k9l',
        message: 'Updated load combinations per new code',
        author: 'Jane Smith',
        date: '2025-01-18 16:45',
        branch: 'main',
        version: 'v2',
        filesChanged: 2,
        analyzed: true
    },
    {
        hash: 'm0n1o2p',
        message: 'Deep foundation redesign',
        author: 'John Doe',
        date: '2025-01-21 11:20',
        branch: 'foundation-redesign',
        version: 'v1',
        filesChanged: 8,
        analyzed: false
    },
    {
        hash: 'q3r4s5t',
        message: 'Initial structural design',
        author: 'Jane Smith',
        date: '2025-01-15 10:00',
        branch: 'main',
        version: 'v1',
        filesChanged: 12,
        analyzed: true
    }
];

export function HistoryPanel() {
    const [searchQuery, setSearchQuery] = useState('');
    const [selectedCommits, setSelectedCommits] = useState<string[]>([]);
    const [filterBranch, setFilterBranch] = useState<string>('all');

    const filteredCommits = mockCommits.filter(commit => {
        const matchesSearch = commit.message.toLowerCase().includes(searchQuery.toLowerCase()) ||
            commit.author.toLowerCase().includes(searchQuery.toLowerCase());
        const matchesBranch = filterBranch === 'all' || commit.branch === filterBranch;
        return matchesSearch && matchesBranch;
    });

    const toggleCommitSelection = (hash: string) => {
        setSelectedCommits(prev =>
            prev.includes(hash)
                ? prev.filter(h => h !== hash)
                : [...prev, hash].slice(-2) // Only keep last 2
        );
    };

    const branches = ['all', ...new Set(mockCommits.map(c => c.branch))];

    return (
        <div className="h-full overflow-y-auto">
            <div className="max-w-6xl mx-auto p-6 space-y-6">
                {/* Header */}
                <div className="flex items-center justify-between">
                    <div>
                        <h1 className="text-2xl font-bold flex items-center gap-2">
                            <Clock className="w-6 h-6 text-primary" />
                            Version History
                        </h1>
                        <p className="text-muted-foreground mt-1">
                            Complete timeline of all design changes across branches
                        </p>
                    </div>
                    {selectedCommits.length === 2 && (
                        <Button>
                            <GitCompare className="w-4 h-4" />
                            Compare Selected
                        </Button>
                    )}
                </div>

                {/* Search and Filter Bar */}
                <Card>
                    <CardContent className="p-4">
                        <div className="flex gap-4">
                            <div className="flex-1 relative">
                                <Search className="w-4 h-4 absolute left-3 top-2.5 text-muted-foreground" />
                                <Input
                                    placeholder="Search commits by message or author..."
                                    value={searchQuery}
                                    onChange={(e) => setSearchQuery(e.target.value)}
                                    className="pl-9"
                                />
                            </div>
                            <div className="flex items-center gap-2">
                                <Filter className="w-4 h-4 text-muted-foreground" />
                                <select
                                    value={filterBranch}
                                    onChange={(e) => setFilterBranch(e.target.value)}
                                    className="h-9 rounded-md border border-input bg-background px-3 text-sm"
                                >
                                    {branches.map(branch => (
                                        <option key={branch} value={branch}>
                                            {branch === 'all' ? 'All Branches' : branch}
                                        </option>
                                    ))}
                                </select>
                            </div>
                        </div>
                    </CardContent>
                </Card>

                {/* Tabs for different views */}
                <Tabs defaultValue="timeline">
                    <TabsList>
                        <TabsTrigger value="timeline">Timeline View</TabsTrigger>
                        <TabsTrigger value="by-branch">By Branch</TabsTrigger>
                        <TabsTrigger value="by-author">By Author</TabsTrigger>
                    </TabsList>

                    <TabsContent value="timeline" className="space-y-3 mt-4">
                        {filteredCommits.map((commit, idx) => (
                            <Card
                                key={commit.hash}
                                className={`cursor-pointer transition-all ${
                                    selectedCommits.includes(commit.hash)
                                        ? 'border-primary bg-primary/5'
                                        : 'hover:border-border'
                                }`}
                                onClick={() => toggleCommitSelection(commit.hash)}
                            >
                                <CardContent className="p-4">
                                    <div className="flex items-start gap-4">
                                        {/* Checkbox */}
                                        <div className="flex items-center h-6">
                                            <div className={`w-4 h-4 rounded border-2 flex items-center justify-center ${
                                                selectedCommits.includes(commit.hash)
                                                    ? 'bg-primary border-primary'
                                                    : 'border-muted-foreground'
                                            }`}>
                                                {selectedCommits.includes(commit.hash) && (
                                                    <div className="w-2 h-2 bg-white rounded-sm" />
                                                )}
                                            </div>
                                        </div>

                                        {/* Timeline connector */}
                                        <div className="relative flex flex-col items-center">
                                            <div className="w-3 h-3 rounded-full bg-primary border-2 border-background ring-2 ring-primary/20" />
                                            {idx < filteredCommits.length - 1 && (
                                                <div className="w-0.5 h-16 bg-border absolute top-3" />
                                            )}
                                        </div>

                                        {/* Content */}
                                        <div className="flex-1 min-w-0">
                                            <div className="flex items-start justify-between gap-4">
                                                <div className="flex-1">
                                                    <div className="flex items-center gap-2 mb-2">
                                                        <GitCommit className="w-4 h-4 text-muted-foreground" />
                                                        <span className="font-mono text-sm text-muted-foreground">
                              {commit.hash}
                            </span>
                                                        <Badge variant="outline">{commit.branch}</Badge>
                                                        <Badge variant="default">{commit.version}</Badge>
                                                        {commit.analyzed && (
                                                            <Badge variant="outline" className="text-green-600 border-green-600">
                                                                Analyzed
                                                            </Badge>
                                                        )}
                                                    </div>
                                                    <p className="font-medium mb-2">{commit.message}</p>
                                                    <div className="flex items-center gap-4 text-xs text-muted-foreground">
                            <span className="flex items-center gap-1">
                              <User className="w-3 h-3" />
                                {commit.author}
                            </span>
                                                        <span className="flex items-center gap-1">
                              <Calendar className="w-3 h-3" />
                                                            {commit.date}
                            </span>
                                                        <span className="flex items-center gap-1">
                              <FileText className="w-3 h-3" />
                                                            {commit.filesChanged} files changed
                            </span>
                                                    </div>
                                                </div>
                                                <Button variant="ghost" size="sm">
                                                    View Details
                                                </Button>
                                            </div>
                                        </div>
                                    </div>
                                </CardContent>
                            </Card>
                        ))}

                        {filteredCommits.length === 0 && (
                            <Card>
                                <CardContent className="p-12 text-center">
                                    <Clock className="w-12 h-12 mx-auto text-muted-foreground mb-4" />
                                    <p className="text-muted-foreground">No commits found</p>
                                </CardContent>
                            </Card>
                        )}
                    </TabsContent>

                    <TabsContent value="by-branch" className="mt-4">
                        <Card>
                            <CardContent className="p-6">
                                <p className="text-muted-foreground text-center">
                                    Branch-grouped view coming soon
                                </p>
                            </CardContent>
                        </Card>
                    </TabsContent>

                    <TabsContent value="by-author" className="mt-4">
                        <Card>
                            <CardContent className="p-6">
                                <p className="text-muted-foreground text-center">
                                    Author-grouped view coming soon
                                </p>
                            </CardContent>
                        </Card>
                    </TabsContent>
                </Tabs>

                {/* Selection Info */}
                {selectedCommits.length > 0 && (
                    <Card className="border-primary/50 bg-primary/5">
                        <CardContent className="p-4">
                            <div className="flex items-center justify-between">
                                <div className="flex items-center gap-2">
                                    <GitCompare className="w-4 h-4 text-primary" />
                                    <span className="text-sm">
                    {selectedCommits.length === 1
                        ? '1 commit selected'
                        : `${selectedCommits.length} commits selected - ready to compare`}
                  </span>
                                </div>
                                <Button
                                    variant="ghost"
                                    size="sm"
                                    onClick={() => setSelectedCommits([])}
                                >
                                    Clear Selection
                                </Button>
                            </div>
                        </CardContent>
                    </Card>
                )}
            </div>
        </div>
    );
}