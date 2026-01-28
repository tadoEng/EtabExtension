import { DiffEditor } from '@monaco-editor/react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { GitCompare } from 'lucide-react';

const ORIGINAL_CODE = `function calculateSum(a, b) {
  return a + b;
}

const result = calculateSum(5, 15);
console.log(\`Sum: \${result}\`);`;

const MODIFIED_CODE = `function calculateSum(a, b) {
  return a + b;
}

const result = calculateSum(10, 20);
console.log(\`Result: \${result}\`);`;

export function MonacoDiffViewer() {
    return (
        <Card className="h-full border-border/50 flex flex-col">
            <CardHeader>
                <div className="flex items-center gap-2">
                    <GitCompare className="w-5 h-5 text-primary" />
                    <div>
                        <CardTitle className="text-sm">Git Compare - File Changes</CardTitle>
                        <CardDescription>View differences between original and modified code</CardDescription>
                    </div>
                </div>
            </CardHeader>
            <CardContent className="flex-1 p-0 border-t border-border/50">
                <DiffEditor
                    height="100%"
                    original={ORIGINAL_CODE}
                    modified={MODIFIED_CODE}
                    language="javascript"
                    theme="vs-dark"
                    options={{
                        minimap: { enabled: false },
                        fontSize: 14,
                        fontFamily: 'Fira Code, Courier New',
                        lineNumbers: 'on',
                        scrollBeyondLastLine: false,
                        automaticLayout: true,
                        padding: { top: 16, bottom: 16 },
                        renderSideBySide: true,
                    }}
                />
            </CardContent>
        </Card>
    );
}