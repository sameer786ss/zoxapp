import { motion } from 'framer-motion';
import { useState, useEffect } from 'react';

interface SplashScreenProps {
    onReady?: () => void;
}

export default function SplashScreen({ onReady }: SplashScreenProps) {
    const [statusText, setStatusText] = useState('Initializing...');
    const [progress, setProgress] = useState(0);

    useEffect(() => {
        // Simulate initialization phases
        const phases = [
            { text: 'Loading components...', progress: 20 },
            { text: 'Connecting to backend...', progress: 50 },
            { text: 'Preparing workspace...', progress: 80 },
            { text: 'Ready!', progress: 100 },
        ];

        let currentPhase = 0;
        const interval = setInterval(() => {
            if (currentPhase < phases.length) {
                setStatusText(phases[currentPhase].text);
                setProgress(phases[currentPhase].progress);
                currentPhase++;
            } else {
                clearInterval(interval);
                // Give a moment for the final animation
                setTimeout(() => onReady?.(), 300);
            }
        }, 400);

        return () => clearInterval(interval);
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

            {/* Logo with shimmer effect */}
            <motion.div
                initial={{ opacity: 0, scale: 0.8 }}
                animate={{ opacity: 1, scale: 1 }}
                transition={{ duration: 0.5, ease: 'easeOut' }}
                className="relative mb-8"
            >
                {/* Logo container */}
                <div className="relative w-32 h-32 flex items-center justify-center">
                    {/* Rotating ring */}
                    <motion.div
                        className="absolute inset-0 rounded-full border-2 border-primary/30"
                        animate={{ rotate: 360 }}
                        transition={{
                            duration: 3,
                            repeat: Infinity,
                            ease: 'linear',
                        }}
                    >
                        <div className="absolute top-0 left-1/2 -translate-x-1/2 -translate-y-1/2 w-3 h-3 rounded-full bg-primary" />
                    </motion.div>

                    {/* ZOX Logo */}
                    <motion.div
                        className="text-5xl font-black tracking-tighter bg-gradient-to-br from-primary via-blue-400 to-primary bg-clip-text text-transparent"
                        animate={{
                            backgroundPosition: ['0% 50%', '100% 50%', '0% 50%'],
                        }}
                        transition={{
                            duration: 3,
                            repeat: Infinity,
                            ease: 'easeInOut',
                        }}
                        style={{
                            backgroundSize: '200% 200%',
                        }}
                    >
                        ZOX
                    </motion.div>
                </div>

                {/* Shimmer effect overlay */}
                <motion.div
                    className="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent -skew-x-12"
                    animate={{
                        x: ['-200%', '200%'],
                    }}
                    transition={{
                        duration: 2,
                        repeat: Infinity,
                        ease: 'easeInOut',
                        repeatDelay: 1,
                    }}
                />
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
