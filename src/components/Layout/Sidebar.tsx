import { useState, useMemo } from 'react';
import {
    Menu, X, MessageSquare, Zap, Sparkles,
    Clock, Trash2, PlusCircle, Settings
} from 'lucide-react';
import { useAgentStore } from '@/stores/useAgentStore';
import { toast } from '@/components/ui/sonner';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent } from '@/components/ui/card';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';
import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
} from '@/components/ui/tooltip';
import { cn } from '@/lib/utils';

export default function Sidebar() {
    const [isOpen, setIsOpen] = useState(false); // Default closed on launch
    const { mode, setMode, messages, clearHistory } = useAgentStore();

    // Generate conversation summary from messages
    const conversationSummary = useMemo(() => {
        if (messages.length === 0) return null;

        const firstUserMessage = messages.find(m => m.role === 'user');
        const preview = firstUserMessage?.content.slice(0, 50) || 'New conversation';
        const lastMessage = messages[messages.length - 1];

        return {
            preview: preview + (firstUserMessage && firstUserMessage.content.length > 50 ? '...' : ''),
            messageCount: messages.length,
            timestamp: lastMessage?.timestamp || Date.now(),
        };
    }, [messages]);

    const formatTime = (timestamp: number) => {
        const date = new Date(timestamp);
        return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    };

    return (
        <TooltipProvider>
            {/* Toggle Button (visible when closed) */}
            {!isOpen && (
                <Tooltip>
                    <TooltipTrigger asChild>
                        <Button
                            variant="outline"
                            size="icon"
                            className="fixed top-12 left-4 z-50 bg-card/80 backdrop-blur-sm border-border/50 hover:bg-secondary transition-all duration-fast ease-fluent"
                            onClick={() => setIsOpen(true)}
                        >
                            <Menu className="w-5 h-5" />
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent side="right">
                        <p>Open sidebar</p>
                    </TooltipContent>
                </Tooltip>
            )}

            {/* Sidebar */}
            <div className={cn(
                "fixed left-0 top-8 h-[calc(100%-8rem-28px)] z-40 transition-all duration-normal ease-fluent flex flex-col w-64",
                "bg-card/90 backdrop-blur-md border-r border-border/30",
                isOpen
                    ? "translate-x-0 opacity-100"
                    : "-translate-x-full opacity-0 pointer-events-none"
            )}>
                {/* Header */}
                <div className="p-4 border-b border-border/50 flex items-center justify-between shrink-0">
                    <div className="flex items-center gap-2">
                        <div className="w-6 h-6 rounded-md bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center">
                            <Sparkles className="w-3.5 h-3.5 text-white" />
                        </div>
                        <span className="font-semibold text-foreground text-sm">Agent IDE</span>
                    </div>
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <Button
                                variant="ghost"
                                size="icon"
                                className="h-7 w-7 hover:bg-secondary/80 transition-all duration-fast ease-fluent"
                                onClick={() => setIsOpen(false)}
                            >
                                <X className="w-4 h-4" />
                            </Button>
                        </TooltipTrigger>
                        <TooltipContent>
                            <p>Close sidebar</p>
                        </TooltipContent>
                    </Tooltip>
                </div>

                {/* New Chat Button */}
                <div className="p-3 border-b border-border/50 shrink-0">
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <Button
                                variant="outline"
                                className="w-full justify-start gap-2 bg-secondary/50 hover:bg-secondary border-border/50 transition-all duration-fast ease-fluent"
                                onClick={() => {
                                    clearHistory();
                                    toast.success('Started new chat');
                                }}
                            >
                                <PlusCircle className="w-4 h-4" />
                                New Chat
                            </Button>
                        </TooltipTrigger>
                        <TooltipContent side="right">
                            <p>Start a new conversation</p>
                        </TooltipContent>
                    </Tooltip>
                </div>

                {/* Mode Toggle */}
                <div className="p-3 border-b border-border/50 shrink-0">
                    <div className="flex gap-2">
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <Button
                                    variant={mode === 'chat' ? 'default' : 'outline'}
                                    size="sm"
                                    className={cn(
                                        "flex-1 gap-1 transition-all duration-fast ease-fluent",
                                        mode === 'chat' && "bg-primary hover:bg-primary/90"
                                    )}
                                    onClick={() => {
                                        setMode('chat');
                                        toast.info('Switched to Chat Mode');
                                    }}
                                >
                                    <MessageSquare className="w-3 h-3" />
                                    Chat
                                </Button>
                            </TooltipTrigger>
                            <TooltipContent>
                                <p>Chat only mode</p>
                            </TooltipContent>
                        </Tooltip>
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <Button
                                    variant={mode === 'turbo' ? 'default' : 'outline'}
                                    size="sm"
                                    className={cn(
                                        "flex-1 gap-1 transition-all duration-fast ease-fluent",
                                        mode === 'turbo' && "bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-500 hover:to-purple-500 border-0"
                                    )}
                                    onClick={() => {
                                        setMode('turbo');
                                        toast.info('Switched to Turbo Mode');
                                    }}
                                >
                                    <Zap className="w-3 h-3" />
                                    Turbo
                                </Button>
                            </TooltipTrigger>
                            <TooltipContent>
                                <p>Agent can execute tools</p>
                            </TooltipContent>
                        </Tooltip>
                    </div>
                </div>

                {/* History with ScrollArea */}
                <ScrollArea className="flex-1">
                    <div className="p-3">
                        <span className="text-xs font-medium text-muted-foreground uppercase tracking-wider px-2">
                            History
                        </span>

                        <div className="mt-2 space-y-1">
                            {conversationSummary ? (
                                <Card className="bg-secondary/50 border-border/50 hover:bg-secondary/70 transition-all duration-fast ease-fluent cursor-pointer">
                                    <CardContent className="p-3">
                                        <div className="flex items-start gap-2">
                                            <MessageSquare className="w-4 h-4 text-muted-foreground mt-0.5 shrink-0" />
                                            <div className="flex-1 min-w-0">
                                                <p className="text-sm text-foreground truncate">
                                                    {conversationSummary.preview}
                                                </p>
                                                <div className="flex items-center gap-2 mt-1">
                                                    <Badge variant="secondary" className="text-[9px] h-4 bg-muted/50">
                                                        <Clock className="w-2.5 h-2.5 mr-1" />
                                                        {formatTime(conversationSummary.timestamp)}
                                                    </Badge>
                                                    <Badge variant="outline" className="text-[9px] h-4 border-border/50">
                                                        {conversationSummary.messageCount} msgs
                                                    </Badge>
                                                </div>
                                            </div>
                                        </div>
                                    </CardContent>
                                </Card>
                            ) : (
                                <div className="px-2 py-8 text-center">
                                    <MessageSquare className="w-8 h-8 text-muted-foreground/30 mx-auto mb-2" />
                                    <p className="text-sm text-muted-foreground">No conversations yet</p>
                                    <p className="text-xs text-muted-foreground/70 mt-1">Start chatting to see history</p>
                                </div>
                            )}
                        </div>
                    </div>
                </ScrollArea>

                {/* Footer */}
                <div className="p-3 border-t border-border/50 shrink-0 space-y-1">
                    {messages.length > 0 && (
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <Button
                                    variant="ghost"
                                    size="sm"
                                    className="w-full justify-start gap-2 text-destructive hover:text-destructive hover:bg-destructive/10 transition-all duration-fast ease-fluent"
                                    onClick={() => {
                                        clearHistory();
                                        toast.info('History cleared');
                                    }}
                                >
                                    <Trash2 className="w-4 h-4" />
                                    Clear History
                                </Button>
                            </TooltipTrigger>
                            <TooltipContent side="right">
                                <p>Delete all messages</p>
                            </TooltipContent>
                        </Tooltip>
                    )}
                    <Separator className="my-2 bg-border/50" />
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <Button
                                variant="ghost"
                                size="sm"
                                className="w-full justify-start gap-2 hover:bg-secondary/80 transition-all duration-fast ease-fluent"
                                onClick={() => toast.info('Settings coming soon!')}
                            >
                                <Settings className="w-4 h-4" />
                                Settings
                            </Button>
                        </TooltipTrigger>
                        <TooltipContent side="right">
                            <p>Open settings</p>
                        </TooltipContent>
                    </Tooltip>
                </div>
            </div>
        </TooltipProvider>
    );
}
