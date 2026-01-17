//! Connection State Slice
//! 
//! Connection mode and model loading state.

import { StateCreator } from 'zustand';

export type ConnectionMode = 'cloud' | 'offline';
export type SetupStatus = 'complete' | 'downloading_binaries' | 'downloading_model' | 'needs_setup' | null;
export type DownloadState = 'downloading' | 'paused' | 'resuming' | 'completed' | 'error';

export interface GpuInfo {
    type: 'nvidia' | 'amd' | 'intel' | 'cpu';
    name: string;
    vram_mb?: number;
}

export interface DownloadProgress {
    step: 'binaries' | 'model';
    percent: number;
    speed_mbps: number;
    eta_seconds: number;
    state: DownloadState;
}

export interface ConnectionSlice {
    connectionMode: ConnectionMode;
    isModelLoaded: boolean;
    modelLoadProgress: number | null;
    setupStatus: SetupStatus;
    downloadProgress: DownloadProgress | null;
    detectedGpu: GpuInfo | null;

    setConnectionMode: (mode: ConnectionMode) => void;
    setModelLoaded: (loaded: boolean) => void;
    setModelLoadProgress: (progress: number | null) => void;
    setSetupStatus: (status: SetupStatus) => void;
    setDownloadProgress: (progress: DownloadProgress | null) => void;
    setDetectedGpu: (gpu: GpuInfo | null) => void;
}

export const createConnectionSlice: StateCreator<ConnectionSlice> = (set) => ({
    connectionMode: 'cloud',
    isModelLoaded: false,
    modelLoadProgress: null,
    setupStatus: null,
    downloadProgress: null,
    detectedGpu: null,

    setConnectionMode: (connectionMode) => set({ connectionMode }),
    setModelLoaded: (isModelLoaded) => set({ isModelLoaded }),
    setModelLoadProgress: (modelLoadProgress) => set({ modelLoadProgress }),
    setSetupStatus: (setupStatus) => set({ setupStatus }),
    setDownloadProgress: (downloadProgress) => set({ downloadProgress }),
    setDetectedGpu: (detectedGpu) => set({ detectedGpu }),
});
