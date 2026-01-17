import { useState, useRef, useEffect, useCallback } from 'react';
import { Sparkles, Command as CommandIcon, X, Plus, History } from 'lucide-react';
import { useAgentStore } from '@/stores/useAgentStore';
import { useAgent } from '@/hooks/useAgent';
import VirtualizedMessageList from './VirtualizedMessageList';
import SmartInputBox from './SmartInputBox';
import ConnectionToggle from '@/components/Common/ConnectionToggle';
import StreamingProgress from '@/components/Common/StreamingProgress';
import { toast } from '@/components/ui/sonner';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';
import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
} from '@/components/ui/tooltip';
import { AnimatePresence, motion } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export default function ChatPanel() {
    const [input, setInput] = useState('');
    const { messages, addMessage, status, mode, pendingTool, setPendingTool, setStatus, thinkingText, conversations, clearHistory } = useAgentStore();
    const { startAgent, cancelAgent } = useAgent();
    const inputRef = useRef<HTMLTextAreaElement>(null);
    const [showHistory, setShowHistory] = useState(false);
    const [modelLoadProgress, setModelLoadProgress] = useState(0);
    const [isModelLoading, setIsModelLoading] = useState(false);
    const [isModelUnloading, setIsModelUnloading] = useState(false);

    // Focus input on mount and mode change
    useEffect(() => {
        inputRef.current?.focus();
    }, [mode]);

    // Ref for previous progress to persist across renders
    const previousProgressRef = useRef(0);
    // Refs for unlisten functions to ensure cleanup
    const unlistenProgressRef = useRef<(() => void) | null>(null);
    const unlistenCompleteRef = useRef<(() => void) | null>(null);

    // Listen for model load progress events
    useEffect(() => {
        // Setup listeners
        listen<number>('model-load-progress', (event) => {
            const progress = event.payload;
            const previousProgress = previousProgressRef.current;

            // Detect loading vs unloading
            if (progress > previousProgress) {
                setIsModelUnloading(false);
            } else if (progress < previousProgress && previousProgress > 50) {
                setIsModelUnloading(true);
            }

            previousProgressRef.current = progress;
            setModelLoadProgress(progress);

            if (progress > 0 && progress < 100) {
                setIsModelLoading(true);
            }
        }).then(fn => { unlistenProgressRef.current = fn; });

        listen('model-load-complete', () => {
            setIsModelLoading(false);
            setModelLoadProgress(0);
            previousProgressRef.current = 0;
        }).then(fn => { unlistenCompleteRef.current = fn; });

        return () => {
            // Cleanup with stored refs
            unlistenProgressRef.current?.();
            unlistenCompleteRef.current?.();
        };
    }, []);

    // Keyboard shortcuts
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                if (pendingTool) {
                    handleDenyTool();
                } else if (status !== 'idle') {
                    cancelAgent();
                }
            }
            if (e.key === 'Enter' && !e.shiftKey && pendingTool) {
                e.preventDefault();
                handleApproveTool();
            }
        };

        document.addEventListener('keydown', handleKeyDown);
        return () => document.removeEventListener('keydown', handleKeyDown);
    }, [pendingTool, status, cancelAgent]);

    const handleSend = useCallback(() => {
        if (!input.trim() || status !== 'idle') return;

        const messageToSend = input.trim();
        setInput('');
        addMessage({ role: 'user', content: messageToSend });
        startAgent(messageToSend);
    }, [input, status, addMessage, startAgent]);

    const handleApproveTool = useCallback(async () => {
        try {
            await invoke('send_user_feedback', { approved: true });
            setPendingTool(null);
            setStatus('executing');
        } catch (err) {
            console.error('Failed to approve:', err);
            toast.error('Failed to send approval');
        }
    }, [setPendingTool, setStatus]);

    const handleDenyTool = useCallback(async () => {
        try {
            await invoke('send_user_feedback', { approved: false });
            setPendingTool(null);
            setStatus('thinking');
        } catch (err) {
            console.error('Failed to deny:', err);
        }
    }, [setPendingTool, setStatus]);

    const handleNewChat = () => {
        clearHistory();
        toast.success('Started new conversation');
    };

    const isDisabled = status !== 'idle' && !pendingTool;
    const isAwaitingApproval = !!pendingTool;

    return (
        <TooltipProvider>
            <div className="h-full flex flex-col bg-card/50 overflow-hidden">
                {/* Header */}
                <div className="shrink-0 h-12 bg-card/60 border-b border-border/40 flex items-center justify-between px-4">
                    <div className="flex items-center gap-3">
                        <Badge variant="outline" className="text-[10px] h-5 border-border/50 text-muted-foreground flex items-center gap-1 font-semibold">
                            <img src="/zox-logo.png" alt="ZOX" className="w-4 h-4" />
                            ZOX
                        </Badge>

                        {/* Mode switcher */}
                        <Tabs value={mode} onValueChange={(v) => useAgentStore.getState().setMode(v as 'chat' | 'turbo')}>
                            <TabsList className="h-7 bg-secondary/50">
                                <TabsTrigger value="chat" className="text-[10px] h-5 px-2">Chat</TabsTrigger>
                                <TabsTrigger value="turbo" className="text-[10px] h-5 px-2 gap-1">
                                    <Sparkles className="w-3 h-3" />
                                    Turbo
                                </TabsTrigger>
                            </TabsList>
                        </Tabs>

                        {/* Status indicator */}
                        {status !== 'idle' && (
                            <Badge
                                variant={isAwaitingApproval ? "destructive" : "secondary"}
                                className="text-[10px] h-5 animate-pulse"
                            >
                                {isAwaitingApproval ? 'Approval' : status}
                            </Badge>
                        )}
                    </div>

                    <div className="flex items-center gap-1">
                        {/* Connection mode toggle */}
                        <ConnectionToggle />
                        {/* History button */}
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    className="h-7 w-7"
                                    onClick={() => setShowHistory(!showHistory)}
                                >
                                    <History className="w-4 h-4" />
                                </Button>
                            </TooltipTrigger>
                            <TooltipContent>
                                <p>Chat History</p>
                            </TooltipContent>
                        </Tooltip>

                        {/* New chat button */}
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    className="h-7 w-7"
                                    onClick={handleNewChat}
                                >
                                    <Plus className="w-4 h-4" />
                                </Button>
                            </TooltipTrigger>
                            <TooltipContent>
                                <p>New Chat</p>
                            </TooltipContent>
                        </Tooltip>

                        {/* Cancel button when active */}
                        {(status !== 'idle' || pendingTool) && (
                            <Tooltip>
                                <TooltipTrigger asChild>
                                    <Button
                                        variant="ghost"
                                        size="icon"
                                        className="h-7 w-7 text-destructive hover:text-destructive hover:bg-destructive/10"
                                        onClick={() => {
                                            if (pendingTool) handleDenyTool();
                                            else cancelAgent();
                                        }}
                                    >
                                        <X className="w-4 h-4" />
                                    </Button>
                                </TooltipTrigger>
                                <TooltipContent>
                                    <p>{pendingTool ? 'Deny (Esc)' : 'Cancel (Esc)'}</p>
                                </TooltipContent>
                            </Tooltip>
                        )}
                    </div>
                </div>

                {/* Main content area - Messages or History */}
                <div className="flex-1 flex overflow-hidden">
                    {/* History sidebar */}
                    <AnimatePresence>
                        {showHistory && (
                            <motion.div
                                initial={{ width: 0, opacity: 0 }}
                                animate={{ width: 200, opacity: 1 }}
                                exit={{ width: 0, opacity: 0 }}
                                className="border-r border-border/40 bg-secondary/20 overflow-hidden"
                            >
                                <HistorySidebar
                                    conversations={conversations}
                                    onClose={() => setShowHistory(false)}
                                />
                            </motion.div>
                        )}
                    </AnimatePresence>

                    {/* Messages */}
                    <ScrollArea className="flex-1">
                        {messages.length === 0 ? (
                            <EmptyState mode={mode} />
                        ) : (
                            <div className="pb-4">
                                <VirtualizedMessageList messages={messages} />
                            </div>
                        )}
                    </ScrollArea>
                </div>

                {/* Input Area - SmartInputBox handles all states */}
                <div className="p-4 border-t border-border/40 bg-card/40 relative">
                    {/* Streaming Progress Indicator */}
                    <AnimatePresence>
                        {status === 'executing' && (
                            <motion.div
                                initial={{ opacity: 0, y: 10 }}
                                animate={{ opacity: 1, y: 0 }}
                                exit={{ opacity: 0, y: 10 }}
                                className="absolute -top-12 left-0 right-0 z-10 flex justify-center pointer-events-none"
                            >
                                <StreamingProgress
                                    startTime={Date.now()} // Ideally track actual start time in store
                                    tokenCount={0} // We need to add token tracking to store
                                />
                            </motion.div>
                        )}
                    </AnimatePresence>

                    <SmartInputBox
                        value={input}
                        onChange={setInput}
                        onSend={handleSend}
                        state={getInputBoxState(status, !!pendingTool, isModelLoading)}
                        mode={mode}
                        pendingToolName={pendingTool?.name}
                        thinkingText={thinkingText}
                        modelLoadProgress={modelLoadProgress}
                        isModelUnloading={isModelUnloading}
                        onApprove={handleApproveTool}
                        onDeny={handleDenyTool}
                        disabled={isDisabled && !isAwaitingApproval}
                    />
                </div>
            </div>
        </TooltipProvider>
    );
}

