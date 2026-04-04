import { useState, useRef, useEffect } from 'react';
import Editor, { OnMount } from '@monaco-editor/react';
import type * as MonacoType from 'monaco-editor';

const ETABS_SAMPLE = `// ETABS story drift analysis — TypeScript
// Tests Monaco: syntax highlight, IntelliSense, multi-language

interface StoryDrift {
    story: string;
    driftX: number;
    driftY: number;
    exceedsLimit: boolean;
}

const DRIFT_LIMIT = 0.005; // ASCE 7-22 §12.12.1 — 0.5% for Risk Category II

function analyzeDrifts(rawData: Record<string, [number, number]>): StoryDrift[] {
    return Object.entries(rawData).map(([story, [driftX, driftY]]) => ({
        story,
        driftX,
        driftY,
        exceedsLimit: driftX > DRIFT_LIMIT || driftY > DRIFT_LIMIT,
    }));
}

const data: Record<string, [number, number]> = {
    B1: [0.000, 0.000],
    GF: [0.0012, 0.0010],
    L1: [0.0045, 0.0038],
    L2: [0.0068, 0.0059], // exceeds limit
    L3: [0.0082, 0.0074], // exceeds limit
    L4: [0.0091, 0.0088], // exceeds limit
};

const results = analyzeDrifts(data);
const exceeding = results.filter(r => r.exceedsLimit);

console.log(\`\${exceeding.length} stories exceed drift limit of \${DRIFT_LIMIT * 100}%\`);
`;

const RUST_SAMPLE = `// ext-calc story drift calculation — Rust
// Tests Monaco: Rust syntax highlighting

use polars::prelude::*;

#[derive(Debug)]
pub struct StoryDrift {
    pub story: String,
    pub drift_x: f64,
    pub drift_y: f64,
}

impl StoryDrift {
    pub fn exceeds_limit(&self, limit: f64) -> bool {
        self.drift_x > limit || self.drift_y > limit
    }
}

pub fn compute_drifts(df: &DataFrame) -> PolarsResult<Vec<StoryDrift>> {
    let stories = df.column("Story")?.str()?;
    let drift_x = df.column("DriftX")?.f64()?;
    let drift_y = df.column("DriftY")?.f64()?;

    let results = stories
        .into_iter()
        .zip(drift_x.into_iter())
        .zip(drift_y.into_iter())
        .filter_map(|((s, dx), dy)| {
            Some(StoryDrift {
                story: s?.to_string(),
                drift_x: dx?,
                drift_y: dy?,
            })
        })
        .collect();

    Ok(results)
}
`;

type Lang = 'typescript' | 'rust' | 'json';

const LANGS: { id: Lang; label: string; content: string }[] = [
    { id: 'typescript', label: 'TypeScript', content: ETABS_SAMPLE },
    { id: 'rust',       label: 'Rust',       content: RUST_SAMPLE  },
    {
        id: 'json', label: 'JSON',
        content: JSON.stringify({
            project: 'EtabExtension',
            version: '0.1.0',
            analysis: { type: 'ResponseSpectrum', driftLimit: 0.005, stories: 10 },
            results: [
                { story: 'L4', driftX: 0.0091, driftY: 0.0088, status: 'FAIL' },
                { story: 'L3', driftX: 0.0082, driftY: 0.0074, status: 'FAIL' },
                { story: 'GF', driftX: 0.0012, driftY: 0.0010, status: 'OK'   },
            ],
        }, null, 2),
    },
];

export function MonacoTest() {
    const [lang, setLang] = useState<Lang>('typescript');
    const [editorReady, setEditorReady] = useState(false);
    const [editCount, setEditCount] = useState(0);
    const editorRef = useRef<MonacoType.editor.IStandaloneCodeEditor | null>(null);

    // FIX: defer mounting the Editor by one rAF tick.
    // Monaco's AMD loader + automaticLayout:true measures the container on mount.
    // On first React.lazy() load the container has 0px height at the moment
    // React commits — deferring one frame guarantees the DOM has been painted.
    const [ready, setReady] = useState(false);
    useEffect(() => {
        const id = requestAnimationFrame(() => setReady(true));
        return () => cancelAnimationFrame(id);
    }, []);

    const current = LANGS.find(l => l.id === lang)!;

    const handleMount: OnMount = (editor) => {
        editorRef.current = editor;
        setEditorReady(true);
        editor.focus();
    };

    const formatDocument = () => {
        editorRef.current?.getAction('editor.action.formatDocument')?.run();
    };

    return (
        <div className="space-y-3 h-full flex flex-col">
            <p className="text-xs text-muted-foreground">
                <code className="text-xs bg-muted px-1 rounded">@monaco-editor/react</code> uses
                its own AMD loader and TypeScript worker. Tests Rolldown chunk isolation — Monaco's
                internal module system must not conflict with the app's ES module graph.
            </p>

            <div className="flex items-center gap-2 flex-wrap">
                <div className="flex gap-1 bg-muted/40 p-1 rounded-lg border border-border/40">
                    {LANGS.map(l => (
                        <button
                            key={l.id}
                            onClick={() => setLang(l.id)}
                            className={`px-3 py-1 text-xs rounded-md transition-colors ${
                                lang === l.id
                                    ? 'bg-primary text-primary-foreground'
                                    : 'text-muted-foreground hover:text-foreground'
                            }`}
                        >
                            {l.label}
                        </button>
                    ))}
                </div>

                <button
                    onClick={formatDocument}
                    disabled={!editorReady}
                    className="px-3 py-1 text-xs rounded-md border border-border/60 hover:bg-accent transition-colors disabled:opacity-40"
                >
                    Format document
                </button>

                <div className="flex gap-2 ml-auto">
                    <span className={`text-xs px-2 py-1 rounded-md border ${
                        editorReady
                            ? 'bg-green-500/10 border-green-500/30 text-green-400'
                            : 'bg-muted/40 border-border/40 text-muted-foreground'
                    }`}>
                        {editorReady ? 'Editor ready' : 'Mounting…'}
                    </span>
                    <span className="text-xs px-2 py-1 rounded-md border border-border/40 text-muted-foreground bg-muted/40">
                        {editCount} edits
                    </span>
                </div>
            </div>

            <div className="flex-1 min-h-0 rounded-lg overflow-hidden border border-border/40">
                {!ready ? (
                    // Placeholder keeps the layout stable while we wait one frame
                    <div className="flex items-center justify-center h-full bg-[#1e1e1e] text-xs text-slate-500">
                        Initialising Monaco…
                    </div>
                ) : (
                    <Editor
                        key={lang}
                        height="100%"
                        language={current.id}
                        defaultValue={current.content}
                        theme="vs-dark"
                        onMount={handleMount}
                        onChange={() => setEditCount(c => c + 1)}
                        options={{
                            minimap: { enabled: false },
                            fontSize: 13,
                            fontFamily: 'Fira Code, Cascadia Code, Consolas, monospace',
                            lineNumbers: 'on',
                            scrollBeyondLastLine: false,
                            automaticLayout: true,
                            padding: { top: 12, bottom: 12 },
                            tabSize: 4,
                            wordWrap: 'on',
                        }}
                    />
                )}
            </div>
        </div>
    );
}
