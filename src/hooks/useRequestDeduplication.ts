//! Request Deduplication Hook
//! 
//! Prevents duplicate requests from being sent.

import { useRef, useCallback } from 'react';

interface RequestState {
    isSubmitting: boolean;
    lastRequestTime: number;
}

const MIN_REQUEST_INTERVAL_MS = 500; // Minimum time between requests

/**
 * Hook to prevent duplicate requests
 * 
 * @example
 * const { isSubmitting, executeRequest } = useRequestDeduplication();
 * 
 * const handleSubmit = () => {
 *   executeRequest(async () => {
 *     await someAsyncOperation();
 *   });
 * };
 */
export function useRequestDeduplication() {
    const stateRef = useRef<RequestState>({
        isSubmitting: false,
        lastRequestTime: 0,
    });

    const executeRequest = useCallback(async <T>(
        fn: () => Promise<T>,
        options?: { minInterval?: number }
    ): Promise<T | null> => {
        const now = Date.now();
        const minInterval = options?.minInterval ?? MIN_REQUEST_INTERVAL_MS;

        // Check if already submitting
        if (stateRef.current.isSubmitting) {
            console.debug('[RequestDedup] Request blocked: already submitting');
            return null;
        }

        // Check minimum interval
        if (now - stateRef.current.lastRequestTime < minInterval) {
            console.debug('[RequestDedup] Request blocked: too soon after last request');
            return null;
        }

        stateRef.current.isSubmitting = true;
        stateRef.current.lastRequestTime = now;

        try {
            return await fn();
        } finally {
            stateRef.current.isSubmitting = false;
        }
    }, []);

    return {
        isSubmitting: stateRef.current.isSubmitting,
        executeRequest,
    };
}

/**
 * Simple debounced submit wrapper
 */
export function useDebouncedSubmit<T extends (...args: unknown[]) => Promise<unknown>>(
    fn: T,
    delay = 300
) {
    const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
    const isSubmittingRef = useRef(false);

    const debouncedFn = useCallback((...args: Parameters<T>) => {
        if (isSubmittingRef.current) {
            return;
        }

        if (timeoutRef.current) {
            clearTimeout(timeoutRef.current);
        }

        timeoutRef.current = setTimeout(async () => {
            isSubmittingRef.current = true;
            try {
                await fn(...args);
            } finally {
                isSubmittingRef.current = false;
            }
        }, delay);
    }, [fn, delay]);

    return debouncedFn;
}

export default useRequestDeduplication;
