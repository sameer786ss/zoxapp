import { useRef, useEffect, useCallback, memo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
    Send,
    Sparkles,
    Loader2,
    Shield,
    Check,
    X,
    Terminal,
    FileText,
    FilePlus,
    FileEdit,
    FolderTree,
    Search,
    Code2,
    Play,
    Keyboard
} from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
} from '@/components/ui/tooltip';

export type InputBoxState = 'idle' | 'typing' | 'thinking' | 'awaiting_approval' | 'executing' | 'model_loading';

interface SmartInputBoxProps {
    value: string;
    onChange: (value: string) => void;
    onSend: () => void;
    state: InputBoxState;
    mode: 'chat' | 'turbo';
    pendingToolName?: string | null;
    thinkingText?: string;
    modelLoadProgress?: number;  // 0-100 for loading progress
    isModelUnloading?: boolean;  // true when unloading model
    onApprove?: () => void;
    onDeny?: () => void;
    disabled?: boolean;
    className?: string;
}

// Tool icon mapping
const TOOL_ICONS: Record<string, { icon: React.ElementType; color: string; label: string }> = {
    read_file: { icon: FileText, color: 'text-blue-400', label: 'Read File' },
    write_file: { icon: FilePlus, color: 'text-orange-400', label: 'Write File' },
    replace_lines: { icon: FileEdit, color: 'text-yellow-400', label: 'Edit File' },
    run_terminal: { icon: Terminal, color: 'text-green-400', label: 'Run Command' },
    search_project: { icon: Search, color: 'text-purple-400', label: 'Search' },
    list_files: { icon: FolderTree, color: 'text-cyan-400', label: 'List Files' },
    default: { icon: Code2, color: 'text-muted-foreground', label: 'Tool' },
};

// State configurations
const STATE_CONFIG = {
    idle: {
        bg: 'bg-secondary/40',
        border: 'border-border/40',
        shadow: '',
        glow: '',
    },
    typing: {
        bg: 'bg-secondary/50',
        border: 'border-primary/30',
        shadow: 'shadow-lg shadow-primary/5',
        glow: '',
    },
    thinking: {
        bg: 'bg-blue-500/10 backdrop-blur-xl',
        border: 'border-blue-500/40',
        shadow: 'shadow-lg shadow-blue-500/10',
        glow: 'ring-2 ring-blue-500/20',
    },
    awaiting_approval: {
        bg: 'bg-gradient-to-r from-amber-500/15 to-orange-500/15 backdrop-blur-xl',
        border: 'border-amber-500/50',
        shadow: 'shadow-xl shadow-amber-500/10',
        glow: 'ring-2 ring-amber-500/30',
    },
    executing: {
        bg: 'bg-gradient-to-r from-green-500/15 to-emerald-500/15 backdrop-blur-xl',
        border: 'border-green-500/50',
        shadow: 'shadow-xl shadow-green-500/10',
        glow: 'ring-2 ring-green-500/30',
    },
    model_loading: {
        bg: 'bg-gradient-to-r from-violet-500/15 to-purple-500/15 backdrop-blur-xl',
        border: 'border-violet-500/50',
        shadow: 'shadow-xl shadow-violet-500/10',
        glow: 'ring-2 ring-violet-500/30',
    },
};

// Animation variants - smoother, faster transitions
const containerVariants = {
    initial: { opacity: 0, scale: 0.98 },
    animate: { opacity: 1, scale: 1 },
    exit: { opacity: 0, scale: 0.98 },
};

const pulseKeyframes = {
    opacity: [0.5, 1, 0.5],
    scale: [1, 1.02, 1],
};

