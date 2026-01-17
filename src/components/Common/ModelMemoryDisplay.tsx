//! Model Memory Display Component
//!
//! Displays VRAM/memory usage for loaded models.

import { memo, useEffect, useState } from 'react';
import { Cpu, HardDrive } from 'lucide-react';
import { Badge } from '@/components/ui/badge';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';

interface ModelMemoryDisplayProps {
    isModelLoaded?: boolean;
    deviceName?: string;
    className?: string;
    // New props for direct display
    gpuType?: string;
    vramMb?: number;
    modelSize?: string;
}

interface MemoryInfo {
    usedMb: number;
    totalMb: number;
    percentage: number;
}

export const ModelMemoryDisplay = memo(function ModelMemoryDisplay({
    isModelLoaded,
    deviceName,
    className = '',
    gpuType,
    vramMb,
    modelSize,
}: ModelMemoryDisplayProps) {
    const [memoryInfo, setMemoryInfo] = useState<MemoryInfo | null>(null);

    // Estimate memory based on device and model
    useEffect(() => {
        // Mode 1: Direct VRAM display (offline mode check)
        if (vramMb !== undefined && gpuType) {
            setMemoryInfo({
                usedMb: 0, // Not tracking usage, just capacity? Or maybe we want to show model size vs capacity?
                // Let's assume we want to show total capacity
                totalMb: vramMb,
                percentage: 0 // Optional
            });
            return;
        }

        // Mode 2: Live model usage estimate
        if (!isModelLoaded) {
            setMemoryInfo(null);
            return;
        }

        const currentDevice = deviceName || 'CPU';

        // Estimate based on typical GGUF Q4 model sizes
        // Gemma 2B ~1.5GB, Gemma 7B ~4GB, Gemma 12B ~7GB
        const estimatedUsage = {
            usedMb: 1500, // Default estimate for smaller models
            totalMb: currentDevice.toLowerCase().includes('nvidia') ? 8000 :
                currentDevice.toLowerCase().includes('metal') ? 16000 :
                    32000,
            percentage: 0,
        };

        // Adjust for model size if known?

        estimatedUsage.percentage = Math.round((estimatedUsage.usedMb / estimatedUsage.totalMb) * 100);
        setMemoryInfo(estimatedUsage);
    }, [isModelLoaded, deviceName, gpuType, vramMb]);

    if ((!isModelLoaded && !vramMb) || !memoryInfo) return null;

    const isGpu = (gpuType || deviceName || '').toLowerCase().includes('nvidia') ||
        (gpuType || deviceName || '').toLowerCase().includes('metal') ||
        (gpuType && gpuType !== 'cpu');

    const Icon = isGpu ? HardDrive : Cpu;
    const label = isGpu ? 'VRAM' : 'RAM';

    // Color based on usage percentage (only if tracking usage)
    const getVariant = () => {
        if (!isModelLoaded) return 'outline'; // Just capacity display
        if (memoryInfo.percentage < 50) return 'secondary';
        if (memoryInfo.percentage < 80) return 'outline';
        return 'destructive';
    };

    return (
        <Tooltip>
            <TooltipTrigger asChild>
                <Badge
                    variant={getVariant()}
                    className={`gap-1 text-xs cursor-default ${className}`}
                >
                    <Icon className="w-3 h-3" />
                    <span>{label}: {(memoryInfo.usedMb / 1024).toFixed(1)}GB</span>
                </Badge>
            </TooltipTrigger>
            <TooltipContent>
                <div className="text-xs space-y-1">
                    <div>Device: {deviceName}</div>
                    <div>Usage: {memoryInfo.usedMb}MB / {memoryInfo.totalMb}MB</div>
                    <div>Percentage: {memoryInfo.percentage}%</div>
                </div>
            </TooltipContent>
        </Tooltip>
    );
});

export default ModelMemoryDisplay;
