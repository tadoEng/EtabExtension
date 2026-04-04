import { lazy, Suspense, useState, useRef } from 'react';
import { useAnalysisStore } from './analysisStore';
import { TailwindTest } from './TailwindTest';

// Dynamic imports — Rolldown chunk splitting test
const ThreeScene = lazy(() =>
    import('./ThreeScene').then((m) => ({ default: m.ThreeScene }))
);
const EChartsPanel = lazy(() =>
    import('./EChartsPanel').then((m) => ({ default: m.EChartsPanel }))
);
const MonacoTest = lazy(() =>
    import('./MonacoTest').then((m) => ({ default: m.MonacoTest }))
);

interface WorkerResult {
    result: string;
    primeCount: number;
    largestPrime: number;
}

function ChunkFallback({ label }: { label: string }) {
    return (
        <div className="flex items-center justify-center h-full min-h-48 text-xs text-muted-foreground border border-border/40 rounded-lg">
            Loading {label} chunk…
        </div>
    );
}

function LazyTest() {
    const [show3d, setShow3d] = useState(false);
    const [showCharts, setShowCharts] = useState(false);
    const [showMonaco, setShowMonaco] = useState(false);

    const anyShown = show3d || showCharts || showMonaco;

    return (
        <div className="space-y-3 h-full flex flex-col">
            <p className="text-xs text-muted-foreground">
                Each button triggers a <code className="text-xs bg-muted px-1 rounded">React.lazy()</code> dynamic import.
                Rolldown must correctly split Three.js, ECharts, and Monaco into separate async chunks
                and resolve them at runtime inside the Tauri WebView.
            </p>
            <div className="flex gap-2 flex-wrap">
                {[
                    { label: '3D scene (Three.js)', active: show3d,    toggle: () => setShow3d(v => !v)    },
                    { label: 'ECharts',             active: showCharts, toggle: () => setShowCharts(v => !v)},
                    { label: 'Monaco editor',       active: showMonaco, toggle: () => setShowMonaco(v => !v)},
                ].map(({ label, active, toggle }) => (
                    <button
                        key={label}
                        onClick={toggle}
                        className={`px-3 py-1.5 text-xs rounded-md border transition-colors ${
                            active
                                ? 'bg-primary/10 border-primary/40 text-primary'
                                : 'border-border/60 hover:bg-accent'
                        }`}
                    >
                        {active ? 'Unload' : 'Load'} {label}
                    </button>
                ))}
            </div>

            {anyShown && (
                <div
                    className="flex-1 min-h-0 grid gap-3"
                    style={{
                        gridTemplateColumns:
                            [show3d, showCharts, showMonaco].filter(Boolean).length > 1
                                ? '1fr 1fr'
                                : '1fr',
                    }}
                >
                    {show3d && (
                        <Suspense fallback={<ChunkFallback label="Three.js" />}>
                            <ThreeScene />
                        </Suspense>
                    )}
                    {showCharts && (
                        <Suspense fallback={<ChunkFallback label="ECharts" />}>
                            <EChartsPanel />
                        </Suspense>
                    )}
                    {showMonaco && (
                        <Suspense fallback={<ChunkFallback label="Monaco" />}>
                            <MonacoTest />
                        </Suspense>
                    )}
                </div>
            )}
        </div>
    );
}

