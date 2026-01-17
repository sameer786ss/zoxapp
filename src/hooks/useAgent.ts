import { useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { useAgentStore } from '@/stores/useAgentStore';

interface FileAccessEvent {
    action: 'read' | 'write';
    path: string;
}

interface ApprovalRequest {
    tool: string;
    parameters: string;
}

// Streaming timeout in milliseconds (2 minutes)
const STREAMING_TIMEOUT_MS = 120000;

export function useAgent() {
    const {
        mode,
        addMessage,
        updateStreamingMessage,
        setStatus,
        setStreaming,
        setPendingTool,
        setThinkingText,
        openFile,
    } = useAgentStore();

    // Use ref for stable access in listeners
    const storeRef = useRef({ addMessage, updateStreamingMessage, setStatus, setStreaming, setThinkingText });
    // Store resolved unlisten functions to avoid cleanup race condition
    const unlistenRef = useRef<UnlistenFn[]>([]);
    // Timeout ref for streaming timeout
    const streamingTimeoutRef = useRef<number | null>(null);

    useEffect(() => {
        storeRef.current = { addMessage, updateStreamingMessage, setStatus, setStreaming, setThinkingText };
    }, [addMessage, updateStreamingMessage, setStatus, setStreaming, setThinkingText]);

    // Setup event listeners
    useEffect(() => {
        let isMounted = true;
        const listeners: Promise<UnlistenFn>[] = [];

        // Function to reset/start streaming timeout
        const resetStreamingTimeout = () => {
            if (streamingTimeoutRef.current) {
                clearTimeout(streamingTimeoutRef.current);
            }
            streamingTimeoutRef.current = window.setTimeout(() => {
                console.warn('[useAgent] Streaming timeout - resetting status');
                if (isMounted) { // Only update state if component is still mounted
                    storeRef.current.setStatus('idle');
                    storeRef.current.setStreaming(false);
                }
            }, STREAMING_TIMEOUT_MS);
        };

        // Function to clear timeout
        const clearStreamingTimeout = () => {
            if (streamingTimeoutRef.current) {
                clearTimeout(streamingTimeoutRef.current);
                streamingTimeoutRef.current = null;
            }
        };

        // Listen for thinking content (displayed in input box)
        listeners.push(listen<string>('agent-thinking', (event) => {
            if (!isMounted) return;
            const { setThinkingText } = storeRef.current;
            setThinkingText(event.payload);
            resetStreamingTimeout();
        }));

        // Listen for streaming chunks
        listeners.push(listen<string>('agent-stream-chunk', (event) => {
            if (!isMounted) return;
            const { updateStreamingMessage, setStreaming } = storeRef.current;
            updateStreamingMessage(event.payload);
            setStreaming(true);
            resetStreamingTimeout();
        }));

        // Listen for Streaming Status
        listeners.push(listen<boolean>('agent-streaming', (event) => {
            if (!isMounted) return;
            const { setStreaming, setStatus } = storeRef.current;
            setStreaming(event.payload);
            if (event.payload) {
                setStatus('thinking');
            }
        }));

        // Listen for Status Updates
        listeners.push(listen<string>('agent-status', (event) => {
            if (!isMounted) return;
            const status = event.payload.toLowerCase();
            const { setStatus, setStreaming } = storeRef.current;

            if (status.includes('idle') || status.includes('ready') || status.includes('complete')) {
                setStatus('idle');
                setStreaming(false);
            } else if (status.includes('thinking')) {
                setStatus('thinking');
            } else if (status.includes('executing')) {
                setStatus('executing');
            } else if (status.includes('cancel') || status.includes('denied')) {
                setStatus('idle');
                setStreaming(false);
            }
        }));

        // Listen for Tool Results
        interface ToolResultPayload {
            tool: string;
            parameters: unknown;
            result: string;
        }
        listeners.push(listen<ToolResultPayload>('agent-tool-result', (event) => {
            if (!isMounted) return;
            const { addMessage, setStreaming } = storeRef.current;
            const { tool, result } = event.payload;
            const content = `**${tool}**\n\`\`\`\n${result}\n\`\`\``;
            addMessage({ role: 'tool', content });
            setStreaming(false);
        }));

        // Listen for Approval Requests
        listeners.push(listen<ApprovalRequest>('agent-approval-request', (event) => {
            if (!isMounted) return;
            console.log('[useAgent] Approval request:', event.payload);
            setPendingTool({
                name: event.payload.tool,
                args: event.payload.parameters
            });
        }));

        // Listen for File Access events
        listeners.push(listen<FileAccessEvent>('agent-file-access', (event) => {
            if (!isMounted) return;
            const { action, path } = event.payload;
            console.log(`[FileAccess] ${action}: ${path}`);
            if (path && (action === 'read' || action === 'write')) {
                openFile(path, 'agent');
            }
        }));

        // Listen for Stream End
        listeners.push(listen<string>('agent-stream-end', (event) => {
            if (!isMounted) return;
            const { setStatus, setStreaming } = storeRef.current;
            console.log('[useAgent] Stream ended:', event.payload);
            setStatus('idle');
            setStreaming(false);
            clearStreamingTimeout();
        }));

        // Listen for Errors
        listeners.push(listen<string>('agent-error', (event) => {
            if (!isMounted) return;
            console.error('[Agent Error]', event.payload);
            storeRef.current.setStatus('idle');
            storeRef.current.setStreaming(false);
            clearStreamingTimeout();
        }));

        // Robust cleanup: only set if mounted, cleanup immediately if unmounted
        Promise.all(listeners).then((unlistenFns) => {
            if (isMounted) {
                unlistenRef.current = unlistenFns;
            } else {
                unlistenFns.forEach(fn => fn());
            }
        });

        // Cleanup on unmount
        return () => {
            isMounted = false;
            unlistenRef.current.forEach((unlisten) => unlisten());
            unlistenRef.current = [];
            clearStreamingTimeout();
        };
    }, [openFile, setPendingTool]);

    // Start agent task
    const startAgent = useCallback(async (prompt: string) => {
        const currentMode = useAgentStore.getState().mode;
        setStatus('thinking');
        setStreaming(true);

        // Note: User message is added by ChatPanel before calling startAgent

        try {
            await invoke('start_agent_task', {
                task: prompt,
                is_turbo: currentMode === 'turbo'
            });
        } catch (err) {
            console.error('Failed to start agent:', err);
            setStatus('error');
            setStreaming(false);
        }
    }, [setStatus, setStreaming]);

    // Cancel agent task
    const cancelAgent = useCallback(async () => {
        try {
            await invoke('cancel_agent_task');
            setStatus('idle');
            setStreaming(false);
            setPendingTool(null);
        } catch (err) {
            console.error('Failed to cancel:', err);
        }
    }, [setStatus, setStreaming, setPendingTool]);

    return {
        startAgent,
        cancelAgent,
        mode
    };
}
