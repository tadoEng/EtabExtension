import { useRef, Component, ReactNode, useState, useEffect } from 'react';
import { Canvas, useFrame } from '@react-three/fiber';
import { OrbitControls, Grid, Text } from '@react-three/drei';
import * as THREE from 'three';

// Error boundary — catches any R3F/WebGL errors and shows a readable message
class CanvasErrorBoundary extends Component<
    { children: ReactNode },
    { error: Error | null }
> {
    state = { error: null };
    static getDerivedStateFromError(error: Error) { return { error }; }
    render() {
        if (this.state.error) {
            return (
                <div className="flex items-center justify-center h-full bg-slate-900 rounded-lg text-xs text-red-400 p-4">
                    WebGL error: {(this.state.error as Error).message}
                </div>
            );
        }
        return this.props.children;
    }
}

function Building({
    position, height, color,
}: {
    position: [number, number, number];
    height: number;
    color: string;
}) {
    const meshRef = useRef<THREE.Mesh>(null);
    useFrame((_, delta) => {
        if (meshRef.current) meshRef.current.rotation.y += delta * 0.2;
    });
    return (
        <mesh ref={meshRef} position={[position[0], height / 2, position[2]]} castShadow>
            <boxGeometry args={[1, height, 1]} />
            <meshStandardMaterial color={color} />
        </mesh>
    );
}

function Scene() {
    const buildings: { position: [number, number, number]; height: number; color: string }[] = [
        { position: [-3, 0, 0], height: 3, color: '#4f86c6' },
        { position: [-1.5, 0, 0], height: 5, color: '#6baed6' },
        { position: [0, 0, 0],  height: 7, color: '#2171b5' },
        { position: [1.5, 0, 0], height: 4, color: '#6baed6' },
        { position: [3, 0, 0],  height: 6, color: '#4f86c6' },
    ];
    return (
        <>
            <ambientLight intensity={0.5} />
            <directionalLight position={[10, 10, 5]} intensity={1} castShadow />
            <Grid
                args={[20, 20]}
                cellSize={1}
                cellThickness={0.5}
                cellColor="#6e6e6e"
                sectionSize={5}
                sectionThickness={1}
                sectionColor="#9d4b4b"
                fadeDistance={30}
                fadeStrength={1}
                followCamera={false}
                infiniteGrid
            />
            {buildings.map((b, i) => (
                <Building key={i} {...b} />
            ))}
            <Text position={[0, 9, 0]} fontSize={0.6} color="#ffffff" anchorX="center" anchorY="middle">
                Story Heights — 3D View
            </Text>
            <OrbitControls makeDefault />
        </>
    );
}

export function ThreeScene() {
    // Defer Canvas mount by one rAF tick so the container div has been
    // painted and has real pixel dimensions before WebGL context creation.
    // Without this, Canvas initialises with 0×0 on first React.lazy() load.
    const [ready, setReady] = useState(false);

    useEffect(() => {
        const id = requestAnimationFrame(() => setReady(true));
        return () => cancelAnimationFrame(id);
    }, []);

    return (
        <div className="w-full h-full rounded-lg overflow-hidden border border-border/40" style={{ minHeight: 240 }}>
            {!ready ? (
                <div className="flex items-center justify-center h-full bg-slate-900 text-xs text-slate-500">
                    Initialising WebGL…
                </div>
            ) : (
                <CanvasErrorBoundary>
                    <Canvas
                        camera={{ position: [8, 8, 8], fov: 50 }}
                        shadows
                        style={{ background: '#0f172a', width: '100%', height: '100%' }}
                    >
                        <Scene />
                    </Canvas>
                </CanvasErrorBoundary>
            )}
        </div>
    );
}
