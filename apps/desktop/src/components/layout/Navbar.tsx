import { Menu, Zap, GitBranch, FolderOpen, Settings } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { useProjectStore } from '@/store/projectStore';
import { open } from '@tauri-apps/plugin-dialog';

interface NavbarProps {
    onToggleSidebar: () => void;
}

export function Navbar({ onToggleSidebar }: NavbarProps) {
    const { currentProject, etabsStatus, openProject } = useProjectStore();

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

    return (
        <div className="border-b border-border/40 bg-background/80 backdrop-blur-md sticky top-0 z-50">
            <div className="px-4 py-3 flex items-center justify-between">
                <div className="flex items-center gap-3">
                    <button
                        onClick={onToggleSidebar}
                        className="p-1.5 hover:bg-accent rounded-md transition"
                    >
                        <Menu className="w-5 h-5" />
                    </button>

                    <div className="flex items-center gap-2 border-r border-border pr-4">
                        <div className="p-1.5 bg-primary/10 rounded-md">
                            <Zap className="w-4 h-4 text-primary" />
                        </div>
                        <span className="font-semibold text-sm">ETABS Extension</span>
                    </div>

                    {currentProject ? (
                        <>
                            <div className="flex items-center gap-2">
                                <div className="text-sm">
                                    <div className="font-medium">{currentProject.projectName}</div>
                                    <div className="flex items-center gap-1 text-xs text-muted-foreground">
                                        <GitBranch className="w-3 h-3" />
                                        <span>{currentProject.currentBranch}</span>
                                    </div>
                                </div>
                            </div>

                            {etabsStatus.isRunning && (
                                <Badge variant="outline" className="text-green-600 border-green-600">
                                    ETABS Running
                                </Badge>
                            )}
                        </>
                    ) : (
                        <div className="text-sm text-muted-foreground">
                            No project open
                        </div>
                    )}
                </div>

                <div className="flex items-center gap-2">
                    {!currentProject && (
                        <Button variant="ghost" size="sm" onClick={handleOpenProject}>
                            <FolderOpen className="w-4 h-4 mr-2" />
                            Open Project
                        </Button>
                    )}

                    <Button variant="ghost" size="icon">
                        <Settings className="w-4 h-4" />
                    </Button>
                </div>
            </div>
        </div>
    );
}