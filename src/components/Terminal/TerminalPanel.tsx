import { useEffect, useRef, useState, useCallback } from 'react';
import { Terminal as XTerm } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { Maximize2, Minimize2, X, Terminal as TerminalIcon } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import '@xterm/xterm/css/xterm.css';

interface TerminalPanelProps {
    onClose?: () => void;
    onMinimize?: () => void;
    isMinimized?: boolean;
}

export default function TerminalPanel({ onClose, onMinimize, isMinimized = false }: TerminalPanelProps) {
    const terminalRef = useRef<HTMLDivElement>(null);
    const xtermRef = useRef<XTerm | null>(null);
    const fitAddonRef = useRef<FitAddon | null>(null);
    const [isReady, setIsReady] = useState(false);

    useEffect(() => {
        if (!terminalRef.current || isMinimized) return;

        // Initialize xterm
        const xterm = new XTerm({
            theme: {
                background: 'hsl(240 5% 4%)',
                foreground: 'hsl(0 0% 98%)',
                cursor: 'hsl(0 0% 98%)',
                cursorAccent: 'hsl(240 5% 4%)',
                selectionBackground: 'hsla(221 83% 53% / 0.4)',
                black: 'hsl(240 4% 8%)',
                red: '#ef4444',
                green: '#22c55e',
                yellow: '#eab308',
                blue: '#3b82f6',
                magenta: '#a855f7',
                cyan: '#06b6d4',
                white: 'hsl(0 0% 98%)',
                brightBlack: 'hsl(240 4% 46%)',
                brightRed: '#f87171',
                brightGreen: '#4ade80',
                brightYellow: '#facc15',
                brightBlue: '#60a5fa',
                brightMagenta: '#c084fc',
                brightCyan: '#22d3ee',
                brightWhite: '#ffffff',
            },
            fontFamily: "'JetBrains Mono', 'Fira Code', Consolas, monospace",
            fontSize: 13,
            lineHeight: 1.4,
            cursorBlink: true,
            cursorStyle: 'bar',
            scrollback: 5000,
            allowProposedApi: true,
        });

        const fitAddon = new FitAddon();
        xterm.loadAddon(fitAddon);
        xterm.open(terminalRef.current);

        // Fit after a small delay to ensure container is sized
        setTimeout(() => {
            fitAddon.fit();
            setIsReady(true);
        }, 100);

        xtermRef.current = xterm;
        fitAddonRef.current = fitAddon;

        // Welcome message
        xterm.writeln('\x1b[1;34m╭──────────────────────────────────────────╮\x1b[0m');
        xterm.writeln('\x1b[1;34m│\x1b[0m  \x1b[1;36mAgent IDE Terminal\x1b[0m                      \x1b[1;34m│\x1b[0m');
        xterm.writeln('\x1b[1;34m│\x1b[0m  Tool outputs will appear here           \x1b[1;34m│\x1b[0m');
        xterm.writeln('\x1b[1;34m╰──────────────────────────────────────────╯\x1b[0m');
        xterm.writeln('');

        // Handle resize
        const handleResize = () => {
            if (fitAddonRef.current) {
                fitAddonRef.current.fit();
            }
        };
        window.addEventListener('resize', handleResize);

        return () => {
            window.removeEventListener('resize', handleResize);
            xterm.dispose();
        };
    }, [isMinimized]);

    // Refit when container changes
    useEffect(() => {
        if (fitAddonRef.current && !isMinimized) {
            setTimeout(() => fitAddonRef.current?.fit(), 50);
        }
    }, [isMinimized]);

    if (isMinimized) {
        return (
            <div
                onClick={onMinimize}
                className="h-8 bg-card border-t border-border flex items-center px-3 gap-2 cursor-pointer hover:bg-accent transition-colors"
            >
                <TerminalIcon className="w-4 h-4 text-muted-foreground" />
                <span className="text-xs text-muted-foreground">Terminal</span>
                <Maximize2 className="w-3 h-3 text-muted-foreground ml-auto" />
            </div>
        );
    }

    return (
        <div className="flex flex-col h-full bg-background border-t border-border">
            {/* Header */}
            <div className="flex items-center justify-between h-8 px-3 bg-card border-b border-border shrink-0">
                <div className="flex items-center gap-2">
                    <TerminalIcon className="w-4 h-4 text-muted-foreground" />
                    <span className="text-xs text-foreground">Terminal</span>
                    {isReady && (
                        <span className="w-1.5 h-1.5 bg-green-500 rounded-full animate-pulse" />
                    )}
                </div>
                <div className="flex items-center gap-1">
                    <Button
                        variant="ghost"
                        size="icon"
                        className="h-6 w-6"
                        onClick={onMinimize}
                        title="Minimize"
                    >
                        <Minimize2 className="w-3.5 h-3.5" />
                    </Button>
                    {onClose && (
                        <Button
                            variant="ghost"
                            size="icon"
                            className="h-6 w-6 hover:bg-destructive/10 hover:text-destructive"
                            onClick={onClose}
                            title="Close"
                        >
                            <X className="w-3.5 h-3.5" />
                        </Button>
                    )}
                </div>
            </div>

            {/* Terminal content */}
            <div
                ref={terminalRef}
                className="flex-1 p-2 overflow-hidden"
                style={{ minHeight: '150px' }}
            />
        </div>
    );
}

// Hook to write to terminal from anywhere
export function useTerminal() {
    const write = useCallback((text: string) => {
        window.dispatchEvent(new CustomEvent('terminal-write', { detail: text }));
    }, []);

    const writeLine = useCallback((text: string) => {
        window.dispatchEvent(new CustomEvent('terminal-write', { detail: text + '\r\n' }));
    }, []);

    const clear = useCallback(() => {
        window.dispatchEvent(new CustomEvent('terminal-clear'));
    }, []);

    return { write, writeLine, clear };
}

// Terminal output component for displaying command results inline
interface TerminalOutputProps {
    content: string;
    exitCode?: number;
    className?: string;
}

export function TerminalOutput({ content, exitCode, className = '' }: TerminalOutputProps) {
    const [expanded, setExpanded] = useState(true);
    const lines = content.split('\n');
    const isLong = lines.length > 20;

    return (
        <div className={cn("bg-background border border-border rounded-md overflow-hidden", className)}>
            {/* Header */}
            <div className="flex items-center justify-between px-3 py-1.5 bg-muted border-b border-border">
                <div className="flex items-center gap-2">
                    <TerminalIcon className="w-3.5 h-3.5 text-muted-foreground" />
                    <span className="text-[10px] text-muted-foreground font-mono uppercase">Output</span>
                </div>
                <div className="flex items-center gap-2">
                    {exitCode !== undefined && (
                        <span className={cn("text-[10px] font-mono", exitCode === 0 ? 'text-green-500' : 'text-destructive')}>
                            exit: {exitCode}
                        </span>
                    )}
                    {isLong && (
                        <button
                            onClick={() => setExpanded(!expanded)}
                            className="text-[10px] text-primary hover:text-primary/80"
                        >
                            {expanded ? 'Collapse' : 'Expand'}
                        </button>
                    )}
                </div>
            </div>

            {/* Content */}
            <pre className={cn("p-3 text-xs font-mono text-foreground overflow-x-auto", !expanded && isLong && 'max-h-[200px] overflow-y-hidden')}>
                {expanded ? content : lines.slice(0, 10).join('\n') + '\n...'}
            </pre>
        </div>
    );
}