// Map agent status to input box state
function getInputBoxState(
    status: string,
    hasPendingTool: boolean,
    isModelLoading: boolean
): 'idle' | 'typing' | 'thinking' | 'awaiting_approval' | 'executing' | 'model_loading' {
    if (isModelLoading) return 'model_loading';
    if (hasPendingTool) return 'awaiting_approval';
    if (status === 'executing') return 'executing';
    if (status === 'thinking') return 'thinking';
    return 'idle';
}


// History sidebar
interface HistorySidebarProps {
    conversations: Array<{ id: string; title: string; updatedAt: string }>;
    onClose: () => void;
}

function HistorySidebar({ conversations, onClose }: HistorySidebarProps) {
    return (
        <div className="h-full flex flex-col p-2">
            <div className="flex items-center justify-between mb-2">
                <span className="text-xs font-semibold text-muted-foreground">History</span>
                <Button variant="ghost" size="icon" className="h-5 w-5" onClick={onClose}>
                    <X className="w-3 h-3" />
                </Button>
            </div>
            <ScrollArea className="flex-1">
                {conversations.length === 0 ? (
                    <div className="text-xs text-muted-foreground text-center py-4">
                        No conversations yet
                    </div>
                ) : (
                    <div className="space-y-1">
                        {conversations.map((conv) => (
                            <button
                                key={conv.id}
                                className="w-full text-left p-2 rounded-lg hover:bg-secondary/50 transition-colors"
                                onClick={() => {
                                    // TODO: Load conversation
                                    console.log('Load:', conv.id);
                                }}
                            >
                                <div className="text-xs font-medium truncate">{conv.title}</div>
                                <div className="text-[10px] text-muted-foreground">
                                    {new Date(conv.updatedAt).toLocaleDateString()}
                                </div>
                            </button>
                        ))}
                    </div>
                )}
            </ScrollArea>
        </div>
    );
}

