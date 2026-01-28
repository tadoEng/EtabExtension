import { Package, FileCode, Code2, Settings } from 'lucide-react';

export function Sidebar() {
    return (
        <div className="w-64 border-r border-border/40 bg-background/50 overflow-y-auto">
            <div className="p-4 space-y-4">
                {/* Project Info */}
                <div>
                    <h3 className="text-xs font-semibold text-muted-foreground uppercase mb-3">
                        Project
                    </h3>
                    <div className="space-y-2">
                        <div className="p-3 rounded-lg bg-card border border-border/50 hover:border-primary/50 cursor-pointer transition">
                            <div className="flex items-center gap-2 mb-1">
                                <Package className="w-4 h-4 text-primary" />
                                <span className="text-sm font-medium">Dependencies</span>
                            </div>
                            <p className="text-xs text-muted-foreground">
                                Tauri + React + Vite
                            </p>
                        </div>
                    </div>
                </div>

                {/* Tools */}
                <div>
                    <h3 className="text-xs font-semibold text-muted-foreground uppercase mb-3">
                        Tools
                    </h3>
                    <div className="space-y-1">
                        <button className="w-full text-left px-3 py-2 rounded-md text-sm flex items-center gap-2 hover:bg-accent transition">
                            <FileCode className="w-4 h-4" />
                            Editor
                        </button>
                        <button className="w-full text-left px-3 py-2 rounded-md text-sm flex items-center gap-2 hover:bg-accent transition">
                            <Code2 className="w-4 h-4" />
                            Terminal
                        </button>
                        <button className="w-full text-left px-3 py-2 rounded-md text-sm flex items-center gap-2 hover:bg-accent transition">
                            <Settings className="w-4 h-4" />
                            Settings
                        </button>
                    </div>
                </div>

                {/* Features */}
                <div>
                    <h3 className="text-xs font-semibold text-muted-foreground uppercase mb-3">
                        Features
                    </h3>
                    <div className="space-y-2 text-xs">
                        <div className="flex items-center gap-2 p-2 rounded-md bg-primary/10 border border-primary/20">
                            <div className="w-2 h-2 rounded-full bg-primary animate-pulse" />
                            <span>Hot Module Reload</span>
                        </div>
                        <div className="flex items-center gap-2 p-2 rounded-md hover:bg-accent">
                            <div className="w-2 h-2 rounded-full bg-secondary" />
                            <span>Type Safe</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}