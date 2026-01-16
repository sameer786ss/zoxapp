import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

export type UpdateStatus =
    | 'idle'
    | 'checking'
    | 'available'
    | 'downloading'
    | 'ready'
    | 'error'
    | 'up-to-date';

export interface UpdateInfo {
    version: string;
    currentVersion: string;
    releaseNotes?: string;
    releaseDate?: string;
    downloadSize?: number;
}

export interface UpdateProgress {
    downloaded: number;
    total: number;
    percent: number;
    speed: number; // bytes per second
}

export interface UpdateError {
    code: 'network' | 'signature' | 'install' | 'unknown';
    message: string;
    retryable: boolean;
}

interface UseUpdaterReturn {
    status: UpdateStatus;
    updateInfo: UpdateInfo | null;
    progress: UpdateProgress | null;
    error: UpdateError | null;
    checkForUpdates: () => Promise<void>;
    downloadUpdate: () => Promise<void>;
    installUpdate: () => Promise<void>;
    dismissUpdate: () => void;
    retryCount: number;
}

const MAX_RETRIES = 3;
const RETRY_DELAY_BASE = 2000; // 2 seconds, doubles each retry

export function useUpdater(): UseUpdaterReturn {
    const [status, setStatus] = useState<UpdateStatus>('idle');
    const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
    const [progress, setProgress] = useState<UpdateProgress | null>(null);
    const [error, setError] = useState<UpdateError | null>(null);
    const [retryCount, setRetryCount] = useState(0);

    // Refs for cleanup
    const unlistenProgressRef = useRef<UnlistenFn | null>(null);
    const unlistenErrorRef = useRef<UnlistenFn | null>(null);
    const retryTimeoutRef = useRef<number | null>(null);

    // Setup event listeners
    useEffect(() => {
        // Listen for download progress
        listen<UpdateProgress>('update-download-progress', (event) => {
            setProgress(event.payload);
        }).then(fn => { unlistenProgressRef.current = fn; });

        // Listen for update errors
        listen<UpdateError>('update-error', (event) => {
            setError(event.payload);
            setStatus('error');
        }).then(fn => { unlistenErrorRef.current = fn; });

        return () => {
            unlistenProgressRef.current?.();
            unlistenErrorRef.current?.();
            if (retryTimeoutRef.current) {
                clearTimeout(retryTimeoutRef.current);
            }
        };
    }, []);

    // Check for updates on mount (after a delay to not block startup)
    useEffect(() => {
        const timer = setTimeout(() => {
            checkForUpdates();
        }, 5000); // Check 5 seconds after mount

        return () => clearTimeout(timer);
    }, []);

    const handleError = useCallback((err: unknown, code: UpdateError['code'] = 'unknown') => {
        const message = err instanceof Error ? err.message : String(err);
        const isNetworkError = message.toLowerCase().includes('network') ||
            message.toLowerCase().includes('fetch') ||
            message.toLowerCase().includes('timeout');

        const updateError: UpdateError = {
            code: isNetworkError ? 'network' : code,
            message,
            retryable: isNetworkError && retryCount < MAX_RETRIES,
        };

        setError(updateError);
        setStatus('error');

        // Auto-retry for network errors
        if (updateError.retryable && retryCount < MAX_RETRIES) {
            const delay = RETRY_DELAY_BASE * Math.pow(2, retryCount);
            console.log(`[Updater] Retrying in ${delay}ms (attempt ${retryCount + 1}/${MAX_RETRIES})`);

            retryTimeoutRef.current = window.setTimeout(() => {
                setRetryCount(prev => prev + 1);
                checkForUpdates();
            }, delay);
        }
    }, [retryCount]);

    const checkForUpdates = useCallback(async () => {
        try {
            setStatus('checking');
            setError(null);

            const result = await invoke<UpdateInfo | null>('check_for_updates');

            if (result) {
                setUpdateInfo(result);
                setStatus('available');
                setRetryCount(0); // Reset retry count on success
            } else {
                setStatus('up-to-date');
                setRetryCount(0);
            }
        } catch (err) {
            console.error('[Updater] Check failed:', err);
            handleError(err);
        }
    }, [handleError]);

    const downloadUpdate = useCallback(async () => {
        if (!updateInfo) {
            console.warn('[Updater] No update available to download');
            return;
        }

        try {
            setStatus('downloading');
            setProgress({ downloaded: 0, total: 0, percent: 0, speed: 0 });
            setError(null);

            await invoke('download_update');

            setStatus('ready');
            setProgress(null);
        } catch (err) {
            console.error('[Updater] Download failed:', err);
            handleError(err);
        }
    }, [updateInfo, handleError]);

    const installUpdate = useCallback(async () => {
        if (status !== 'ready') {
            console.warn('[Updater] Update not ready to install');
            return;
        }

        try {
            // This will restart the app
            await invoke('install_update');
        } catch (err) {
            console.error('[Updater] Install failed:', err);
            handleError(err, 'install');
        }
    }, [status, handleError]);

    const dismissUpdate = useCallback(() => {
        setStatus('idle');
        setUpdateInfo(null);
        setProgress(null);
        setError(null);
    }, []);

    return {
        status,
        updateInfo,
        progress,
        error,
        checkForUpdates,
        downloadUpdate,
        installUpdate,
        dismissUpdate,
        retryCount,
    };
}
