import { useEffect, useState } from 'react';
import { FileCode, GitBranch, Clock, FileText, Settings } from 'lucide-react';
import { useProjectStore } from '@/store/projectStore';
import { mockProjectState } from '@etab-extension/shared/mocks';
// Layout components
import { Navbar } from '@/components/layout/Navbar';

// Main feature panels
import { WorkspacePanel } from '@/components/workspace/WorkspacePanel';
import { BranchesPanel } from '@/components/branches/BranchesPanel';
import { HistoryPanel } from '@/components/history/HistoryPanel';
import { ReportsPanel } from '@/components/reports/ReportsPanel';
import { SettingsPanel } from '@/components/settings/SettingsPanel';

type MainView = 'workspace' | 'branches' | 'history' | 'reports' | 'settings';

function App() {
    const [sidebarOpen, setSidebarOpen] = useState(true);
    const [activeView, setActiveView] = useState<MainView>('workspace');
    const { currentProject, openProject } = useProjectStore();

    useEffect(() => {
        // Auto-load mock project in development
        if (!currentProject) {
            openProject(mockProjectState.project_path);
        }
    }, []);

    const renderMainContent = () => {
        switch (activeView) {
            case 'workspace':
                return <WorkspacePanel />;
            case 'branches':
                return <BranchesPanel />;
            case 'history':
                return <HistoryPanel />;
            case 'reports':
                return <ReportsPanel />;
            case 'settings':
                return <SettingsPanel />;
            default:
                return <WorkspacePanel />;
        }
    };

    return (
        <div className="min-h-screen w-screen bg-background flex flex-col">
            {/* Top Navbar */}
            <Navbar
                onToggleSidebar={() => setSidebarOpen(!sidebarOpen)}
                currentProject={currentProject}
                onOpenProject={() => {/* TODO: Implement project selector */}}
            />

            {/* Main Layout */}
            <div className="flex flex-1 overflow-hidden">
                {/* Left Sidebar - View Navigation */}
                {sidebarOpen && (
                    <div className="w-16 border-r border-border/40 bg-background/50 flex flex-col items-center py-4 gap-2">
                        <button
                            onClick={() => setActiveView('workspace')}
                            className={`p-3 rounded-lg transition-colors ${
                                activeView === 'workspace'
                                    ? 'bg-primary text-primary-foreground'
                                    : 'hover:bg-accent text-muted-foreground'
                            }`}
                            title="Workspace"
                        >
                            <FileCode className="w-5 h-5" />
                        </button>

                        <button
                            onClick={() => setActiveView('branches')}
                            className={`p-3 rounded-lg transition-colors ${
                                activeView === 'branches'
                                    ? 'bg-primary text-primary-foreground'
                                    : 'hover:bg-accent text-muted-foreground'
                            }`}
                            title="Branches"
                        >
                            <GitBranch className="w-5 h-5" />
                        </button>

                        <button
                            onClick={() => setActiveView('history')}
                            className={`p-3 rounded-lg transition-colors ${
                                activeView === 'history'
                                    ? 'bg-primary text-primary-foreground'
                                    : 'hover:bg-accent text-muted-foreground'
                            }`}
                            title="History"
                        >
                            <Clock className="w-5 h-5" />
                        </button>

                        <button
                            onClick={() => setActiveView('reports')}
                            className={`p-3 rounded-lg transition-colors ${
                                activeView === 'reports'
                                    ? 'bg-primary text-primary-foreground'
                                    : 'hover:bg-accent text-muted-foreground'
                            }`}
                            title="Reports"
                        >
                            <FileText className="w-5 h-5" />
                        </button>

                        <div className="flex-1" />

                        <button
                            onClick={() => setActiveView('settings')}
                            className={`p-3 rounded-lg transition-colors ${
                                activeView === 'settings'
                                    ? 'bg-primary text-primary-foreground'
                                    : 'hover:bg-accent text-muted-foreground'
                            }`}
                            title="Settings"
                        >
                            <Settings className="w-5 h-5" />
                        </button>
                    </div>
                )}

                {/* Main Content Area */}
                <div className="flex-1 overflow-hidden">
                    {renderMainContent()}
                </div>
            </div>
        </div>
    );
}

export default App;