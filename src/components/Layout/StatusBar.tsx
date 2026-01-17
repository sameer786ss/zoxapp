import { useState } from 'react';
import { useAgentStore } from '@/stores/useAgentStore';
import { useUpdater } from '@/hooks/useUpdater';
import {
    Activity,
    MessageSquare,
    Zap,
    Download,
    RefreshCw,
    CheckCircle2,
    AlertCircle,
    Loader2,
    RotateCcw
} from 'lucide-react';
import { cn } from '@/lib/utils';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
} from '@/components/ui/tooltip';
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
} from '@/components/ui/popover';

export default function StatusBar() {
    const { status, mode, messages } = useAgentStore();
    const {
        status: updateStatus,
        updateInfo,
        progress,
        error,
        checkForUpdates,
        downloadUpdate,
        installUpdate,
        dismissUpdate,
        retryCount
    } = useUpdater();

    const [showUpdatePopover, setShowUpdatePopover] = useState(false);

    // Count messages
    const messageCount = messages.length;

    // Status indicator config
    const statusConfig: Record<string, { variant: 'default' | 'secondary' | 'destructive' | 'outline'; label: string; pulse: boolean }> = {
        idle: { variant: 'secondary', label: 'Ready', pulse: false },
        thinking: { variant: 'default', label: 'Thinking', pulse: true },
        executing: { variant: 'default', label: 'Executing', pulse: true },
        awaiting_approval: { variant: 'outline', label: 'Awaiting Approval', pulse: true },
        error: { variant: 'destructive', label: 'Error', pulse: false },
    };

    const currentStatus = statusConfig[status] || statusConfig.idle;

    // Update status rendering
    const renderUpdateIndicator = () => {
        switch (updateStatus) {
            case 'checking':
                return (
                    <div className="flex items-center gap-1.5 text-muted-foreground">
                        <Loader2 className="w-3 h-3 animate-spin" />
                        <span>Checking...</span>
                    </div>
                );

            case 'available':
                return (
                    <Popover open={showUpdatePopover} onOpenChange={setShowUpdatePopover}>
                        <PopoverTrigger asChild>
                            <button className="flex items-center gap-1.5 text-blue-400 hover:text-blue-300 transition-colors">
                                <Download className="w-3 h-3" />
                                <span>Update v{updateInfo?.version}</span>
                            </button>
                        </PopoverTrigger>
                        <PopoverContent className="w-72" align="end">
                            <div className="space-y-3">
                                <div className="flex items-center gap-2">
                                    <Download className="w-4 h-4 text-blue-400" />
                                    <div>
                                        <p className="font-medium text-sm">Update Available</p>
                                        <p className="text-xs text-muted-foreground">
                                            v{updateInfo?.currentVersion} → v{updateInfo?.version}
                                        </p>
                                    </div>
                                </div>
                                {updateInfo?.releaseNotes && (
                                    <p className="text-xs text-muted-foreground line-clamp-3">
                                        {updateInfo.releaseNotes}
                                    </p>
                                )}
                                <div className="flex gap-2">
                                    <Button size="sm" className="flex-1" onClick={() => {
                                        downloadUpdate();
                                        setShowUpdatePopover(false);
                                    }}>
                                        Download
                                    </Button>
                                    <Button size="sm" variant="ghost" onClick={() => {
                                        dismissUpdate();
                                        setShowUpdatePopover(false);
                                    }}>
                                        Later
                                    </Button>
                                </div>
                            </div>
                        </PopoverContent>
                    </Popover>
                );

            case 'downloading':
                return (
                    <div className="flex items-center gap-2 min-w-32">
                        <Loader2 className="w-3 h-3 animate-spin text-blue-400" />
                        <div className="flex-1 space-y-1">
                            <div className="flex justify-between text-[10px]">
                                <span>Downloading</span>
                                <span>{progress?.percent.toFixed(0)}%</span>
                            </div>
                            <Progress value={progress?.percent || 0} className="h-1" />
                        </div>
                    </div>
                );

            case 'ready':
                return (
                    <button
                        onClick={installUpdate}
                        className="flex items-center gap-1.5 text-green-400 hover:text-green-300 transition-colors animate-pulse"
                    >
                        <RefreshCw className="w-3 h-3" />
                        <span>Restart to Update</span>
                    </button>
                );

            case 'error':
                return (
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <button
                                onClick={checkForUpdates}
                                className="flex items-center gap-1.5 text-destructive hover:text-destructive/80 transition-colors"
                            >
                                <AlertCircle className="w-3 h-3" />
                                <span>Update failed</span>
                                {retryCount > 0 && (
                                    <span className="text-[10px]">({retryCount}/3)</span>
                                )}
                            </button>
                        </TooltipTrigger>
                        <TooltipContent>
                            <p>{error?.message}</p>
                            {error?.retryable && <p className="text-xs">Click to retry</p>}
                        </TooltipContent>
                    </Tooltip>
                );

            case 'up-to-date':
                return (
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <div className="flex items-center gap-1.5 text-green-400/70 cursor-default">
                                <CheckCircle2 className="w-3 h-3" />
                                <span>Up to date</span>
                            </div>
                        </TooltipTrigger>
                        <TooltipContent>
                            <p>You're running the latest version</p>
                            <button
                                onClick={checkForUpdates}
                                className="text-xs text-blue-400 hover:underline flex items-center gap-1 mt-1"
                            >
                                <RotateCcw className="w-3 h-3" />
                                Check again
                            </button>
                        </TooltipContent>
                    </Tooltip>
                );

            default:
                return (
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <button
                                onClick={checkForUpdates}
                                className="text-muted-foreground/50 hover:text-muted-foreground transition-colors"
                            >
                                v0.1.0
                            </button>
                        </TooltipTrigger>
                        <TooltipContent>
                            <p>Click to check for updates</p>
                        </TooltipContent>
                    </Tooltip>
                );
        }
    };

    return (
        <TooltipProvider>
            <div className="h-7 bg-card/60 backdrop-blur-sm border-t border-border/50 flex items-center justify-between px-3 text-[11px] text-muted-foreground shrink-0">
                {/* Left side */}
                <div className="flex items-center gap-3">
                    {/* Status indicator */}
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <div className="flex items-center gap-1.5">
                                <Badge variant={currentStatus.variant} className="h-5 text-[10px] gap-1">
                                    {currentStatus.pulse && (
                                        <span className="w-1.5 h-1.5 rounded-full bg-current animate-pulse" />
                                    )}
                                    {currentStatus.label}
                                </Badge>
                            </div>
                        </TooltipTrigger>
                        <TooltipContent>
                            <p>Agent status: {status}</p>
                        </TooltipContent>
                    </Tooltip>

                    <Separator orientation="vertical" className="h-4 bg-border/50" />

                    {/* Mode indicator */}
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <div className="flex items-center gap-1.5 cursor-default">
                                {mode === 'turbo' ? (
                                    <Zap className="w-3 h-3 text-blue-400" />
                                ) : (
                                    <MessageSquare className="w-3 h-3" />
                                )}
                                <span className={cn(
                                    "capitalize transition-colors duration-fast ease-fluent",
                                    mode === 'turbo' && "text-blue-400"
                                )}>
                                    {mode} Mode
                                </span>
                            </div>
                        </TooltipTrigger>
                        <TooltipContent>
                            <p>{mode === 'turbo' ? 'Agent can execute tools' : 'Chat only mode'}</p>
                        </TooltipContent>
                    </Tooltip>
                </div>

                {/* Right side */}
                <div className="flex items-center gap-3">
                    {/* Message count */}
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <div className="flex items-center gap-1.5 cursor-default">
                                <Activity className="w-3 h-3" />
                                <span>{messageCount} messages</span>
                            </div>
                        </TooltipTrigger>
                        <TooltipContent>
                            <p>Total messages in conversation</p>
                        </TooltipContent>
                    </Tooltip>

                    <Separator orientation="vertical" className="h-4 bg-border/50" />

                    {/* Update indicator */}
                    {renderUpdateIndicator()}
                </div>
            </div>
        </TooltipProvider>
    );
}