function WorkerTest() {
    const [status, setStatus] = useState<'idle' | 'running' | 'done' | 'error'>('idle');
    const [result, setResult] = useState<WorkerResult | null>(null);
    const [elapsed, setElapsed] = useState<number | null>(null);
    const workerRef = useRef<Worker | null>(null);

    const run = () => {
        setStatus('running');
        setResult(null);
        const start = performance.now();

        const worker = new Worker(
            new URL('./heavy.worker.ts', import.meta.url),
            { type: 'module' }
        );
        workerRef.current = worker;

        worker.onmessage = (e: MessageEvent<WorkerResult>) => {
            setElapsed(Math.round(performance.now() - start));
            setResult(e.data);
            setStatus('done');
            worker.terminate();
        };

        worker.onerror = () => {
            setStatus('error');
            worker.terminate();
        };

        worker.postMessage({ n: 5_000_000 });
    };

    return (
        <div className="space-y-3">
            <p className="text-xs text-muted-foreground">
                Spawns a <code className="text-xs bg-muted px-1 rounded">Web Worker</code> via{' '}
                <code className="text-xs bg-muted px-1 rounded">new URL(..., import.meta.url)</code>.
                Rolldown must bundle the worker file as a separate entry point.
            </p>
            <button
                onClick={run}
                disabled={status === 'running'}
                className="px-3 py-1.5 text-xs rounded-md border border-border/60 hover:bg-accent transition-colors disabled:opacity-50"
            >
                {status === 'running' ? 'Running…' : 'Run heavy worker (5M iterations)'}
            </button>
            {status === 'done' && result && (
                <div className="grid grid-cols-4 gap-2 mt-2">
                    {[
                        { label: 'Result',        value: result.result },
                        { label: 'Primes found',  value: result.primeCount },
                        { label: 'Largest prime', value: result.largestPrime.toLocaleString() },
                        { label: 'Time',          value: `${elapsed}ms` },
                    ].map(({ label, value }) => (
                        <div key={label} className="bg-muted/40 rounded-lg p-2 border border-border/40">
                            <p className="text-xs text-muted-foreground">{label}</p>
                            <p className="text-sm font-medium mt-0.5">{value}</p>
                        </div>
                    ))}
                </div>
            )}
            {status === 'error' && (
                <p className="text-xs text-destructive">
                    Worker failed — Rolldown may not have bundled it correctly.
                </p>
            )}
        </div>
    );
}

function ZustandImmerTest() {
    const {
        results, selectedStory, filterExceedingLimit, driftLimit,
        setSelectedStory, toggleFilter, setDriftLimit, reset,
    } = useAnalysisStore();

    const displayed = filterExceedingLimit
        ? results.filter(r => r.driftX > driftLimit || r.driftY > driftLimit)
        : results;

    return (
        <div className="space-y-3">
            <p className="text-xs text-muted-foreground">
                Zustand store with <code className="text-xs bg-muted px-1 rounded">immer</code> middleware.
                Tests Rolldown tree-shaking of proxy-based libs.
            </p>
            <div className="flex items-center gap-3 flex-wrap">
                <button
                    onClick={toggleFilter}
                    className={`px-3 py-1.5 text-xs rounded-md border transition-colors ${
                        filterExceedingLimit
                            ? 'bg-destructive/10 border-destructive/40 text-destructive'
                            : 'border-border/60 hover:bg-accent'
                    }`}
                >
                    {filterExceedingLimit ? 'Showing: exceeding limit' : 'Show all stories'}
                </button>
                <div className="flex items-center gap-2">
                    <span className="text-xs text-muted-foreground">Drift limit:</span>
                    <input
                        type="range" min="0.1" max="1.0" step="0.05"
                        value={driftLimit}
                        onChange={e => setDriftLimit(parseFloat(e.target.value))}
                        className="w-24"
                    />
                    <span className="text-xs font-medium w-8">{driftLimit.toFixed(2)}</span>
                </div>
                <button
                    onClick={reset}
                    className="px-3 py-1.5 text-xs rounded-md border border-border/60 hover:bg-accent transition-colors ml-auto"
                >
                    Reset
                </button>
            </div>
            <div className="border border-border/40 rounded-lg overflow-hidden">
                <table className="w-full text-xs">
                    <thead>
                        <tr className="border-b border-border/40 bg-muted/30">
                            {['Story', 'Drift X', 'Drift Y', 'Shear X', 'Shear Y'].map(h => (
                                <th key={h} className="text-left px-3 py-2 font-medium text-muted-foreground">{h}</th>
                            ))}
                        </tr>
                    </thead>
                    <tbody>
                        {displayed.map(r => {
                            const exceeds = r.driftX > driftLimit || r.driftY > driftLimit;
                            const isSelected = selectedStory === r.story;
                            return (
                                <tr
                                    key={r.story}
                                    onClick={() => setSelectedStory(isSelected ? null : r.story)}
                                    className={`border-b border-border/30 cursor-pointer transition-colors ${
                                        isSelected
                                            ? 'bg-primary/10'
                                            : exceeds
                                                ? 'bg-destructive/5 hover:bg-destructive/10'
                                                : 'hover:bg-muted/30'
                                    }`}
                                >
                                    <td className="px-3 py-2 font-medium">{r.story}</td>
                                    <td className={`px-3 py-2 ${r.driftX > driftLimit ? 'text-destructive font-medium' : ''}`}>{r.driftX.toFixed(2)}</td>
                                    <td className={`px-3 py-2 ${r.driftY > driftLimit ? 'text-destructive font-medium' : ''}`}>{r.driftY.toFixed(2)}</td>
                                    <td className="px-3 py-2">{r.shearX.toLocaleString()}</td>
                                    <td className="px-3 py-2">{r.shearY.toLocaleString()}</td>
                                </tr>
                            );
                        })}
                    </tbody>
                </table>
                {displayed.length === 0 && (
                    <p className="text-xs text-muted-foreground text-center py-4">
                        No stories exceed the drift limit.
                    </p>
                )}
            </div>
            {selectedStory && (
                <p className="text-xs text-muted-foreground">
                    Selected: <span className="font-medium text-foreground">{selectedStory}</span> — immer mutation fired correctly.
                </p>
            )}
        </div>
    );
}

