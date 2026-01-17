//! Streaming Progress Indicator Component
//!
//! Displays real-time streaming progress with token count and elapsed time.

import { memo, useEffect, useState } from 'react';
import { Sparkles, Clock, Zap } from 'lucide-react';

interface StreamingProgressProps {
    isStreaming: boolean;
    content?: string;
}

/**
 * Streaming Progress Indicator
 * Shows visual feedback during model generation
 */
export const StreamingProgress = memo(function StreamingProgress({
    isStreaming,
    content = ''
}: StreamingProgressProps) {
    const [startTime, setStartTime] = useState<number | null>(null);
    const [elapsed, setElapsed] = useState(0);

    // Track start time when streaming begins
    useEffect(() => {
        if (isStreaming && !startTime) {
            setStartTime(Date.now());
        } else if (!isStreaming) {
            setStartTime(null);
            setElapsed(0);
        }
    }, [isStreaming, startTime]);

    // Update elapsed time every 100ms
    useEffect(() => {
        if (!isStreaming || !startTime) return;

        const interval = setInterval(() => {
            setElapsed(Date.now() - startTime);
        }, 100);

        return () => clearInterval(interval);
    }, [isStreaming, startTime]);

    if (!isStreaming) return null;

    // Estimate tokens (rough approximation: 4 chars per token)
    const estimatedTokens = Math.floor(content.length / 4);
    const tokensPerSecond = elapsed > 0 ? (estimatedTokens / (elapsed / 1000)).toFixed(1) : '0';
    const elapsedSeconds = (elapsed / 1000).toFixed(1);

    return (
        <div className="flex items-center gap-3 px-3 py-1.5 text-xs text-muted-foreground bg-muted/30 rounded-lg animate-in fade-in duration-200">
            <div className="flex items-center gap-1.5">
                <Sparkles className="w-3 h-3 animate-pulse text-primary" />
                <span className="font-medium">Generating</span>
            </div>

            <div className="flex items-center gap-1 text-muted-foreground/70">
                <Zap className="w-3 h-3" />
                <span>{estimatedTokens} tokens</span>
            </div>

            <div className="flex items-center gap-1 text-muted-foreground/70">
                <Clock className="w-3 h-3" />
                <span>{elapsedSeconds}s</span>
            </div>

            <div className="text-muted-foreground/50">
                ({tokensPerSecond} tok/s)
            </div>
        </div>
    );
});

export default StreamingProgress;
