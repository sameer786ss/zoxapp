import { useAgentStore } from '@/stores/useAgentStore';
import { invoke } from '@tauri-apps/api/core';
import { AlertTriangle, Terminal, FileText, Search, Info } from 'lucide-react';
import { toast } from '@/components/ui/sonner';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog';
import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
} from '@/components/ui/tooltip';

export default function ApprovalModal() {
    const { pendingTool, setPendingTool } = useAgentStore();

    if (!pendingTool) return null;

    // Parse the tool arguments
    let parsedArgs: Record<string, unknown> = {};
    try {
        parsedArgs = JSON.parse(pendingTool.args);
    } catch {
        parsedArgs = { raw: pendingTool.args };
    }

    // Determine risk level
    const getRiskLevel = (toolName: string): 'high' | 'medium' | 'low' => {
        if (toolName === 'run_terminal') return 'high';
        if (toolName === 'write_file') return 'medium';
        return 'low';
    };

    const riskLevel = getRiskLevel(pendingTool.name);

    // Get tool icon
    const getToolIcon = (name: string) => {
        switch (name) {
            case 'run_terminal': return <Terminal className="h-5 w-5 text-red-400" />;
            case 'write_file': return <FileText className="h-5 w-5 text-orange-400" />;
            case 'read_file': return <FileText className="h-5 w-5 text-blue-400" />;
            case 'search_project': return <Search className="h-5 w-5 text-purple-400" />;
            default: return <Terminal className="h-5 w-5 text-muted-foreground" />;
        }
    };

    const handleApprove = async () => {
        try {
            await invoke('send_user_feedback', { approved: true });
            toast.success('Tool approved');
        } catch (err) {
            toast.error('Failed to send approval');
            console.error(err);
        }
        setPendingTool(null);
    };

    const handleDeny = async () => {
        try {
            await invoke('send_user_feedback', { approved: false });
            toast.info('Tool denied');
        } catch (err) {
            toast.error('Failed to send denial');
            console.error(err);
        }
        setPendingTool(null);
    };

    return (
        <TooltipProvider>
            <Dialog open={!!pendingTool} onOpenChange={(open) => !open && handleDeny()}>
                <DialogContent className="sm:max-w-md bg-background border-border">
                    <DialogHeader>
                        <DialogTitle className="flex items-center gap-3">
                            {getToolIcon(pendingTool.name)}
                            <span>Tool Approval Required</span>
                        </DialogTitle>
                        <DialogDescription className="flex items-center gap-2">
                            The agent wants to execute
                            <Badge variant="outline" className="font-mono">
                                {pendingTool.name}
                            </Badge>
                        </DialogDescription>
                    </DialogHeader>

                    {/* Risk Badge */}
                    <div className="flex items-center gap-2">
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <Badge
                                    variant={riskLevel === 'high' ? 'destructive' : riskLevel === 'medium' ? 'secondary' : 'outline'}
                                    className="gap-1"
                                >
                                    {riskLevel === 'high' && <AlertTriangle className="w-3 h-3" />}
                                    {riskLevel.toUpperCase()} RISK
                                </Badge>
                            </TooltipTrigger>
                            <TooltipContent>
                                <p>
                                    {riskLevel === 'high' && 'This tool can modify system state'}
                                    {riskLevel === 'medium' && 'This tool can modify files'}
                                    {riskLevel === 'low' && 'This tool is read-only'}
                                </p>
                            </TooltipContent>
                        </Tooltip>
                    </div>

                    <Separator />

                    {/* Arguments Display */}
                    <Card>
                        <CardHeader className="py-2 px-3">
                            <CardTitle className="text-xs font-medium text-muted-foreground uppercase tracking-wider flex items-center gap-1">
                                <Info className="w-3 h-3" />
                                Arguments
                            </CardTitle>
                        </CardHeader>
                        <CardContent className="px-3 pb-3 pt-0">
                            <ScrollArea className="max-h-[150px]">
                                {Object.entries(parsedArgs).map(([key, value]) => (
                                    <div key={key} className="flex flex-col gap-0.5 mb-2 last:mb-0">
                                        <Badge variant="secondary" className="w-fit text-[10px] h-4">
                                            {key}
                                        </Badge>
                                        <code className="text-sm text-foreground font-mono bg-muted px-2 py-1 rounded break-all">
                                            {typeof value === 'string' ? value : JSON.stringify(value)}
                                        </code>
                                    </div>
                                ))}
                            </ScrollArea>
                        </CardContent>
                    </Card>

                    {/* Warning for dangerous tools */}
                    {riskLevel === 'high' && (
                        <Card className="border-destructive/50 bg-destructive/10">
                            <CardContent className="p-3 flex items-start gap-3">
                                <AlertTriangle className="h-5 w-5 text-destructive shrink-0 mt-0.5" />
                                <div>
                                    <p className="text-sm font-medium text-destructive">Warning: Shell Command</p>
                                    <p className="text-xs text-destructive/70 mt-1">
                                        This will execute a command on your system. Make sure you trust this action.
                                    </p>
                                </div>
                            </CardContent>
                        </Card>
                    )}

                    <DialogFooter className="gap-2 sm:gap-0">
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <Button variant="outline" onClick={handleDeny}>
                                    Deny
                                </Button>
                            </TooltipTrigger>
                            <TooltipContent>
                                <p>Cancel this tool execution</p>
                            </TooltipContent>
                        </Tooltip>
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <Button
                                    variant={riskLevel === 'high' ? 'destructive' : 'default'}
                                    onClick={handleApprove}
                                >
                                    Approve
                                </Button>
                            </TooltipTrigger>
                            <TooltipContent>
                                <p>Allow this tool to run</p>
                            </TooltipContent>
                        </Tooltip>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </TooltipProvider>
    );
}
