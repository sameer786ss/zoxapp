//! Command Abstraction
//! 
//! Typed command wrappers for Tauri invoke calls.

import { invoke } from '@tauri-apps/api/core';

// --- Response Types ---

export interface SetupStatus {
    binaries_ok: boolean;
    model_ok: boolean;
}

export interface GpuDetectionResult {
    gpu_type: 'nvidia' | 'amd' | 'intel' | 'cpu';
    name: string;
    vram_mb?: number;
}

export interface ConversationMeta {
    id: string;
    title: string;
    created_at: string;
    updated_at: string;
    message_count: number;
    mode: string;
}

export interface UpdateInfo {
    version: string;
    currentVersion: string;
    releaseNotes?: string;
    releaseDate?: string;
    downloadSize?: number;
}

// --- Command Wrappers ---

export const Commands = {
    // Agent Commands
    startAgentTask: (task: string, isTurbo: boolean): Promise<void> =>
        invoke('start_agent_task', { task, is_turbo: isTurbo }),

    cancelAgentTask: (): Promise<void> =>
        invoke('cancel_agent_task'),

    sendUserFeedback: (approved: boolean): Promise<void> =>
        invoke('send_user_feedback', { approved }),

    // Workspace Commands
    readWorkspaceFile: (path: string): Promise<string> =>
        invoke('read_workspace_file', { path }),

    saveWorkspaceFile: (path: string, content: string): Promise<void> =>
        invoke('save_workspace_file', { path, content }),

    // Conversation Commands
    listConversations: (): Promise<ConversationMeta[]> =>
        invoke('list_conversations'),

    loadConversation: (id: string): Promise<void> =>
        invoke('load_conversation', { id }),

    deleteConversation: (id: string): Promise<void> =>
        invoke('delete_conversation', { id }),

    // Setup Commands
    detectGpu: (): Promise<GpuDetectionResult> =>
        invoke('detect_gpu_cmd'),

    checkSetupStatus: (): Promise<SetupStatus> =>
        invoke('check_setup_status'),

    downloadBinaries: (gpuType: string): Promise<void> =>
        invoke('download_binaries', { gpu_type: gpuType }),

    downloadModel: (): Promise<void> =>
        invoke('download_model'),

    setConnectionMode: (mode: 'cloud' | 'offline'): Promise<void> =>
        invoke('set_connection_mode', { mode }),

    pauseDownload: (): Promise<void> =>
        invoke('pause_download'),

    resumeDownload: (): Promise<void> =>
        invoke('resume_download'),

    cancelDownload: (): Promise<void> =>
        invoke('cancel_download'),

    // Update Commands
    checkForUpdates: (): Promise<UpdateInfo | null> =>
        invoke('check_for_updates'),

    downloadUpdate: (): Promise<void> =>
        invoke('download_update'),

    installUpdate: (): Promise<void> =>
        invoke('install_update'),

    getAppVersion: (): Promise<string> =>
        invoke('get_app_version'),
};

export default Commands;
