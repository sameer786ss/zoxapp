import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
    Wifi,
    WifiOff,
    Sparkles,
    Shield,
    Zap,
    Cloud,
    HardDrive,
    ArrowRight,
    ChevronLeft,
    ChevronRight,
    Check
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useAgentStore } from '@/stores/useAgentStore';

interface OnboardingScreenProps {
    onComplete: (mode: 'cloud' | 'offline') => void;
}

const ONBOARDING_SLIDES = [
    {
        id: 'welcome',
        title: 'Welcome to ZOX',
        subtitle: 'Your AI-Powered Coding Companion',
        description: 'ZOX is a next-generation coding assistant that helps you write, debug, and understand code faster than ever.',
        icon: Sparkles,
        color: 'from-violet-500/20 to-purple-500/20',
        iconColor: 'text-violet-400',
    },
    {
        id: 'cloud',
        title: 'Cloud Intelligence',
        subtitle: 'Powered by Gemma Model Cascade',
        description: 'Our cloud mode uses a sophisticated cascade of Gemma models (1B→4B→12B→27B) to provide lightning-fast responses with the right amount of intelligence for each task.',
        icon: Cloud,
        color: 'from-blue-500/20 to-cyan-500/20',
        iconColor: 'text-blue-400',
        features: ['Multi-model cascade', 'Fastest response times', 'Always up-to-date'],
    },
    {
        id: 'offline',
        title: 'Offline Mode',
        subtitle: 'Privacy-First Local Inference',
        description: 'Run a powerful 7B coding model locally on your machine. Your code never leaves your computer. Perfect for sensitive projects.',
        icon: HardDrive,
        color: 'from-purple-500/20 to-pink-500/20',
        iconColor: 'text-purple-400',
        features: ['Complete privacy', 'No internet required', 'GPU accelerated'],
    },
    {
        id: 'turbo',
        title: 'Turbo Mode',
        subtitle: 'Autonomous Coding Agent',
        description: 'Let ZOX take the wheel. In Turbo mode, ZOX can read, write, and execute code autonomously. You approve each action with a single click.',
        icon: Zap,
        color: 'from-amber-500/20 to-orange-500/20',
        iconColor: 'text-amber-400',
        features: ['File operations', 'Terminal commands', 'Smart approvals'],
    },
    {
        id: 'security',
        title: 'Your Security Matters',
        subtitle: 'Designed with Privacy in Mind',
        description: 'Every tool execution requires your explicit approval. ZOX never accesses files or runs commands without permission.',
        icon: Shield,
        color: 'from-emerald-500/20 to-green-500/20',
        iconColor: 'text-emerald-400',
        features: ['Approval-based actions', 'Sandboxed execution', 'Full transparency'],
    },
];

