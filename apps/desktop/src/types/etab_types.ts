/**
 * Type definitions for ETABS CLI integration
 * These types match the C# Result<T> pattern from your CLI
 */

/**
 * Base result type from CLI
 * All CLI commands return this structure
 */
export interface CliResult<T = unknown> {
    /** Whether the operation succeeded */
    success: boolean;

    /** Error message if operation failed */
    error?: string;

    /** ISO 8601 timestamp when operation completed */
    timestamp: string;

    /** Operation-specific data (only present if success = true) */
    data?: T;
}

/**
 * Validation command data
 * Returned by: validate_etabs_file
 */
export interface ValidationData {
    /** Whether ETABS is installed and running */
    etabsInstalled: boolean;

    /** ETABS version string (e.g., "22.7.0") */
    etabsVersion?: string;

    /** Whether the file is a valid ETABS file */
    fileValid?: boolean;

    /** Full path to the validated file */
    filePath?: string;

    /** Whether the file exists on disk */
    fileExists?: boolean;

    /** File extension (e.g., ".edb", ".e2k") */
    fileExtension?: string;

    /** Whether the model has been analyzed in ETABS */
    isAnalyzed?: boolean;

    /** Array of validation messages from the process */
    validationMessages: string[];
}

/**
 * Generate E2K command data
 * Returned by: generate_e2k
 */
export interface GenerateE2KData {
    /** Path to the input .edb file */
    inputFile: string;

    /** Path to the generated .e2k file */
    outputFile?: string;

    /** Whether input file exists */
    fileExists: boolean;

    /** Input file extension */
    fileExtension?: string;

    /** Whether output file existed before generation */
    outputExists?: boolean;

    /** Whether E2K generation succeeded */
    generationSuccessful?: boolean;

    /** Size of generated file in bytes */
    fileSizeBytes?: number;

    /** Time taken to generate in milliseconds */
    generationTimeMs?: number;

    /** Array of process messages/logs */
    messages: string[];
}

/**
 * Type-safe result types for each command
 */
export type ValidationResult = CliResult<ValidationData>;
export type GenerateE2KResult = CliResult<GenerateE2KData>;

/**
 * Tauri command signatures
 * Use these for type-safe invoke calls
 */
export interface EtabsCommands {
    /**
     * Validate ETABS installation and file
     * @param filePath - Path to ETABS file (.edb or .e2k)
     * @returns Promise resolving to validation result
     */
    validate_etabs_file(filePath: string): Promise<ValidationResult>;

    /**
     * Generate .e2k file from .edb
     * @param inputFile - Path to input .edb file
     * @param outputFile - Optional path for output .e2k file
     * @param overwrite - Whether to overwrite existing output file
     * @returns Promise resolving to generation result
     */
    generate_e2k(
        inputFile: string,
        outputFile: string | null,
        overwrite: boolean
    ): Promise<GenerateE2KResult>;

    /**
     * Check if CLI sidecar is available
     * @returns Promise resolving to availability status
     */
    check_cli_available(): Promise<boolean>;

    /**
     * Get CLI version string
     * @returns Promise resolving to version string
     */
    get_cli_version(): Promise<string>;
}

/**
 * Type-safe wrapper for Tauri invoke
 * Usage:
 * ```ts
 * import { invokeCommand } from '@/lib/tauri';
 *
 * const result = await invokeCommand('validate_etabs_file', {
 *   filePath: 'C:\\test.edb'
 * });
 * ```
 */
export async function invokeCommand<K extends keyof EtabsCommands>(
    command: K,
    args?: Parameters<EtabsCommands[K]>[0] extends undefined
        ? never
        : Record<string, unknown>
): Promise<ReturnType<EtabsCommands[K]>> {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke(command, args) as ReturnType<EtabsCommands[K]>;
}