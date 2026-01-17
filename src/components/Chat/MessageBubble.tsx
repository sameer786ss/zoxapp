import { motion } from 'framer-motion';
import {
    Bot,
    User,
    XCircle,
    Clock,
    Copy,
    Check
} from 'lucide-react';
import { cn } from '@/lib/utils';
import { Message } from '@/stores/useAgentStore';
import { MarkdownRenderer } from '@/components/Common/MarkdownRenderer';
import { useState, useCallback, memo } from 'react';
import { toast } from '@/components/ui/sonner';
import { Badge } from '@/components/ui/badge';

interface MessageBubbleProps {
    message: Message;
}

export default memo(function MessageBubble({ message }: MessageBubbleProps) {
    const isUser = message.role === 'user';
    const isTool = message.role === 'tool';
    const [copied, setCopied] = useState(false);

    // Format timestamp
    const formatTime = (timestamp: number) => {
        const date = new Date(timestamp);
        return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    };

    // Copy message content
    const handleCopy = useCallback(() => {
        navigator.clipboard.writeText(message.content);
        setCopied(true);
        toast.success('Copied to clipboard');
        setTimeout(() => setCopied(false), 2000);
    }, [message.content]);

    // Tool message rendering with enhanced display
    if (isTool) {
        const contentStr = typeof message.content === 'string' ? message.content : String(message.content || '');

        // Parse tool name from content - supports both [toolName] and **toolName** formats
        let toolName = 'Tool';
        let toolContent = contentStr;

        // Try markdown format first: **toolName**
        const markdownMatch = contentStr.match(/^\*\*(\w+)\*\*/);
        if (markdownMatch) {
            toolName = markdownMatch[1];
            toolContent = contentStr.slice(markdownMatch[0].length).trim();
        } else {
            // Fall back to bracket format: [toolName]
            const bracketMatch = contentStr.match(/^\[(\w+)\]/);
            if (bracketMatch) {
                toolName = bracketMatch[1];
                toolContent = contentStr.slice(bracketMatch[0].length).trim();
            }
        }

        // Determine status from content
        const hasError = toolContent.toLowerCase().includes('error') || toolContent.toLowerCase().includes('failed');

        return (
            <motion.div
                initial={{ opacity: 0, y: 10, scale: 0.98 }}
                animate={{ opacity: 1, y: 0, scale: 1 }}
                transition={{ type: 'spring', stiffness: 300, damping: 30 }}
                style={{ willChange: 'transform, opacity' }}
                className={cn(
                    "flex gap-3 p-4 rounded-2xl my-2 mx-4 group relative overflow-hidden",
                    "max-w-[85%] bg-secondary/30 border border-border/30"
                )}
            >
                {/* Avatar */}
                <motion.div
                    className="mt-1 w-8 h-8 rounded-full flex items-center justify-center shrink-0 bg-blue-500/20 text-blue-400"
                    initial={{ scale: 0.8 }}
                    animate={{ scale: 1 }}
                    transition={{ type: 'spring', stiffness: 400 }}
                >
                    <Bot className="w-4 h-4" />
                </motion.div>

                {/* Content */}
                <div className="flex-1 min-w-0 overflow-hidden">
                    {/* Header */}
                    <div className="flex items-center gap-2 mb-1">
                        <span className="font-semibold text-xs text-muted-foreground">
                            Agent
                        </span>
                        <span className="flex items-center gap-1 text-[10px] text-muted-foreground">
                            <Clock className="w-2.5 h-2.5" />
                            {formatTime(message.timestamp)}
                        </span>
                    </div>

                    {/* Tool Info - Minimal */}
                    <div className="flex items-center gap-2">
                        <span className="font-mono text-sm text-blue-400 font-medium">{toolName}</span>
                        <Badge
                            variant="outline"
                            className={cn(
                                "text-[10px] h-5 gap-1 ml-auto",
                                hasError ? "border-red-500/50 text-red-400" : "border-green-500/50 text-green-400"
                            )}
                        >
                            {hasError ? (
                                <>
                                    <XCircle className="w-3 h-3 mr-1" />
                                    Failed
                                </>
                            ) : (
                                <>
                                    <Check className="w-3 h-3 mr-1" />
                                    Completed
                                </>
                            )}
                        </Badge>
                    </div>
                </div>
            </motion.div>
        );
    }

    // User/Model message rendering
    return (
        <motion.div
            initial={{ opacity: 0, y: 10, scale: 0.98 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            transition={{ type: 'spring', stiffness: 300, damping: 30 }}
            style={{ willChange: 'transform, opacity' }}
            className={cn(
                "flex gap-3 p-4 rounded-2xl my-2 mx-4 group relative overflow-hidden",
                isUser
                    ? "ml-auto max-w-[75%] bg-primary text-primary-foreground"
                    : "max-w-[85%] bg-secondary/30 border border-border/30"
            )}
        >
            {/* Avatar */}
            <motion.div
                className={cn(
                    "mt-1 w-8 h-8 rounded-full flex items-center justify-center shrink-0",
                    isUser ? "bg-primary-foreground/20" : "bg-blue-500/20 text-blue-400"
                )}
                initial={{ scale: 0.8 }}
                animate={{ scale: 1 }}
                transition={{ type: 'spring', stiffness: 400 }}
            >
                {isUser ? <User className="w-4 h-4" /> : <Bot className="w-4 h-4" />}
            </motion.div>

            {/* Content */}
            <div className="flex-1 min-w-0 overflow-hidden">
                {/* Header */}
                <div className="flex items-center gap-2 mb-1">
                    <span className={cn(
                        "font-semibold text-xs",
                        isUser ? "text-primary-foreground/70" : "text-muted-foreground"
                    )}>
                        {isUser ? 'You' : 'Agent'}
                    </span>
                    <span className={cn(
                        "flex items-center gap-1 text-[10px]",
                        isUser ? "text-primary-foreground/50" : "text-muted-foreground"
                    )}>
                        <Clock className="w-2.5 h-2.5" />
                        {formatTime(message.timestamp)}
                    </span>

                    {/* Copy button - visible on hover */}
                    <motion.button
                        onClick={handleCopy}
                        className={cn(
                            "ml-auto opacity-0 group-hover:opacity-100 transition-opacity p-1.5 rounded-lg",
                            isUser ? "hover:bg-primary-foreground/20" : "hover:bg-accent"
                        )}
                        title="Copy message"
                        whileHover={{ scale: 1.1 }}
                        whileTap={{ scale: 0.9 }}
                    >
                        {copied ? (
                            <Check className={cn("w-3 h-3", isUser ? "text-primary-foreground" : "text-green-500")} />
                        ) : (
                            <Copy className={cn("w-3 h-3", isUser ? "text-primary-foreground/70" : "text-muted-foreground")} />
                        )}
                    </motion.button>
                </div>

                {/* Message Content */}
                <div className="text-sm overflow-hidden break-words">
                    {isUser ? (
                        <div className="whitespace-pre-wrap break-words leading-relaxed">
                            {message.content}
                        </div>
                    ) : (
                        <MarkdownRenderer
                            content={message.content}
                            className="text-foreground"
                        />
                    )}
                </div>
            </div>
        </motion.div>
    );
});
