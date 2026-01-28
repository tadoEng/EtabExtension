import { useState } from 'react';
import Editor from '@monaco-editor/react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Code2, Copy } from 'lucide-react';

const DEFAULT_CODE = `function calculateSum(a, b) {
  return a + b;
}

const result = calculateSum(10, 20);
console.log(\`Result: \${result}\`);`;

export function MonacoCodeEditor() {
    const [code, setCode] = useState(DEFAULT_CODE);

    return (
        <Card className="h-full flex flex-col border-border/50">
            <CardHeader>
                <div className="flex items-center justify-between">
                    <CardTitle className="text-sm flex items-center gap-2">
                        <Code2 className="w-4 h-4 text-primary" />
                        Monaco Editor
                    </CardTitle>
                    <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => navigator.clipboard.writeText(code)}
                    >
                        <Copy className="w-4 h-4" />
                    </Button>
                </div>
            </CardHeader>
            <CardContent className="flex-1 p-0 border-t border-border/50">
                <Editor
                    height="100%"
                    defaultLanguage="javascript"
                    value={code}
                    onChange={(value) => setCode(value || '')}
                    theme="vs-dark"
                    options={{
                        minimap: { enabled: false },
                        fontSize: 14,
                        fontFamily: 'Fira Code, Courier New',
                        lineNumbers: 'on',
                        scrollBeyondLastLine: false,
                        automaticLayout: true,
                        padding: { top: 16, bottom: 16 },
                    }}
                />
            </CardContent>
        </Card>
    );
}