export default memo(function SmartInputBox({
    value,
    onChange,
    onSend,
    state,
    mode,
    pendingToolName,
    thinkingText = '',
    modelLoadProgress = 0,
    isModelUnloading = false,
    onApprove,
    onDeny,
    disabled = false,
    className,
}: SmartInputBoxProps) {
    const inputRef = useRef<HTMLTextAreaElement>(null);
    const config = STATE_CONFIG[state];

    // Get tool info
    const toolInfo = pendingToolName
        ? (TOOL_ICONS[pendingToolName.toLowerCase()] || TOOL_ICONS.default)
        : TOOL_ICONS.default;
    const ToolIconComponent = toolInfo.icon;

    // Focus input when returning to idle/typing
    useEffect(() => {
        if (state === 'idle' || state === 'typing') {
            inputRef.current?.focus();
        }
    }, [state]);

    // Handle key events
    const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            if (state === 'awaiting_approval' && onApprove) {
                onApprove();
            } else if ((state === 'idle' || state === 'typing') && value.trim()) {
                onSend();
            }
        }
        if (e.key === 'Escape' && state === 'awaiting_approval' && onDeny) {
            onDeny();
        }
    }, [state, value, onSend, onApprove, onDeny]);

    // Render different content based on state
    const renderContent = () => {
        switch (state) {
            case 'thinking':
                return (
                    <motion.div
                        key="thinking"
                        variants={containerVariants}
                        initial="initial"
                        animate="animate"
                        exit="exit"
                        className="p-4 min-h-[60px]"
                    >
                        {/* Show thinking text if available, otherwise shimmer */}
                        {thinkingText ? (
                            <motion.div
                                className="text-sm text-muted-foreground/70 italic leading-relaxed"
                                initial={{ opacity: 0 }}
                                animate={{ opacity: 1 }}
                            >
                                {thinkingText.split('\n').map((line, i) => (
                                    <motion.p
                                        key={i}
                                        initial={{ opacity: 0, x: -5 }}
                                        animate={{ opacity: 1, x: 0 }}
                                        transition={{ delay: i * 0.05 }}
                                        className="mb-0.5"
                                    >
                                        {line}
                                    </motion.p>
                                ))}
                            </motion.div>
                        ) : (
                            <div className="flex items-center justify-center">
                                <motion.span
                                    className="text-lg font-medium bg-gradient-to-r from-blue-400 via-cyan-300 to-blue-400 bg-[length:200%_100%] text-transparent bg-clip-text"
                                    animate={{
                                        backgroundPosition: ['200% 0', '-200% 0'],
                                    }}
                                    transition={{
                                        duration: 2,
                                        repeat: Infinity,
                                        ease: 'linear',
                                    }}
                                >
                                    Thinking...
                                </motion.span>
                            </div>
                        )}
                    </motion.div>
                );

            case 'model_loading':
                return (
                    <motion.div
                        key="model-loading"
                        variants={containerVariants}
                        initial="initial"
                        animate="animate"
                        exit="exit"
                        className="p-4 min-h-[60px]"
                    >
                        <div className="flex flex-col gap-3">
                            {/* Loading/Unloading Label */}
                            <div className="flex items-center justify-between">
                                <motion.span
                                    className="text-sm font-medium bg-gradient-to-r from-violet-400 via-purple-300 to-violet-400 bg-[length:200%_100%] text-transparent bg-clip-text"
                                    animate={{
                                        backgroundPosition: ['200% 0', '-200% 0'],
                                    }}
                                    transition={{
                                        duration: 1.5,
                                        repeat: Infinity,
                                        ease: 'linear',
                                    }}
                                >
                                    {isModelUnloading ? '⏳ Unloading Model...' : '🚀 Loading Model...'}
                                </motion.span>
                                <span className="text-xs text-muted-foreground font-mono">
                                    {modelLoadProgress}%
                                </span>
                            </div>

                            {/* Progress Bar Container */}
                            <div className="relative h-2 bg-secondary/50 rounded-full overflow-hidden">
                                {/* Progress Fill */}
                                <motion.div
                                    className="absolute top-0 left-0 h-full bg-gradient-to-r from-violet-500 via-purple-400 to-violet-500 rounded-full"
                                    style={{
                                        width: `${modelLoadProgress}%`,
                                    }}
                                    initial={{ width: 0 }}
                                    animate={{ width: `${modelLoadProgress}%` }}
                                    transition={{ duration: 0.3, ease: 'easeOut' }}
                                />

                                {/* Shimmer Overlay */}
                                <motion.div
                                    className="absolute top-0 left-0 h-full w-1/4 bg-gradient-to-r from-transparent via-white/30 to-transparent rounded-full"
                                    animate={{
                                        x: ['-100%', '500%'],
                                    }}
                                    transition={{
                                        duration: 1.2,
                                        repeat: Infinity,
                                        ease: 'easeInOut',
                                    }}
                                />
                            </div>

                            {/* Status Text */}
                            <p className="text-xs text-muted-foreground/70 text-center">
                                {isModelUnloading
                                    ? 'Releasing model from memory...'
                                    : modelLoadProgress < 50
                                        ? 'Initializing llama.cpp backend...'
                                        : 'Loading GGUF model into memory...'
                                }
                            </p>
                        </div>
                    </motion.div>
                );

            case 'awaiting_approval':
                return (
                    <motion.div
                        key="approval"
                        variants={containerVariants}
                        initial="initial"
                        animate="animate"
                        exit="exit"
                        className="p-4"
                    >
                        <div className="flex items-center justify-between mb-4">
                            <div className="flex items-center gap-3">
                                <motion.div
                                    className="w-12 h-12 rounded-xl bg-amber-500/20 flex items-center justify-center"
                                    animate={{ scale: [1, 1.05, 1] }}
                                    transition={{ duration: 2, repeat: Infinity }}
                                >
                                    <Shield className="w-6 h-6 text-amber-400" />
                                </motion.div>
                                <div>
                                    <div className="text-sm font-semibold text-foreground flex items-center gap-2">
                                        Approve tool execution?
                                        <Badge variant="outline" className="text-[10px] border-amber-500/50 text-amber-400">
                                            Requires Approval
                                        </Badge>
                                    </div>
                                    <div className="flex items-center gap-2 mt-1">
                                        <ToolIconComponent className={cn("w-4 h-4", toolInfo.color)} />
                                        <span className="text-xs font-mono text-amber-400">{pendingToolName || 'Unknown Tool'}</span>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div className="flex items-center gap-3">
                            <Button
                                onClick={onApprove}
                                className="flex-1 bg-green-600 hover:bg-green-700 text-white gap-2 h-11 font-semibold"
                            >
                                <Check className="w-4 h-4" />
                                Approve
                                <kbd className="ml-2 text-[10px] bg-green-700/50 px-2 py-0.5 rounded">↵</kbd>
                            </Button>
                            <Button
                                onClick={onDeny}
                                variant="outline"
                                className="flex-1 border-red-500/50 text-red-400 hover:bg-red-500/10 gap-2 h-11 font-semibold"
                            >
                                <X className="w-4 h-4" />
                                Deny
                                <kbd className="ml-2 text-[10px] bg-red-500/20 px-2 py-0.5 rounded">Esc</kbd>
                            </Button>
                        </div>
                    </motion.div>
                );

            case 'executing':
                return (
                    <motion.div
                        key="executing"
                        variants={containerVariants}
                        initial="initial"
                        animate="animate"
                        exit="exit"
                        className="flex items-center justify-center gap-4 h-[80px]"
                    >
                        <motion.div
                            className="w-12 h-12 rounded-xl bg-green-500/20 flex items-center justify-center"
                            animate={pulseKeyframes}
                            transition={{ duration: 2, repeat: Infinity, ease: 'easeInOut' }}
                        >
                            <ToolIconComponent className={cn("w-6 h-6", toolInfo.color)} />
                        </motion.div>
                        <div className="flex flex-col">
                            <div className="flex items-center gap-2">
                                <span className="text-sm font-semibold text-green-300">Executing</span>
                                <Loader2 className="w-4 h-4 animate-spin text-green-400" />
                            </div>
                            <span className="text-xs font-mono text-green-400/80">{pendingToolName || 'Tool'}</span>
                        </div>
                        <motion.div
                            className="absolute right-4 top-1/2 -translate-y-1/2"
                            animate={{ opacity: [0.5, 1, 0.5] }}
                            transition={{ duration: 1, repeat: Infinity }}
                        >
                            <Play className="w-5 h-5 text-green-400 fill-green-400" />
                        </motion.div>
                    </motion.div>
                );

            default: // idle or typing
                return (
                    <motion.div
                        key="input"
                        variants={containerVariants}
                        initial="initial"
                        animate="animate"
                        exit="exit"
                        className="relative"
                    >
                        <textarea
                            ref={inputRef}
                            value={value}
                            onChange={(e) => onChange(e.target.value)}
                            onKeyDown={handleKeyDown}
                            placeholder={mode === 'turbo'
                                ? 'Ask me to code something...'
                                : 'Ask a question...'}
                            disabled={disabled}
                            className="w-full bg-transparent border-0 p-4 pr-28 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none resize-none h-[80px] disabled:opacity-50 disabled:cursor-not-allowed"
                        />

                        {/* Action buttons */}
                        <div className="absolute bottom-4 right-4 flex items-center gap-3">
                            {value.length > 0 && (
                                <motion.span
                                    initial={{ opacity: 0, scale: 0.8 }}
                                    animate={{ opacity: 1, scale: 1 }}
                                    className="text-[10px] text-muted-foreground bg-secondary/50 px-2 py-0.5 rounded"
                                >
                                    {value.length}
                                </motion.span>
                            )}

                            <Tooltip>
                                <TooltipTrigger asChild>
                                    <Button
                                        onClick={onSend}
                                        disabled={disabled || !value.trim()}
                                        size="sm"
                                        className={cn(
                                            "rounded-xl h-10 w-10 p-0 transition-all",
                                            value.trim()
                                                ? "bg-primary hover:bg-primary/90 shadow-lg shadow-primary/25"
                                                : "bg-secondary/50"
                                        )}
                                    >
                                        <Send className="w-4 h-4" />
                                    </Button>
                                </TooltipTrigger>
                                <TooltipContent>
                                    <p>Send (Enter)</p>
                                </TooltipContent>
                            </Tooltip>
                        </div>
                    </motion.div>
                );
        }
    };

    return (
        <motion.div
            className={cn(
                "relative rounded-2xl border transition-all duration-300",
                config.bg,
                config.border,
                config.shadow,
                config.glow,
                className
            )}
            layout
            transition={{ type: 'spring', stiffness: 500, damping: 40 }}
        >
            <AnimatePresence mode="popLayout">
                {renderContent()}
            </AnimatePresence>

            {/* Footer hints - only show for idle/typing */}
            {(state === 'idle' || state === 'typing') && (
                <motion.div
                    className="flex items-center justify-between px-4 pb-3 text-[10px] text-muted-foreground"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    transition={{ delay: 0.1 }}
                >
                    <div className="flex items-center gap-2">
                        {mode === 'turbo' && (
                            <Badge variant="secondary" className="text-[9px] h-4 gap-1 bg-secondary/50">
                                <Sparkles className="w-2.5 h-2.5 text-primary" />
                                Can execute tools
                            </Badge>
                        )}
                    </div>
                    <div className="flex items-center gap-1">
                        <Keyboard className="w-3 h-3" />
                        <span>Enter to send</span>
                    </div>
                </motion.div>
            )}
        </motion.div>
    );
});

// Export tool icons for use elsewhere
export { TOOL_ICONS };
