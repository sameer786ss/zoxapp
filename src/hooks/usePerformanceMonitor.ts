//! Performance Monitoring Hook
//! 
//! Tracks render performance and long tasks.

import { useEffect, useRef } from 'react';

interface PerformanceMetric {
    name: string;
    duration: number;
    timestamp: number;
}

// Store metrics in memory for debugging
const metricsBuffer: PerformanceMetric[] = [];
const MAX_METRICS = 100;

function addMetric(metric: PerformanceMetric) {
    metricsBuffer.push(metric);
    if (metricsBuffer.length > MAX_METRICS) {
        metricsBuffer.shift();
    }

    // Log slow operations
    if (metric.duration > 100) {
        console.warn(`[Perf] Slow operation: ${metric.name} took ${metric.duration.toFixed(2)}ms`);
    }
}

// Get all metrics for debugging
export function getPerformanceMetrics(): PerformanceMetric[] {
    return [...metricsBuffer];
}

// Clear metrics buffer
export function clearPerformanceMetrics(): void {
    metricsBuffer.length = 0;
}

/**
 * Hook to monitor long tasks and performance
 */
export function usePerformanceMonitor(enabled = true) {
    const observerRef = useRef<PerformanceObserver | null>(null);

    useEffect(() => {
        if (!enabled || typeof PerformanceObserver === 'undefined') {
            return;
        }

        try {
            // Monitor long tasks (> 50ms)
            observerRef.current = new PerformanceObserver((list) => {
                for (const entry of list.getEntries()) {
                    addMetric({
                        name: entry.name || entry.entryType,
                        duration: entry.duration,
                        timestamp: entry.startTime,
                    });
                }
            });

            // Observe longtask and measure entries
            observerRef.current.observe({ entryTypes: ['longtask', 'measure'] });
        } catch (e) {
            // PerformanceObserver may not support all entry types
            console.debug('[Perf] PerformanceObserver setup failed:', e);
        }

        return () => {
            observerRef.current?.disconnect();
        };
    }, [enabled]);
}

/**
 * Measure execution time of a function
 */
export function measureAsync<T>(name: string, fn: () => Promise<T>): Promise<T> {
    const start = performance.now();
    return fn().finally(() => {
        const duration = performance.now() - start;
        addMetric({ name, duration, timestamp: start });
    });
}

/**
 * Measure execution time of a sync function
 */
export function measureSync<T>(name: string, fn: () => T): T {
    const start = performance.now();
    try {
        return fn();
    } finally {
        const duration = performance.now() - start;
        addMetric({ name, duration, timestamp: start });
    }
}

export default usePerformanceMonitor;