function MonacoStandaloneTest() {
    return (
        <div className="h-full flex flex-col">
            <Suspense fallback={<ChunkFallback label="Monaco" />}>
                <MonacoTest />
            </Suspense>
        </div>
    );
}

type TestTab = 'lazy' | 'worker' | 'zustand' | 'monaco' | 'tailwind';

export function SandboxPanel() {
    const [tab, setTab] = useState<TestTab>('tailwind');

    const tabs: { id: TestTab; label: string; desc: string }[] = [
        { id: 'lazy',     label: 'Dynamic imports', desc: 'React.lazy chunk splitting'  },
        { id: 'worker',   label: 'Web Worker',       desc: 'new URL worker bundling'    },
        { id: 'zustand',  label: 'Zustand + Immer',  desc: 'proxy tree-shaking'         },
        { id: 'monaco',   label: 'Monaco editor',    desc: 'AMD + worker isolation'     },
        { id: 'tailwind', label: 'Tailwind v4',      desc: 'CSS scan under Rolldown'    },
    ];

    return (
        <div className="flex flex-col h-full p-4 gap-4">
            <div>
                <h2 className="text-sm font-medium text-foreground">Vite 8 / Rolldown stress tests</h2>
                <p className="text-xs text-muted-foreground mt-0.5">
                    Three.js + ECharts passed. Testing chunk splitting, workers, tree-shaking, Monaco AMD, and Tailwind v4 CSS scanning.
                </p>
            </div>

            <div className="flex gap-1 bg-muted/40 p-1 rounded-lg border border-border/40 w-fit">
                {tabs.map(t => (
                    <button
                        key={t.id}
                        onClick={() => setTab(t.id)}
                        className={`px-3 py-1.5 text-xs rounded-md transition-colors flex flex-col items-start ${
                            tab === t.id
                                ? 'bg-primary text-primary-foreground'
                                : 'text-muted-foreground hover:text-foreground'
                        }`}
                    >
                        <span className="font-medium">{t.label}</span>
                        <span className={`text-[10px] ${tab === t.id ? 'text-primary-foreground/70' : 'text-muted-foreground/70'}`}>
                            {t.desc}
                        </span>
                    </button>
                ))}
            </div>

            <div className="flex-1 overflow-auto min-h-0">
                {tab === 'lazy'     && <LazyTest />}
                {tab === 'worker'   && <WorkerTest />}
                {tab === 'zustand'  && <ZustandImmerTest />}
                {tab === 'monaco'   && <MonacoStandaloneTest />}
                {tab === 'tailwind' && <TailwindTest />}
            </div>
        </div>
    );
}
