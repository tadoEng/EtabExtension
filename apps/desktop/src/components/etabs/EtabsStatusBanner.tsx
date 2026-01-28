import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { CheckCircle2, AlertCircle } from 'lucide-react';

export function EtabsStatusBanner() {
    const [cliAvailable, setCliAvailable] = useState<boolean | null>(null);
    const [cliVersion, setCliVersion] = useState<string | null>(null);

    useEffect(() => {
        checkCLI();
    }, []);

    const checkCLI = async () => {
        try {
            const available = await invoke<boolean>('check_cli_available');
            setCliAvailable(available);

            if (available) {
                try {
                    const version = await invoke<string>('get_cli_version');
                    setCliVersion(version);
                } catch (err) {
                    console.error('Failed to get CLI version:', err);
                }
            }
        } catch (err) {
            console.error('Failed to check CLI availability:', err);
            setCliAvailable(false);
        }
    };

    if (cliAvailable === null) {
        return null; // Loading, don't show anything
    }

    if (!cliAvailable) {
        return (
            <Alert variant="destructive" className="mb-6">
                <AlertCircle className="h-4 w-4" />
                <AlertTitle>CLI Not Available</AlertTitle>
                <AlertDescription>
                    The ETABS CLI sidecar is not available. Please ensure it's properly installed.
                </AlertDescription>
            </Alert>
        );
    }

    return (
        <Alert className="mb-6 border-green-200 bg-green-50">
            <CheckCircle2 className="h-4 w-4 text-green-600" />
            <AlertTitle className="text-green-900">CLI Ready</AlertTitle>
            <AlertDescription className="text-green-800">
                {cliVersion ? `Version: ${cliVersion}` : 'CLI is available and ready to use'}
            </AlertDescription>
        </Alert>
    );
}