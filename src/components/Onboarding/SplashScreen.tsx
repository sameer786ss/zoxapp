import { motion } from 'framer-motion';
import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';

interface SplashScreenProps {
    onReady?: () => void;
}

export default function SplashScreen({ onReady }: SplashScreenProps) {
    const [statusText, setStatusText] = useState('Initializing...');
    const [progress, setProgress] = useState(0);

    useEffect(() => {
        let mounted = true;

        // Listen for real backend initialization events
        const setupListeners = async () => {
            try {
                // Listen for app initialization progress from backend
                const unlistenInit = await listen<{ stage: string; progress: number }>('app-init-progress', (event) => {
                    if (!mounted) return;
                    setStatusText(event.payload.stage);
                    setProgress(event.payload.progress);
                });

                // Listen for app ready signal
                const unlistenReady = await listen('app-init-complete', () => {
                    if (!mounted) return;
                    setStatusText('Ready!');
                    setProgress(100);
                    setTimeout(() => onReady?.(), 200);
                });

                // Fallback: If no events received within 2s, proceed anyway
                // This handles the case where backend doesn't emit events
                const fallbackTimer = setTimeout(() => {
                    if (mounted && progress < 100) {
                        console.log('[SplashScreen] Fallback: No events received, proceeding...');
                        setStatusText('Ready!');
                        setProgress(100);
                        setTimeout(() => onReady?.(), 200);
                    }
                }, 2000);

                return () => {
                    unlistenInit();
                    unlistenReady();
                    clearTimeout(fallbackTimer);
                };
            } catch (e) {
                // If Tauri events fail (e.g., in browser dev), use simple timer fallback
                console.log('[SplashScreen] Tauri events not available, using fallback');
                setTimeout(() => {
                    if (mounted) {
                        setStatusText('Ready!');
                        setProgress(100);
                        setTimeout(() => onReady?.(), 200);
                    }
                }, 1000);
            }
        };

        setupListeners();

        return () => {
            mounted = false;
        };
    }, [onReady]);

    return (
        <div className="fixed inset-0 z-50 flex flex-col items-center justify-center bg-background">
            {/* Animated Background */}
            <div className="absolute inset-0 overflow-hidden">
                <div className="absolute inset-0 bg-gradient-to-br from-primary/5 via-transparent to-blue-500/5" />
                {/* Animated orbs */}
                <motion.div
                    className="absolute w-96 h-96 rounded-full bg-primary/10 blur-3xl"
                    animate={{
                        x: ['-50%', '50%', '-50%'],
                        y: ['-50%', '50%', '-50%'],
                    }}
                    transition={{
                        duration: 8,
                        repeat: Infinity,
                        ease: 'easeInOut',
                    }}
                    style={{ top: '20%', left: '30%' }}
                />
                <motion.div
                    className="absolute w-64 h-64 rounded-full bg-blue-500/10 blur-3xl"
                    animate={{
                        x: ['50%', '-50%', '50%'],
                        y: ['50%', '-50%', '50%'],
                    }}
                    transition={{
                        duration: 10,
                        repeat: Infinity,
                        ease: 'easeInOut',
                    }}
                    style={{ bottom: '20%', right: '30%' }}
                />
            </div>

            {/* Logo */}
            <motion.div
                initial={{ opacity: 0, scale: 0.8 }}
                animate={{ opacity: 1, scale: 1 }}
                transition={{ duration: 0.5, ease: 'easeOut' }}
                className="relative mb-8"
            >
                <div className="relative w-32 h-32 flex items-center justify-center">
                    <img
                        src="/zox-logo.png"
                        alt="ZOX Logo"
                        className="w-24 h-24 object-contain filter drop-shadow-lg"
                    />
                </div>
            </motion.div>

            {/* Status text */}
            <motion.div
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.3 }}
                className="text-center"
            >
                <motion.p
                    key={statusText}
                    initial={{ opacity: 0, y: 5 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, y: -5 }}
                    className="text-sm text-muted-foreground mb-4"
                >
                    {statusText}
                </motion.p>

                {/* Progress bar */}
                <div className="w-48 h-1 bg-secondary rounded-full overflow-hidden">
                    <motion.div
                        className="h-full bg-gradient-to-r from-primary to-blue-500"
                        initial={{ width: 0 }}
                        animate={{ width: `${progress}%` }}
                        transition={{ duration: 0.3, ease: 'easeOut' }}
                    />
                </div>
            </motion.div>

            {/* Version */}
            <motion.p
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ delay: 0.5 }}
                className="absolute bottom-8 text-xs text-muted-foreground/50"
            >
                v0.1.0
            </motion.p>
        </div>
    );
}
