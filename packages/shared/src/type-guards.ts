import type { CliResult } from './types';

export function isCliSuccess<T>(result: CliResult<T>): result is CliResult<T> & { success: true; data: T } {
    return result.success && result.data !== null && result.data !== undefined;
}

export function isCliError<T>(result: CliResult<T>): result is CliResult<T> & { success: false; error: string } {
    return !result.success && result.error !== null && result.error !== undefined;
}