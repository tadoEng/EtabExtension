import { useState } from 'react';

// Tests Tailwind v4 + Rolldown: dynamically constructed class strings,
// conditional classes, animation utilities, and arbitrary values.
// If Rolldown's CSS scanning drops any classes, they'll be visually missing
// in the production build but work fine in dev (where Tailwind scans on-demand).

const COLORS = ['blue', 'green', 'red', 'amber', 'purple', 'pink'] as const;
type Color = typeof COLORS[number];

// Explicit full class strings — never construct partial strings like `bg-${color}-500`
// because Tailwind's scanner needs the full token present in source.
const COLOR_MAP: Record<Color, { bg: string; border: string; text: string; ring: string }> = {
    blue:   { bg: 'bg-blue-500',   border: 'border-blue-400',   text: 'text-blue-400',   ring: 'ring-blue-500'   },
    green:  { bg: 'bg-green-500',  border: 'border-green-400',  text: 'text-green-400',  ring: 'ring-green-500'  },
    red:    { bg: 'bg-red-500',    border: 'border-red-400',    text: 'text-red-400',    ring: 'ring-red-500'    },
    amber:  { bg: 'bg-amber-500',  border: 'border-amber-400',  text: 'text-amber-400',  ring: 'ring-amber-500'  },
    purple: { bg: 'bg-purple-500', border: 'border-purple-400', text: 'text-purple-400', ring: 'ring-purple-500' },
    pink:   { bg: 'bg-pink-500',   border: 'border-pink-400',   text: 'text-pink-400',   ring: 'ring-pink-500'   },
};

const SIZES = [
    { label: 'xs',  cls: 'w-8 h-8 text-xs'    },
    { label: 'sm',  cls: 'w-12 h-12 text-sm'  },
    { label: 'md',  cls: 'w-16 h-16 text-base'},
    { label: 'lg',  cls: 'w-20 h-20 text-lg'  },
    { label: 'xl',  cls: 'w-24 h-24 text-xl'  },
];

const ANIMATIONS = [
    { label: 'none',    cls: ''                 },
    { label: 'spin',    cls: 'animate-spin'     },
    { label: 'ping',    cls: 'animate-ping'     },
    { label: 'pulse',   cls: 'animate-pulse'    },
    { label: 'bounce',  cls: 'animate-bounce'   },
];

const TRANSITIONS = [
    { label: 'none',    cls: ''                                              },
    { label: 'all',     cls: 'transition-all duration-300'                  },
    { label: 'colors',  cls: 'transition-colors duration-300'               },
    { label: 'spring',  cls: 'transition-all duration-500 ease-in-out'      },
    { label: 'slow',    cls: 'transition-all duration-1000 ease-out'        },
];

