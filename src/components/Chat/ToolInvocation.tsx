import { Loader2, CheckCircle, XCircle, Terminal, FileText, Search } from 'lucide-react';
import { cn } from '../../lib/utils';

interface ToolInvocationProps {
    toolName: string;
    args: any;
    status: 'running' | 'success' | 'error' | 'awaiting_approval';
    output?: string;
}

export default function ToolInvocation({ toolName, args, status, output }: ToolInvocationProps) {
    const icons = {
        read_file: FileText,
        write_file: FileText,
        run_terminal: Terminal,
        search_project: Search
    };

    const Icon = icons[toolName as keyof typeof icons] || Terminal;

    return (
        <div className="my-2 ml-4 max-w-[90%] font-mono text-sm">
            {/* Tool Header */}
            <div className={cn(
                "flex items-center gap-2 px-3 py-2 rounded-t-md border-t border-x border-white/10 bg-white/5",
                status === 'running' ? "animate-pulse" : ""
            )}>
                <Icon className="w-4 h-4 text-blue-400" />
                <span className="font-semibold text-zinc-300">{toolName}</span>
                <span className="text-zinc-500 truncate max-w-[200px]">{JSON.stringify(args)}</span>

                <div className="ml-auto flex items-center gap-2">
                    {status === 'running' && <Loader2 className="w-3 h-3 animate-spin text-zinc-400" />}
                    {status === 'success' && <CheckCircle className="w-3 h-3 text-green-500" />}
                    {status === 'error' && <XCircle className="w-3 h-3 text-red-500" />}
                </div>
            </div>

            {/* Tool Output / Body */}
            <div className="bg-black/40 border border-white/10 rounded-b-md p-3 overflow-x-auto">
                {output ? (
                    <pre className="text-zinc-400 text-xs">{output}</pre>
                ) : (
                    <span className="text-zinc-600 italic">Processing...</span>
                )}
            </div>
        </div>
    );
}
