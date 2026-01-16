import { useState } from 'react';
import { Wifi, WifiOff, Loader2, Download, AlertTriangle, Pause, Play, X } from 'lucide-react';
import { useAgentStore, ConnectionMode, DownloadProgress } from '@/stores/useAgentStore';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
} from '@/components/ui/tooltip';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog';
import { Progress } from '@/components/ui/progress';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { toast } from '@/components/ui/sonner';

interface SetupStatus {
    binaries_ok: boolean;
    model_ok: boolean;
}

interface GpuDetectionResult {
    gpu_type: 'nvidia' | 'amd' | 'intel' | 'cpu';
    name: string;
    vram_mb?: number;
}

export default function ConnectionToggle() {
    const {
        connectionMode,
        setConnectionMode,
        modelLoadProgress,
        setModelLoadProgress,
        setModelLoaded,
        setSetupStatus,
        downloadProgress,
        setDownloadProgress,
        setDetectedGpu,
        status,
    } = useAgentStore();

    const [showConfirmDialog, setShowConfirmDialog] = useState(false);
    const [showSetupDialog, setShowSetupDialog] = useState(false);
    const [pendingMode, setPendingMode] = useState<ConnectionMode | null>(null);
    const [isChecking, setIsChecking] = useState(false);
    const [detectedGpuLocal, setDetectedGpuLocal] = useState<GpuDetectionResult | null>(null);

    const isLoading = modelLoadProgress !== null;
    const isBusy = status !== 'idle';

    const handleToggleClick = async () => {
        if (isLoading || isBusy) return;

        const targetMode: ConnectionMode = connectionMode === 'cloud' ? 'offline' : 'cloud';
        setPendingMode(targetMode);

        if (targetMode === 'offline') {
            // Check if setup is complete before switching to offline
            setIsChecking(true);
            try {
                const setupResult = await invoke<SetupStatus>('check_setup_status');

                if (!setupResult.binaries_ok || !setupResult.model_ok) {
                    // Need to run setup
                    const gpuResult = await invoke<GpuDetectionResult>('detect_gpu_cmd');
                    setDetectedGpuLocal(gpuResult);
                    setDetectedGpu({
                        type: gpuResult.gpu_type,
                        name: gpuResult.name,
                        vram_mb: gpuResult.vram_mb,
                    });
                    setShowSetupDialog(true);
                } else {
                    // Setup already complete, just confirm switch
                    setShowConfirmDialog(true);
                }
            } catch (err) {
                console.error('Failed to check setup status:', err);
                toast.error('Failed to check offline mode requirements');
            } finally {
                setIsChecking(false);
            }
        } else {
            // Switching to cloud - just confirm
            setShowConfirmDialog(true);
        }
    };

    const handleConfirmSwitch = async () => {
        if (!pendingMode) return;
        setShowConfirmDialog(false);

        try {
            // Set up event listeners for progress
            const unlistenLoad = await listen<number>('model-load-progress', (event) => {
                setModelLoadProgress(event.payload);
            });

            const unlistenComplete = await listen<string>('model-load-complete', () => {
                setModelLoadProgress(null);
                setModelLoaded(pendingMode === 'offline');
                unlistenLoad();
            });

            // Start the switch
            setModelLoadProgress(0);
            await invoke('set_connection_mode', { mode: pendingMode });

            setConnectionMode(pendingMode);
            toast.success(pendingMode === 'offline' ? 'Switched to offline mode' : 'Switched to cloud mode');

            unlistenComplete();
        } catch (err) {
            console.error('Failed to switch mode:', err);
            setModelLoadProgress(null);
            toast.error(`Failed to switch to ${pendingMode} mode`);
        }
    };

    const handleStartSetup = async () => {
        if (!detectedGpuLocal) return;
        setShowSetupDialog(false);
        setSetupStatus('downloading_binaries');

        try {
            // Listen for download progress
            const unlistenProgress = await listen<DownloadProgress>('download-progress', (event) => {
                setDownloadProgress(event.payload);
                if (event.payload.step === 'model') {
                    setSetupStatus('downloading_model');
                }
            });

            const unlistenComplete = await listen('setup-complete', () => {
                setSetupStatus('complete');
                setDownloadProgress(null);
                unlistenProgress();
            });

            // Start binary download
            await invoke('download_binaries', { gpuType: detectedGpuLocal.gpu_type });

            // Start model download
            await invoke('download_model');

            // Now switch to offline mode
            await handleConfirmSwitch();

            unlistenComplete();
        } catch (err) {
            console.error('Setup failed:', err);
            setSetupStatus('needs_setup');
            setDownloadProgress(null);
            toast.error('Setup failed. Please try again.');
        }
    };

    const getGpuTypeBadge = (type: string) => {
        const colors: Record<string, string> = {
            nvidia: 'bg-green-500/20 text-green-400 border-green-500/30',
            amd: 'bg-red-500/20 text-red-400 border-red-500/30',
            intel: 'bg-blue-500/20 text-blue-400 border-blue-500/30',
            cpu: 'bg-gray-500/20 text-gray-400 border-gray-500/30',
        };
        return colors[type] || colors.cpu;
    };

    return (
        <>
            <Tooltip>
                <TooltipTrigger asChild>
                    <Button
                        variant="ghost"
                        size="icon"
                        className={`h-7 w-7 relative transition-colors ${connectionMode === 'cloud'
                            ? 'text-green-400 hover:text-green-300 hover:bg-green-500/10'
                            : 'text-purple-400 hover:text-purple-300 hover:bg-purple-500/10'
                            }`}
                        onClick={handleToggleClick}
                        disabled={isLoading || isBusy || isChecking}
                    >
                        {isLoading || isChecking ? (
                            <Loader2 className="w-4 h-4 animate-spin" />
                        ) : connectionMode === 'cloud' ? (
                            <Wifi className="w-4 h-4" />
                        ) : (
                            <WifiOff className="w-4 h-4" />
                        )}

                        {/* Loading progress indicator */}
                        {isLoading && modelLoadProgress !== null && (
                            <div
                                className="absolute bottom-0 left-0 h-0.5 bg-primary rounded-full transition-all"
                                style={{ width: `${modelLoadProgress}%` }}
                            />
                        )}
                    </Button>
                </TooltipTrigger>
                <TooltipContent side="bottom">
                    <div className="flex flex-col gap-1">
                        <span className="font-semibold">
                            {connectionMode === 'cloud' ? 'Cloud Mode' : 'Offline Mode'}
                        </span>
                        <span className="text-xs text-muted-foreground">
                            {connectionMode === 'cloud'
                                ? 'Using Gemma cloud models'
                                : 'Using local Qwen model'}
                        </span>
                        {isLoading && modelLoadProgress !== null && (
                            <span className="text-xs">
                                {connectionMode === 'cloud' ? 'Unloading' : 'Loading'}: {modelLoadProgress}%
                            </span>
                        )}
                    </div>
                </TooltipContent>
            </Tooltip>

            {/* Mode Switch Confirmation Dialog */}
            <Dialog open={showConfirmDialog} onOpenChange={setShowConfirmDialog}>
                <DialogContent className="bg-card border-border/40">
                    <DialogHeader>
                        <DialogTitle className="flex items-center gap-2">
                            {pendingMode === 'offline' ? (
                                <>
                                    <WifiOff className="w-5 h-5 text-purple-400" />
                                    Switch to Offline Mode?
                                </>
                            ) : (
                                <>
                                    <Wifi className="w-5 h-5 text-green-400" />
                                    Switch to Cloud Mode?
                                </>
                            )}
                        </DialogTitle>
                        <DialogDescription>
                            {pendingMode === 'offline' ? (
                                <>
                                    The local model will be loaded into memory. This may take a moment
                                    depending on your system specs.
                                </>
                            ) : (
                                <>
                                    The local model will be unloaded from memory. You'll use Gemma
                                    cloud models which offer faster response times and model cascade.
                                </>
                            )}
                        </DialogDescription>
                    </DialogHeader>
                    <DialogFooter>
                        <Button variant="outline" onClick={() => setShowConfirmDialog(false)}>
                            Cancel
                        </Button>
                        <Button onClick={handleConfirmSwitch}>
                            Switch Mode
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>

            {/* Setup Required Dialog */}
            <Dialog open={showSetupDialog} onOpenChange={setShowSetupDialog}>
                <DialogContent className="bg-card border-border/40 max-w-md">
                    <DialogHeader>
                        <DialogTitle className="flex items-center gap-2">
                            <Download className="w-5 h-5 text-primary" />
                            Setup Required for Offline Mode
                        </DialogTitle>
                        <DialogDescription>
                            To use offline mode, we need to download the required files for your hardware.
                        </DialogDescription>
                    </DialogHeader>

                    {detectedGpuLocal && (
                        <div className="space-y-4">
                            {/* GPU Detection Result */}
                            <div className="p-3 rounded-lg bg-secondary/30 border border-border/40">
                                <div className="text-xs text-muted-foreground mb-1">Detected Hardware</div>
                                <div className="flex items-center gap-2">
                                    <Badge className={getGpuTypeBadge(detectedGpuLocal.gpu_type)}>
                                        {detectedGpuLocal.gpu_type.toUpperCase()}
                                    </Badge>
                                    <span className="text-sm font-medium">{detectedGpuLocal.name}</span>
                                </div>
                                {detectedGpuLocal.vram_mb && (
                                    <div className="text-xs text-muted-foreground mt-1">
                                        VRAM: {(detectedGpuLocal.vram_mb / 1024).toFixed(1)} GB
                                    </div>
                                )}
                            </div>

                            {/* Download Info */}
                            <div className="space-y-2 text-sm">
                                <div className="flex items-center gap-2">
                                    <div className="w-2 h-2 rounded-full bg-primary" />
                                    <span>GPU Libraries (~50-100 MB)</span>
                                </div>
                                <div className="flex items-center gap-2">
                                    <div className="w-2 h-2 rounded-full bg-primary" />
                                    <span>Qwen 2.5 Coder 7B Model (~7 GB)</span>
                                </div>
                            </div>

                            {/* Warning for CPU */}
                            {detectedGpuLocal.gpu_type === 'cpu' && (
                                <div className="flex items-start gap-2 p-2 rounded-lg bg-amber-500/10 border border-amber-500/30">
                                    <AlertTriangle className="w-4 h-4 text-amber-400 mt-0.5 shrink-0" />
                                    <span className="text-xs text-amber-200">
                                        No dedicated GPU detected. Inference will be slower on CPU only.
                                    </span>
                                </div>
                            )}
                        </div>
                    )}

                    <DialogFooter>
                        <Button variant="outline" onClick={() => setShowSetupDialog(false)}>
                            Cancel
                        </Button>
                        <Button onClick={handleStartSetup} className="gap-2">
                            <Download className="w-4 h-4" />
                            Download & Setup
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>

            {/* Download Progress Overlay with Pause/Resume/Cancel */}
            {downloadProgress && (
                <div className="fixed inset-0 bg-background/80 backdrop-blur-sm z-50 flex items-center justify-center">
                    <div className="bg-card border border-border/40 rounded-xl p-6 max-w-md w-full mx-4 shadow-2xl">
                        <div className="text-center mb-4">
                            {downloadProgress.state === 'paused' ? (
                                <Pause className="w-10 h-10 text-amber-400 mx-auto mb-3" />
                            ) : (
                                <Loader2 className="w-10 h-10 animate-spin text-primary mx-auto mb-3" />
                            )}
                            <h3 className="text-lg font-semibold">
                                {downloadProgress.state === 'paused'
                                    ? 'Download Paused'
                                    : downloadProgress.step === 'binaries'
                                        ? 'Downloading GPU Libraries'
                                        : 'Downloading AI Model'}
                            </h3>
                            <p className="text-sm text-muted-foreground mt-1">
                                {downloadProgress.state === 'paused'
                                    ? 'Click Resume to continue downloading'
                                    : downloadProgress.step === 'binaries'
                                        ? 'Setting up hardware acceleration...'
                                        : 'This may take a while depending on your connection...'}
                            </p>
                        </div>

                        <Progress value={downloadProgress.percent} className="h-2 mb-3" />

                        <div className="flex justify-between text-xs text-muted-foreground mb-4">
                            <span>{downloadProgress.percent.toFixed(1)}%</span>
                            <span>
                                {downloadProgress.state === 'paused'
                                    ? 'Paused'
                                    : `${downloadProgress.speed_mbps.toFixed(1)} MB/s`}
                            </span>
                            <span>
                                {downloadProgress.state === 'paused'
                                    ? '--'
                                    : downloadProgress.eta_seconds > 60
                                        ? `${Math.floor(downloadProgress.eta_seconds / 60)}m ${downloadProgress.eta_seconds % 60}s remaining`
                                        : `${downloadProgress.eta_seconds}s remaining`}
                            </span>
                        </div>

                        {/* Control Buttons */}
                        <div className="flex gap-2">
                            {downloadProgress.state === 'paused' ? (
                                <Button
                                    className="flex-1 gap-2 bg-green-600 hover:bg-green-700"
                                    onClick={async () => {
                                        try {
                                            await invoke('resume_download');
                                            setDownloadProgress({ ...downloadProgress, state: 'downloading' });
                                        } catch (err) {
                                            console.error('Resume failed:', err);
                                        }
                                    }}
                                >
                                    <Play className="w-4 h-4" />
                                    Resume
                                </Button>
                            ) : (
                                <Button
                                    variant="outline"
                                    className="flex-1 gap-2 border-amber-500/50 text-amber-400 hover:bg-amber-500/10"
                                    onClick={async () => {
                                        try {
                                            await invoke('pause_download');
                                            setDownloadProgress({ ...downloadProgress, state: 'paused' });
                                        } catch (err) {
                                            console.error('Pause failed:', err);
                                        }
                                    }}
                                >
                                    <Pause className="w-4 h-4" />
                                    Pause
                                </Button>
                            )}
                            <Button
                                variant="outline"
                                className="gap-2 border-red-500/50 text-red-400 hover:bg-red-500/10"
                                onClick={async () => {
                                    try {
                                        await invoke('cancel_download');
                                        setDownloadProgress(null);
                                        setSetupStatus('needs_setup');
                                        toast.error('Download cancelled');
                                    } catch (err) {
                                        console.error('Cancel failed:', err);
                                    }
                                }}
                            >
                                <X className="w-4 h-4" />
                                Cancel
                            </Button>
                        </div>
                    </div>
                </div>
            )}
        </>
    );
}
