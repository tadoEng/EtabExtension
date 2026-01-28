import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Box } from 'lucide-react';
import { BuildingModel } from './BuildingModel';
import { Canvas } from '@react-three/fiber';
import { OrbitControls, Grid, Environment } from '@react-three/drei';

export function BuildingScene() {
    return (
        <Card className="h-full border-border/50 flex flex-col">
            <CardHeader>
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <Box className="w-5 h-5 text-primary" />
                        <div>
                            <CardTitle className="text-sm">3D Building Renderer</CardTitle>
                            <CardDescription>Interactive architectural visualization</CardDescription>
                        </div>
                    </div>
                    <div className="flex gap-2">
                        <div className="text-xs bg-primary/20 text-primary px-2 py-1 rounded">
                            Left Click: Rotate
                        </div>
                        <div className="text-xs bg-primary/20 text-primary px-2 py-1 rounded">
                            Right Click: Pan
                        </div>
                    </div>
                </div>
            </CardHeader>
            <CardContent className="flex-1 p-0 border-t border-border/50 bg-[#1e1e1e] relative">
                <Canvas
                    gl={{
                        powerPreference: "high-performance",
                        antialias: true,
                    }}
                    dpr={[1, 2]}
                    shadows
                    camera={{ position: [5, 5, 5], fov: 50 }}
                >
                    <ambientLight intensity={0.5} />
                    <directionalLight position={[10, 10, 5]} intensity={1} castShadow />
                    <Environment preset="city" />
                    <OrbitControls makeDefault minPolarAngle={0} maxPolarAngle={Math.PI / 1.75} />
                    <group position={[0, -1, 0]}>
                        <Grid infiniteGrid fadeDistance={30} sectionColor="#4d4d4d" cellColor="#333" />
                        <BuildingModel />
                    </group>
                </Canvas>
            </CardContent>
        </Card>
    );
}