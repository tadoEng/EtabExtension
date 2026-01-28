import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import {
    Settings,
    FolderOpen,
    User,
    GitBranch,
    Save,
    CheckCircle2,
    HardDrive,
    Zap
} from 'lucide-react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';

export function SettingsPanel() {
    const [settings, setSettings] = useState({
        // General
        userName: 'John Doe',
        userEmail: 'john.doe@example.com',
        defaultBranch: 'main',

        // Paths
        etabsPath: 'C:\\Program Files\\Computers and Structures\\ETABS 22\\ETABS.exe',
        cliPath: 'C:\\Program Files\\EtabsExtension\\cli\\EtabsCLI.exe',
        projectsRoot: 'D:\\ETABS Projects',

        // Behavior
        autoGenerateE2K: true,
        autoOpenInEtabs: false,
        confirmBeforeDelete: true,
        autoSaveInterval: 5, // minutes

        // Git
        gitAuthor: 'John Doe',
        gitEmail: 'john.doe@example.com',
        enableGitRemote: false,
        gitRemoteUrl: '',

        // UI
        theme: 'dark',
        showLineNumbers: true,
        enableAnimations: true,
        compactMode: false
    });

    const [hasChanges, setHasChanges] = useState(false);

    const updateSetting = (key: string, value: any) => {
        setSettings(prev => ({ ...prev, [key]: value }));
        setHasChanges(true);
    };

    const handleSave = () => {
        console.log('Saving settings:', settings);
        // TODO: Save to backend
        setHasChanges(false);
    };

    const handleReset = () => {
        // TODO: Load from backend
        setHasChanges(false);
    };

    return (
        <div className="h-full overflow-y-auto">
            <div className="max-w-4xl mx-auto p-6 space-y-6">
                {/* Header */}
                <div className="flex items-center justify-between">
                    <div>
                        <h1 className="text-2xl font-bold flex items-center gap-2">
                            <Settings className="w-6 h-6 text-primary" />
                            Settings
                        </h1>
                        <p className="text-muted-foreground mt-1">
                            Configure application preferences and paths
                        </p>
                    </div>
                    {hasChanges && (
                        <div className="flex gap-2">
                            <Button variant="outline" onClick={handleReset}>
                                Reset
                            </Button>
                            <Button onClick={handleSave}>
                                <Save className="w-4 h-4" />
                                Save Changes
                            </Button>
                        </div>
                    )}
                </div>

                {/* System Status */}
                <Card className="border-green-200 bg-green-50 dark:border-green-900 dark:bg-green-950">
                    <CardContent className="p-4">
                        <div className="flex items-center justify-between">
                            <div className="flex items-center gap-2">
                                <CheckCircle2 className="w-5 h-5 text-green-600 dark:text-green-400" />
                                <div>
                                    <p className="font-medium text-green-900 dark:text-green-100">
                                        System Ready
                                    </p>
                                    <p className="text-sm text-green-700 dark:text-green-300">
                                        ETABS CLI v1.0.0 • Git installed • All paths configured
                                    </p>
                                </div>
                            </div>
                            <Button variant="outline" size="sm">
                                Run Diagnostics
                            </Button>
                        </div>
                    </CardContent>
                </Card>

                {/* Settings Tabs */}
                <Tabs defaultValue="general" className="space-y-4">
                    <TabsList>
                        <TabsTrigger value="general">General</TabsTrigger>
                        <TabsTrigger value="paths">Paths</TabsTrigger>
                        <TabsTrigger value="behavior">Behavior</TabsTrigger>
                        <TabsTrigger value="git">Git</TabsTrigger>
                        <TabsTrigger value="ui">Interface</TabsTrigger>
                    </TabsList>

                    {/* General Tab */}
                    <TabsContent value="general" className="space-y-4">
                        <Card>
                            <CardHeader>
                                <CardTitle className="text-base flex items-center gap-2">
                                    <User className="w-4 h-4" />
                                    User Information
                                </CardTitle>
                                <CardDescription>
                                    Your identity for version commits
                                </CardDescription>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <div className="space-y-2">
                                    <Label htmlFor="userName">Name</Label>
                                    <Input
                                        id="userName"
                                        value={settings.userName}
                                        onChange={(e) => updateSetting('userName', e.target.value)}
                                    />
                                </div>

                                <div className="space-y-2">
                                    <Label htmlFor="userEmail">Email</Label>
                                    <Input
                                        id="userEmail"
                                        type="email"
                                        value={settings.userEmail}
                                        onChange={(e) => updateSetting('userEmail', e.target.value)}
                                    />
                                </div>

                                <div className="space-y-2">
                                    <Label htmlFor="defaultBranch">Default Branch Name</Label>
                                    <Input
                                        id="defaultBranch"
                                        value={settings.defaultBranch}
                                        onChange={(e) => updateSetting('defaultBranch', e.target.value)}
                                    />
                                    <p className="text-xs text-muted-foreground">
                                        Used when creating new projects
                                    </p>
                                </div>
                            </CardContent>
                        </Card>
                    </TabsContent>

                    {/* Paths Tab */}
                    <TabsContent value="paths" className="space-y-4">
                        <Card>
                            <CardHeader>
                                <CardTitle className="text-base flex items-center gap-2">
                                    <FolderOpen className="w-4 h-4" />
                                    Application Paths
                                </CardTitle>
                                <CardDescription>
                                    Configure locations of ETABS and tools
                                </CardDescription>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <div className="space-y-2">
                                    <Label htmlFor="etabsPath">ETABS Executable</Label>
                                    <div className="flex gap-2">
                                        <Input
                                            id="etabsPath"
                                            value={settings.etabsPath}
                                            onChange={(e) => updateSetting('etabsPath', e.target.value)}
                                            className="flex-1"
                                        />
                                        <Button variant="outline">Browse</Button>
                                    </div>
                                </div>

                                <div className="space-y-2">
                                    <Label htmlFor="cliPath">CLI Tool</Label>
                                    <div className="flex gap-2">
                                        <Input
                                            id="cliPath"
                                            value={settings.cliPath}
                                            onChange={(e) => updateSetting('cliPath', e.target.value)}
                                            className="flex-1"
                                        />
                                        <Button variant="outline">Browse</Button>
                                    </div>
                                </div>

                                <div className="space-y-2">
                                    <Label htmlFor="projectsRoot">Projects Directory</Label>
                                    <div className="flex gap-2">
                                        <Input
                                            id="projectsRoot"
                                            value={settings.projectsRoot}
                                            onChange={(e) => updateSetting('projectsRoot', e.target.value)}
                                            className="flex-1"
                                        />
                                        <Button variant="outline">Browse</Button>
                                    </div>
                                    <p className="text-xs text-muted-foreground">
                                        Default location for new ETABS projects
                                    </p>
                                </div>
                            </CardContent>
                        </Card>
                    </TabsContent>

                    {/* Behavior Tab */}
                    <TabsContent value="behavior" className="space-y-4">
                        <Card>
                            <CardHeader>
                                <CardTitle className="text-base flex items-center gap-2">
                                    <Zap className="w-4 h-4" />
                                    Application Behavior
                                </CardTitle>
                                <CardDescription>
                                    Configure automatic actions and workflows
                                </CardDescription>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <div className="flex items-center justify-between">
                                    <div className="space-y-0.5">
                                        <Label htmlFor="autoE2K">Auto-generate E2K</Label>
                                        <p className="text-xs text-muted-foreground">
                                            Automatically create E2K file when saving versions
                                        </p>
                                    </div>
                                    <Switch
                                        id="autoE2K"
                                        checked={settings.autoGenerateE2K}
                                        onCheckedChange={(checked) => updateSetting('autoGenerateE2K', checked)}
                                    />
                                </div>

                                <div className="flex items-center justify-between">
                                    <div className="space-y-0.5">
                                        <Label htmlFor="autoOpen">Auto-open in ETABS</Label>
                                        <p className="text-xs text-muted-foreground">
                                            Open file in ETABS after checkout
                                        </p>
                                    </div>
                                    <Switch
                                        id="autoOpen"
                                        checked={settings.autoOpenInEtabs}
                                        onCheckedChange={(checked) => updateSetting('autoOpenInEtabs', checked)}
                                    />
                                </div>

                                <div className="flex items-center justify-between">
                                    <div className="space-y-0.5">
                                        <Label htmlFor="confirmDelete">Confirm deletions</Label>
                                        <p className="text-xs text-muted-foreground">
                                            Ask before deleting branches or versions
                                        </p>
                                    </div>
                                    <Switch
                                        id="confirmDelete"
                                        checked={settings.confirmBeforeDelete}
                                        onCheckedChange={(checked) => updateSetting('confirmBeforeDelete', checked)}
                                    />
                                </div>

                                <div className="space-y-2">
                                    <Label htmlFor="autoSave">Auto-save interval (minutes)</Label>
                                    <Input
                                        id="autoSave"
                                        type="number"
                                        min="0"
                                        value={settings.autoSaveInterval}
                                        onChange={(e) => updateSetting('autoSaveInterval', parseInt(e.target.value))}
                                    />
                                    <p className="text-xs text-muted-foreground">
                                        Set to 0 to disable auto-save
                                    </p>
                                </div>
                            </CardContent>
                        </Card>
                    </TabsContent>

                    {/* Git Tab */}
                    <TabsContent value="git" className="space-y-4">
                        <Card>
                            <CardHeader>
                                <CardTitle className="text-base flex items-center gap-2">
                                    <GitBranch className="w-4 h-4" />
                                    Git Configuration
                                </CardTitle>
                                <CardDescription>
                                    Version control settings
                                </CardDescription>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <div className="space-y-2">
                                    <Label htmlFor="gitAuthor">Git Author Name</Label>
                                    <Input
                                        id="gitAuthor"
                                        value={settings.gitAuthor}
                                        onChange={(e) => updateSetting('gitAuthor', e.target.value)}
                                    />
                                </div>

                                <div className="space-y-2">
                                    <Label htmlFor="gitEmail">Git Email</Label>
                                    <Input
                                        id="gitEmail"
                                        type="email"
                                        value={settings.gitEmail}
                                        onChange={(e) => updateSetting('gitEmail', e.target.value)}
                                    />
                                </div>

                                <div className="border-t pt-4 space-y-4">
                                    <div className="flex items-center justify-between">
                                        <div className="space-y-0.5">
                                            <Label htmlFor="gitRemote">Enable Remote Sync</Label>
                                            <p className="text-xs text-muted-foreground">
                                                Sync with remote Git repository
                                            </p>
                                        </div>
                                        <Switch
                                            id="gitRemote"
                                            checked={settings.enableGitRemote}
                                            onCheckedChange={(checked) => updateSetting('enableGitRemote', checked)}
                                        />
                                    </div>

                                    {settings.enableGitRemote && (
                                        <div className="space-y-2">
                                            <Label htmlFor="gitRemoteUrl">Remote URL</Label>
                                            <Input
                                                id="gitRemoteUrl"
                                                placeholder="https://github.com/user/repo.git"
                                                value={settings.gitRemoteUrl}
                                                onChange={(e) => updateSetting('gitRemoteUrl', e.target.value)}
                                            />
                                        </div>
                                    )}
                                </div>
                            </CardContent>
                        </Card>
                    </TabsContent>

                    {/* UI Tab */}
                    <TabsContent value="ui" className="space-y-4">
                        <Card>
                            <CardHeader>
                                <CardTitle className="text-base">Interface Preferences</CardTitle>
                                <CardDescription>
                                    Customize the application appearance
                                </CardDescription>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <div className="space-y-2">
                                    <Label htmlFor="theme">Theme</Label>
                                    <select
                                        id="theme"
                                        value={settings.theme}
                                        onChange={(e) => updateSetting('theme', e.target.value)}
                                        className="w-full h-9 rounded-md border border-input bg-background px-3 text-sm"
                                    >
                                        <option value="light">Light</option>
                                        <option value="dark">Dark</option>
                                        <option value="system">System</option>
                                    </select>
                                </div>

                                <div className="flex items-center justify-between">
                                    <div className="space-y-0.5">
                                        <Label htmlFor="lineNumbers">Show line numbers</Label>
                                        <p className="text-xs text-muted-foreground">
                                            In code editor and diff viewer
                                        </p>
                                    </div>
                                    <Switch
                                        id="lineNumbers"
                                        checked={settings.showLineNumbers}
                                        onCheckedChange={(checked) => updateSetting('showLineNumbers', checked)}
                                    />
                                </div>

                                <div className="flex items-center justify-between">
                                    <div className="space-y-0.5">
                                        <Label htmlFor="animations">Enable animations</Label>
                                        <p className="text-xs text-muted-foreground">
                                            Smooth transitions and effects
                                        </p>
                                    </div>
                                    <Switch
                                        id="animations"
                                        checked={settings.enableAnimations}
                                        onCheckedChange={(checked) => updateSetting('enableAnimations', checked)}
                                    />
                                </div>

                                <div className="flex items-center justify-between">
                                    <div className="space-y-0.5">
                                        <Label htmlFor="compact">Compact mode</Label>
                                        <p className="text-xs text-muted-foreground">
                                            Reduce spacing for more content
                                        </p>
                                    </div>
                                    <Switch
                                        id="compact"
                                        checked={settings.compactMode}
                                        onCheckedChange={(checked) => updateSetting('compactMode', checked)}
                                    />
                                </div>
                            </CardContent>
                        </Card>
                    </TabsContent>
                </Tabs>

                {/* Storage Info */}
                <Card>
                    <CardHeader>
                        <CardTitle className="text-sm flex items-center gap-2">
                            <HardDrive className="w-4 h-4" />
                            Storage Usage
                        </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-3">
                        <div className="flex justify-between text-sm">
                            <span className="text-muted-foreground">Project files:</span>
                            <span className="font-medium">2.4 GB</span>
                        </div>
                        <div className="flex justify-between text-sm">
                            <span className="text-muted-foreground">Git repository:</span>
                            <span className="font-medium">856 MB</span>
                        </div>
                        <div className="flex justify-between text-sm">
                            <span className="text-muted-foreground">E2K files:</span>
                            <span className="font-medium">124 MB</span>
                        </div>
                        <div className="border-t pt-3 flex justify-between font-medium">
                            <span>Total:</span>
                            <span>3.4 GB</span>
                        </div>
                        <Button variant="outline" className="w-full mt-2" size="sm">
                            Clean up temporary files
                        </Button>
                    </CardContent>
                </Card>
            </div>
        </div>
    );
}