export function TailwindTest() {
    const [color, setColor]       = useState<Color>('blue');
    const [size, setSize]         = useState(2);        // index into SIZES
    const [anim, setAnim]         = useState(0);        // index into ANIMATIONS
    const [trans, setTrans]       = useState(1);        // index into TRANSITIONS
    const [rounded, setRounded]   = useState(false);
    const [shadow, setShadow]     = useState(false);
    const [opacity, setOpacity]   = useState(100);
    const [rotate, setRotate]     = useState(0);

    const c = COLOR_MAP[color];
    const s = SIZES[size];
    const a = ANIMATIONS[anim];
    const t = TRANSITIONS[trans];

    // Explicit rotation classes — no dynamic construction
    const rotateClass =
        rotate === 0   ? 'rotate-0'   :
        rotate === 45  ? 'rotate-45'  :
        rotate === 90  ? 'rotate-90'  :
        rotate === 135 ? 'rotate-[135deg]' :
        rotate === 180 ? 'rotate-180' : 'rotate-0';

    const opacityClass =
        opacity === 100 ? 'opacity-100' :
        opacity === 75  ? 'opacity-75'  :
        opacity === 50  ? 'opacity-50'  :
        opacity === 25  ? 'opacity-25'  : 'opacity-10';

    return (
        <div className="space-y-5">
            <p className="text-xs text-muted-foreground">
                Tests Tailwind v4 class generation under Rolldown. All classes must be full
                string literals in source — no dynamic construction — so the scanner can find them.
                Run <code className="text-xs bg-muted px-1 rounded">pnpm tauri build</code> and
                compare the production binary: if any classes are missing, Rolldown's CSS module
                scan order differs from Rollup's.
            </p>

            {/* Preview box */}
            <div className="flex items-center justify-center py-8 bg-muted/20 rounded-xl border border-border/40">
                <div
                    className={[
                        s.cls,
                        c.bg,
                        a.cls,
                        t.cls,
                        rotateClass,
                        opacityClass,
                        rounded ? 'rounded-full' : 'rounded-lg',
                        shadow  ? 'shadow-2xl'   : '',
                        'flex items-center justify-center font-medium text-white',
                        'ring-2',
                        c.ring,
                        'ring-offset-2 ring-offset-background',
                    ].join(' ')}
                >
                    Tw
                </div>
            </div>

            {/* Controls grid */}
            <div className="grid grid-cols-2 gap-x-8 gap-y-4">

                {/* Color */}
                <div className="space-y-2">
                    <p className="text-xs font-medium text-muted-foreground">Color</p>
                    <div className="flex flex-wrap gap-2">
                        {COLORS.map(col => (
                            <button
                                key={col}
                                onClick={() => setColor(col)}
                                className={[
                                    'w-6 h-6 rounded-md border-2 transition-transform',
                                    COLOR_MAP[col].bg,
                                    color === col ? 'border-white scale-110' : 'border-transparent',
                                ].join(' ')}
                                title={col}
                            />
                        ))}
                    </div>
                </div>

                {/* Size */}
                <div className="space-y-2">
                    <p className="text-xs font-medium text-muted-foreground">
                        Size — <span className={`${c.text}`}>{SIZES[size].label}</span>
                    </p>
                    <input type="range" min={0} max={4} step={1} value={size}
                        onChange={e => setSize(+e.target.value)} className="w-full" />
                </div>

                {/* Animation */}
                <div className="space-y-2">
                    <p className="text-xs font-medium text-muted-foreground">Animation</p>
                    <div className="flex flex-wrap gap-1">
                        {ANIMATIONS.map((a, i) => (
                            <button key={a.label} onClick={() => setAnim(i)}
                                className={`px-2 py-1 text-xs rounded-md border transition-colors ${
                                    anim === i
                                        ? `${c.border} ${c.text} bg-muted`
                                        : 'border-border/40 text-muted-foreground hover:bg-muted/40'
                                }`}>
                                {a.label}
                            </button>
                        ))}
                    </div>
                </div>

                {/* Transition */}
                <div className="space-y-2">
                    <p className="text-xs font-medium text-muted-foreground">Transition</p>
                    <div className="flex flex-wrap gap-1">
                        {TRANSITIONS.map((t, i) => (
                            <button key={t.label} onClick={() => setTrans(i)}
                                className={`px-2 py-1 text-xs rounded-md border transition-colors ${
                                    trans === i
                                        ? `${c.border} ${c.text} bg-muted`
                                        : 'border-border/40 text-muted-foreground hover:bg-muted/40'
                                }`}>
                                {t.label}
                            </button>
                        ))}
                    </div>
                </div>

                {/* Opacity */}
                <div className="space-y-2">
                    <p className="text-xs font-medium text-muted-foreground">
                        Opacity — <span className={c.text}>{opacity}%</span>
                    </p>
                    <input type="range" min={0} max={4} step={1}
                        value={[100,75,50,25,10].indexOf(opacity) === -1 ? 0 : [100,75,50,25,10].indexOf(opacity)}
                        onChange={e => setOpacity([100,75,50,25,10][+e.target.value])}
                        className="w-full" />
                </div>

                {/* Rotation */}
                <div className="space-y-2">
                    <p className="text-xs font-medium text-muted-foreground">
                        Rotation — <span className={c.text}>{rotate}°</span>
                    </p>
                    <input type="range" min={0} max={4} step={1}
                        value={[0,45,90,135,180].indexOf(rotate) === -1 ? 0 : [0,45,90,135,180].indexOf(rotate)}
                        onChange={e => setRotate([0,45,90,135,180][+e.target.value])}
                        className="w-full" />
                </div>

                {/* Toggles */}
                <div className="space-y-2">
                    <p className="text-xs font-medium text-muted-foreground">Modifiers</p>
                    <div className="flex gap-3">
                        {[
                            { label: 'rounded-full', value: rounded, set: setRounded },
                            { label: 'shadow-2xl',   value: shadow,  set: setShadow  },
                        ].map(({ label, value, set }) => (
                            <button key={label} onClick={() => set(v => !v)}
                                className={`px-3 py-1.5 text-xs rounded-md border transition-colors ${
                                    value
                                        ? `${c.border} ${c.text} bg-muted`
                                        : 'border-border/40 text-muted-foreground hover:bg-muted/40'
                                }`}>
                                {label}
                            </button>
                        ))}
                    </div>
                </div>

                {/* Active class readout */}
                <div className="space-y-2">
                    <p className="text-xs font-medium text-muted-foreground">Active classes</p>
                    <p className="text-[11px] text-muted-foreground font-mono leading-5 break-all">
                        {[s.cls, c.bg, a.cls || '—', t.cls || '—', rotateClass, opacityClass,
                          rounded ? 'rounded-full' : 'rounded-lg',
                          shadow ? 'shadow-2xl' : '—'].filter(x => x !== '—').join(' ')}
                    </p>
                </div>
            </div>
        </div>
    );
}