// Empty state component
function EmptyState({ mode }: { mode: string }) {
    return (
        <div className="h-full flex items-center justify-center p-8">
            <Card className="max-w-md w-full bg-secondary/30 border-border/40">
                <CardHeader className="text-center pb-2">
                    <div className="w-16 h-16 rounded-2xl bg-primary/10 flex items-center justify-center mx-auto mb-4">
                        {mode === 'turbo' ? (
                            <Sparkles className="w-8 h-8 text-primary" />
                        ) : (
                            <CommandIcon className="w-8 h-8 text-primary" />
                        )}
                    </div>
                    <CardTitle className="text-lg">
                        {mode === 'turbo' ? 'Turbo Mode' : 'Chat Mode'}
                    </CardTitle>
                    <CardDescription>
                        {mode === 'turbo'
                            ? 'I can read, write, and execute code'
                            : 'Ask me anything about coding'}
                    </CardDescription>
                </CardHeader>
                <CardContent>
                    <Separator className="my-3 bg-border/30" />
                    <div className="space-y-2 text-xs text-muted-foreground">
                        <div className="flex items-center gap-2">
                            <Badge variant="outline" className="text-[9px] h-4">Tip</Badge>
                            <span>{mode === 'turbo'
                                ? 'Tools require your approval'
                                : 'Switch to Turbo for file operations'}</span>
                        </div>
                    </div>
                </CardContent>
            </Card>
        </div>
    );
}

