import { motion } from 'framer-motion';
import { Loader2, Terminal, Zap } from 'lucide-react';
import { cn } from '@/lib/utils';

interface ThinkingIndicatorProps {
    message?: string;
    className?: string;
}

/**
 * Premium thinking indicator with shimmer effect
 * No dots - just smooth shimmer animation
 */
export default function ThinkingIndicator({ message, className }: ThinkingIndicatorProps) {
    return (
        <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className={cn(
                "flex items-center gap-3 px-4 py-3",
                className
            )}
        >
            <Zap className="w-4 h-4 text-primary animate-pulse" />

            {/* Premium shimmer text - larger, no dots */}
            <span className="text-base font-medium bg-gradient-to-r from-zinc-400 via-white to-zinc-400 bg-clip-text text-transparent bg-[length:200%_100%] animate-shimmer">
                {message || 'Thinking'}
            </span>
        </motion.div>
    );
}

// Executing indicator - shows when a tool is being executed
interface ExecutingIndicatorProps {
    toolName?: string;
    className?: string;
}

export function ExecutingIndicator({ toolName, className }: ExecutingIndicatorProps) {
    return (
        <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className={cn(
                "flex items-center gap-3 px-4 py-3",
                className
            )}
        >
            <div className="relative">
                <Terminal className="w-4 h-4 text-green-400" />
                <Loader2 className="w-3 h-3 text-green-400 animate-spin absolute -right-1 -bottom-1" />
            </div>

            {/* Premium shimmer text */}
            <span className="text-base font-medium bg-gradient-to-r from-green-400 via-emerald-200 to-green-400 bg-clip-text text-transparent bg-[length:200%_100%] animate-shimmer">
                Executing {toolName || 'tool'}
            </span>
        </motion.div>
    );
}