export default function OnboardingScreen({ onComplete }: OnboardingScreenProps) {
    const [currentSlide, setCurrentSlide] = useState(0);
    const [showModeSelection, setShowModeSelection] = useState(false);
    const [selectedMode, setSelectedMode] = useState<'cloud' | 'offline' | null>(null);
    const { setConnectionMode } = useAgentStore();

    const isLastSlide = currentSlide === ONBOARDING_SLIDES.length - 1;
    const slide = ONBOARDING_SLIDES[currentSlide];
    const SlideIcon = slide.icon;

    const handleNext = () => {
        if (isLastSlide) {
            setShowModeSelection(true);
        } else {
            setCurrentSlide((prev) => prev + 1);
        }
    };

    const handlePrev = () => {
        if (showModeSelection) {
            setShowModeSelection(false);
        } else if (currentSlide > 0) {
            setCurrentSlide((prev) => prev - 1);
        }
    };

    const handleModeSelect = (mode: 'cloud' | 'offline') => {
        setSelectedMode(mode);
    };

    const handleComplete = () => {
        if (!selectedMode) return;

        // Save onboarding completion
        localStorage.setItem('zox-onboarding-complete', 'true');
        localStorage.setItem('zox-default-connection-mode', selectedMode);

        setConnectionMode(selectedMode);
        onComplete(selectedMode);
    };

    const handleSkip = () => {
        // Default to cloud mode on skip
        localStorage.setItem('zox-onboarding-complete', 'true');
        localStorage.setItem('zox-default-connection-mode', 'cloud');
        setConnectionMode('cloud');
        onComplete('cloud');
    };

    return (
        <div className="fixed inset-0 bg-background z-50 flex items-center justify-center overflow-hidden">
            <div className="relative w-full max-w-2xl px-6">
                {/* Skip button */}
                <motion.div
                    className="absolute -top-16 right-0"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    transition={{ delay: 0.5 }}
                >
                    <Button
                        variant="ghost"
                        className="text-muted-foreground hover:text-foreground"
                        onClick={handleSkip}
                    >
                        Skip
                    </Button>
                </motion.div>

                <AnimatePresence mode="wait">
                    {!showModeSelection ? (
                        <motion.div
                            key={`slide-${currentSlide}`}
                            initial={{ opacity: 0 }}
                            animate={{ opacity: 1 }}
                            exit={{ opacity: 0 }}
                            transition={{ duration: 0.2 }}
                        >
                            <Card className="bg-card/80 backdrop-blur-xl border-border/40 shadow-2xl">
                                <CardHeader className="text-center pb-2">
                                    {/* Icon */}
                                    <motion.div
                                        className={`w-20 h-20 rounded-2xl bg-gradient-to-br ${slide.color} flex items-center justify-center mx-auto mb-4 border border-white/10`}
                                        initial={{ scale: 0.8, rotate: -10 }}
                                        animate={{ scale: 1, rotate: 0 }}
                                        transition={{ type: 'spring', stiffness: 200 }}
                                    >
                                        <SlideIcon className={`w-10 h-10 ${slide.iconColor}`} />
                                    </motion.div>

                                    {/* Centered step counter */}
                                    <div className="flex justify-center mb-2">
                                        <Badge variant="outline" className="text-xs">
                                            Step {currentSlide + 1} of {ONBOARDING_SLIDES.length}
                                        </Badge>
                                    </div>

                                    <CardTitle className="text-2xl font-bold bg-gradient-to-r from-foreground to-foreground/70 bg-clip-text">
                                        {slide.title}
                                    </CardTitle>
                                    <CardDescription className="text-base text-primary/80">
                                        {slide.subtitle}
                                    </CardDescription>
                                </CardHeader>

                                <CardContent className="space-y-6">
                                    <p className="text-center text-muted-foreground leading-relaxed">
                                        {slide.description}
                                    </p>

                                    {/* Feature list */}
                                    {slide.features && (
                                        <div className="flex flex-wrap justify-center gap-2">
                                            {slide.features.map((feature, idx) => (
                                                <motion.div
                                                    key={feature}
                                                    initial={{ opacity: 0, y: 10 }}
                                                    animate={{ opacity: 1, y: 0 }}
                                                    transition={{ delay: 0.2 + idx * 0.1 }}
                                                >
                                                    <Badge
                                                        variant="secondary"
                                                        className="text-xs gap-1 bg-secondary/50"
                                                    >
                                                        <Check className="w-3 h-3 text-primary" />
                                                        {feature}
                                                    </Badge>
                                                </motion.div>
                                            ))}
                                        </div>
                                    )}

                                    {/* Navigation */}
                                    <div className="flex items-center justify-between pt-4">
                                        <Button
                                            variant="ghost"
                                            onClick={handlePrev}
                                            disabled={currentSlide === 0}
                                            className="gap-2"
                                        >
                                            <ChevronLeft className="w-4 h-4" />
                                            Back
                                        </Button>

                                        {/* Progress dots */}
                                        <div className="flex items-center gap-2">
                                            {ONBOARDING_SLIDES.map((_, idx) => (
                                                <button
                                                    key={idx}
                                                    onClick={() => setCurrentSlide(idx)}
                                                    className={`w-2 h-2 rounded-full transition-all ${idx === currentSlide
                                                        ? 'bg-primary w-6'
                                                        : 'bg-muted-foreground/30 hover:bg-muted-foreground/50'
                                                        }`}
                                                />
                                            ))}
                                        </div>

                                        <Button onClick={handleNext} className="gap-2">
                                            {isLastSlide ? 'Choose Mode' : 'Next'}
                                            <ChevronRight className="w-4 h-4" />
                                        </Button>
                                    </div>
                                </CardContent>
                            </Card>
                        </motion.div>
                    ) : (
                        <motion.div
                            key="mode-selection"
                            initial={{ opacity: 0, scale: 0.95 }}
                            animate={{ opacity: 1, scale: 1 }}
                            exit={{ opacity: 0, scale: 0.95 }}
                            transition={{ duration: 0.3 }}
                        >
                            <Card className="bg-card/80 backdrop-blur-xl border-border/40 shadow-2xl">
                                <CardHeader className="text-center">
                                    <motion.div
                                        className="w-20 h-20 rounded-full bg-gradient-to-br from-primary/20 to-purple-500/20 flex items-center justify-center mx-auto mb-4"
                                        initial={{ scale: 0 }}
                                        animate={{ scale: 1 }}
                                        transition={{ type: 'spring', delay: 0.1 }}
                                    >
                                        <img src="/zox-logo.png" alt="ZOX" className="w-12 h-12" />
                                    </motion.div>
                                    <CardTitle className="text-2xl">Choose Your Mode</CardTitle>
                                    <CardDescription>
                                        You can switch between modes anytime using the WiFi toggle
                                    </CardDescription>
                                </CardHeader>

                                <CardContent className="space-y-4">
                                    {/* Cloud Mode Card */}
                                    <motion.button
                                        className={`w-full p-4 rounded-xl border-2 transition-all text-left ${selectedMode === 'cloud'
                                            ? 'border-green-500 bg-green-500/10'
                                            : 'border-border/40 bg-secondary/20 hover:border-border'
                                            }`}
                                        onClick={() => handleModeSelect('cloud')}
                                        whileHover={{ scale: 1.01 }}
                                        whileTap={{ scale: 0.99 }}
                                    >
                                        <div className="flex items-start gap-4">
                                            <div className={`w-12 h-12 rounded-lg flex items-center justify-center ${selectedMode === 'cloud'
                                                ? 'bg-green-500/20'
                                                : 'bg-secondary/50'
                                                }`}>
                                                <Wifi className={`w-6 h-6 ${selectedMode === 'cloud'
                                                    ? 'text-green-400'
                                                    : 'text-muted-foreground'
                                                    }`} />
                                            </div>
                                            <div className="flex-1">
                                                <div className="flex items-center gap-2">
                                                    <span className="font-semibold">Cloud Mode</span>
                                                    <Badge variant="secondary" className="text-[10px]">Recommended</Badge>
                                                </div>
                                                <p className="text-sm text-muted-foreground mt-1">
                                                    Fastest responses with multi-model cascade. Requires internet connection.
                                                </p>
                                            </div>
                                            {selectedMode === 'cloud' && (
                                                <Check className="w-5 h-5 text-green-400" />
                                            )}
                                        </div>
                                    </motion.button>

                                    {/* Offline Mode Card */}
                                    <motion.button
                                        className={`w-full p-4 rounded-xl border-2 transition-all text-left ${selectedMode === 'offline'
                                            ? 'border-purple-500 bg-purple-500/10'
                                            : 'border-border/40 bg-secondary/20 hover:border-border'
                                            }`}
                                        onClick={() => handleModeSelect('offline')}
                                        whileHover={{ scale: 1.01 }}
                                        whileTap={{ scale: 0.99 }}
                                    >
                                        <div className="flex items-start gap-4">
                                            <div className={`w-12 h-12 rounded-lg flex items-center justify-center ${selectedMode === 'offline'
                                                ? 'bg-purple-500/20'
                                                : 'bg-secondary/50'
                                                }`}>
                                                <WifiOff className={`w-6 h-6 ${selectedMode === 'offline'
                                                    ? 'text-purple-400'
                                                    : 'text-muted-foreground'
                                                    }`} />
                                            </div>
                                            <div className="flex-1">
                                                <div className="flex items-center gap-2">
                                                    <span className="font-semibold">Offline Mode</span>
                                                    <Badge variant="outline" className="text-[10px]">~7GB Download</Badge>
                                                </div>
                                                <p className="text-sm text-muted-foreground mt-1">
                                                    Complete privacy with local AI. Requires GPU for best performance.
                                                </p>
                                            </div>
                                            {selectedMode === 'offline' && (
                                                <Check className="w-5 h-5 text-purple-400" />
                                            )}
                                        </div>
                                    </motion.button>

                                    {/* Navigation */}
                                    <div className="flex items-center justify-between pt-4">
                                        <Button variant="ghost" onClick={handlePrev} className="gap-2">
                                            <ChevronLeft className="w-4 h-4" />
                                            Back
                                        </Button>

                                        <Button
                                            onClick={handleComplete}
                                            disabled={!selectedMode}
                                            className="gap-2"
                                        >
                                            Get Started
                                            <ArrowRight className="w-4 h-4" />
                                        </Button>
                                    </div>
                                </CardContent>
                            </Card>
                        </motion.div>
                    )}
                </AnimatePresence>
            </div>
        </div>
    );
}
