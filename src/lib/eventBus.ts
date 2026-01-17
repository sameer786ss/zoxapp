//! Event Bus Abstraction
//! 
//! Typed event handling for Tauri IPC communication.

import { listen, emit, UnlistenFn } from '@tauri-apps/api/event';

// --- Event Payload Types ---

export interface AgentStreamChunk {
    text: string;
}

export interface AgentApprovalRequest {
    tool: string;
    parameters: string;
}

export interface AgentToolResult {
    tool: string;
    parameters: unknown;
    result: string;
}

export interface AgentFileAccess {
    action: 'read' | 'write';
    path: string;
}

export interface DownloadProgress {
    step: 'binaries' | 'model';
    percent: number;
    speed_mbps: number;
    eta_seconds: number;
    state: 'downloading' | 'paused' | 'resuming' | 'completed' | 'error';
}

export interface UpdateProgress {
    downloaded: number;
    total: number;
    percent: number;
    speed: number;
}

export interface UpdateError {
    code: 'network' | 'signature' | 'install' | 'unknown';
    message: string;
    retryable: boolean;
}

// --- Event Bus ---

export const EventBus = {
    // Agent Events
    onStreamChunk: (callback: (chunk: string) => void): Promise<UnlistenFn> =>
        listen<string>('agent-stream-chunk', (e) => callback(e.payload)),

    onThinking: (callback: (text: string) => void): Promise<UnlistenFn> =>
        listen<string>('agent-thinking', (e) => callback(e.payload)),

    onStreaming: (callback: (isStreaming: boolean) => void): Promise<UnlistenFn> =>
        listen<boolean>('agent-streaming', (e) => callback(e.payload)),

    onStatus: (callback: (status: string) => void): Promise<UnlistenFn> =>
        listen<string>('agent-status', (e) => callback(e.payload)),

    onApprovalRequest: (callback: (req: AgentApprovalRequest) => void): Promise<UnlistenFn> =>
        listen<AgentApprovalRequest>('agent-approval-request', (e) => callback(e.payload)),

    onToolResult: (callback: (result: AgentToolResult) => void): Promise<UnlistenFn> =>
        listen<AgentToolResult>('agent-tool-result', (e) => callback(e.payload)),

    onStreamEnd: (callback: (reason: string) => void): Promise<UnlistenFn> =>
        listen<string>('agent-stream-end', (e) => callback(e.payload)),

    onError: (callback: (error: string) => void): Promise<UnlistenFn> =>
        listen<string>('agent-error', (e) => callback(e.payload)),

    onFileAccess: (callback: (access: AgentFileAccess) => void): Promise<UnlistenFn> =>
        listen<AgentFileAccess>('agent-file-access', (e) => callback(e.payload)),

    // Model Events
    onModelLoadProgress: (callback: (progress: number) => void): Promise<UnlistenFn> =>
        listen<number>('model-load-progress', (e) => callback(e.payload)),

    onModelLoadComplete: (callback: (status: string) => void): Promise<UnlistenFn> =>
        listen<string>('model-load-complete', (e) => callback(e.payload)),

    onActiveModelChanged: (callback: (model: string) => void): Promise<UnlistenFn> =>
        listen<string>('active-model-changed', (e) => callback(e.payload)),

    // Download Events
    onDownloadProgress: (callback: (progress: DownloadProgress) => void): Promise<UnlistenFn> =>
        listen<DownloadProgress>('download-progress', (e) => callback(e.payload)),

    onSetupComplete: (callback: () => void): Promise<UnlistenFn> =>
        listen('setup-complete', () => callback()),

    // Update Events
    onUpdateProgress: (callback: (progress: UpdateProgress) => void): Promise<UnlistenFn> =>
        listen<UpdateProgress>('update-download-progress', (e) => callback(e.payload)),

    onUpdateError: (callback: (error: UpdateError) => void): Promise<UnlistenFn> =>
        listen<UpdateError>('update-error', (e) => callback(e.payload)),

    // App Events
    onAppReady: (callback: () => void): Promise<UnlistenFn> =>
        listen('app-ready', () => callback()),

    onConnectionModeChanged: (callback: (mode: string) => void): Promise<UnlistenFn> =>
        listen<string>('connection-mode-changed', (e) => callback(e.payload)),

    // Emit helpers
    emitFrontendLoaded: (): Promise<void> =>
        emit('frontend_loaded', true),
};

export default EventBus;
