import { useState, useEffect, useCallback } from 'react';
import {
    MessageSquare, Zap, FileText, Terminal, Search,
    Trash2, PlusCircle, Keyboard, HelpCircle
} from 'lucide-react';
import { useAgentStore } from '@/stores/useAgentStore';
import { toast } from '@/components/ui/sonner';
import {
    CommandDialog,
    CommandEmpty,
    CommandGroup,
    CommandInput,
    CommandItem,
    CommandList,
    CommandShortcut,
} from '@/components/ui/command';

export function CommandPalette() {
    const [open, setOpen] = useState(false);
    const { setMode, clearHistory } = useAgentStore();

    // Keyboard shortcut to open
    useEffect(() => {
        const down = (e: KeyboardEvent) => {
            if (e.key === 'k' && (e.metaKey || e.ctrlKey)) {
                e.preventDefault();
                setOpen((open) => !open);
            }
        };
        document.addEventListener('keydown', down);
        return () => document.removeEventListener('keydown', down);
    }, []);

    const runCommand = useCallback((command: () => void) => {
        command();
        setOpen(false);
    }, []);

    return (
        <CommandDialog open={open} onOpenChange={setOpen}>
            <CommandInput placeholder="Type a command or search..." />
            <CommandList>
                <CommandEmpty>No results found.</CommandEmpty>

                {/* Mode Commands */}
                <CommandGroup heading="Mode">
                    <CommandItem
                        onSelect={() => runCommand(() => {
                            setMode('chat');
                            toast.info('Switched to Chat Mode');
                        })}
                    >
                        <MessageSquare className="mr-2 h-4 w-4" />
                        <span>Switch to Chat Mode</span>
                        <CommandShortcut>⌘1</CommandShortcut>
                    </CommandItem>
                    <CommandItem
                        onSelect={() => runCommand(() => {
                            setMode('turbo');
                            toast.info('Switched to Turbo Mode', { description: 'Agent can now execute tools' });
                        })}
                    >
                        <Zap className="mr-2 h-4 w-4 text-purple-400" />
                        <span>Switch to Turbo Mode</span>
                        <CommandShortcut>⌘2</CommandShortcut>
                    </CommandItem>
                </CommandGroup>

                {/* Actions */}
                <CommandGroup heading="Actions">
                    <CommandItem
                        onSelect={() => runCommand(() => {
                            clearHistory();
                            toast.success('Started new chat');
                        })}
                    >
                        <PlusCircle className="mr-2 h-4 w-4" />
                        <span>New Chat</span>
                        <CommandShortcut>⌘N</CommandShortcut>
                    </CommandItem>
                    <CommandItem
                        onSelect={() => runCommand(() => {
                            clearHistory();
                            toast.info('History cleared');
                        })}
                    >
                        <Trash2 className="mr-2 h-4 w-4" />
                        <span>Clear History</span>
                    </CommandItem>
                </CommandGroup>

                {/* Tools */}
                <CommandGroup heading="Tools">
                    <CommandItem
                        onSelect={() => runCommand(() => {
                            toast.info('Type a file path to read in chat');
                        })}
                    >
                        <FileText className="mr-2 h-4 w-4" />
                        <span>Read File</span>
                    </CommandItem>
                    <CommandItem
                        onSelect={() => runCommand(() => {
                            toast.info('Type a command to run in chat');
                        })}
                    >
                        <Terminal className="mr-2 h-4 w-4" />
                        <span>Run Command</span>
                    </CommandItem>
                    <CommandItem
                        onSelect={() => runCommand(() => {
                            toast.info('Type what to search for in chat');
                        })}
                    >
                        <Search className="mr-2 h-4 w-4" />
                        <span>Search Project</span>
                    </CommandItem>
                </CommandGroup>

                {/* Help */}
                <CommandGroup heading="Help">
                    <CommandItem
                        onSelect={() => runCommand(() => {
                            toast.info('Keyboard Shortcuts', {
                                description: '⌘K: Command Palette\n⌘Enter: Send\n⌘1/2: Switch Mode'
                            });
                        })}
                    >
                        <Keyboard className="mr-2 h-4 w-4" />
                        <span>Keyboard Shortcuts</span>
                        <CommandShortcut>⌘/</CommandShortcut>
                    </CommandItem>
                    <CommandItem
                        onSelect={() => runCommand(() => {
                            toast.info('Documentation coming soon!');
                        })}
                    >
                        <HelpCircle className="mr-2 h-4 w-4" />
                        <span>Help & Documentation</span>
                    </CommandItem>
                </CommandGroup>
            </CommandList>
        </CommandDialog>
    );
}

export default CommandPalette;
