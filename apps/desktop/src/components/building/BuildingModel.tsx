import { useRef } from 'react';
import { useFrame } from '@react-three/fiber';

export function BuildingModel() {
    const meshRef = useRef(null);

    useFrame((_state, delta) => {
        if (meshRef.current) {
            (meshRef.current as any).rotation.y += delta * 0.2;
        }
    });

    return (
        <group position={[0, 0, 0]}>
            <mesh ref={meshRef} position={[0, 2, 0]} castShadow receiveShadow>
                <boxGeometry args={[2, 4, 2]} />
                <meshStandardMaterial
                    color="#3b82f6"
                    roughness={0.1}
                    metalness={0.8}
                    transparent
                    opacity={0.9}
                />
            </mesh>

            <mesh ref={meshRef} position={[0, 2, 0]}>
                <boxGeometry args={[2.05, 4.05, 2.05]} />
                <meshBasicMaterial wireframe color="#60a5fa" />
            </mesh>

            <mesh position={[0, -0.1, 0]} receiveShadow>
                <cylinderGeometry args={[4, 4, 0.2, 32]} />
                <meshStandardMaterial color="#333" />
            </mesh>
        </group>
    );
}
