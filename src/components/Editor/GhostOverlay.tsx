import { motion, AnimatePresence } from 'framer-motion';
import { ReactNode } from 'react';

interface GhostOverlayProps {
    visible: boolean;
    children?: ReactNode;
    className?: string;
}

/**
 * Ghost overlay component for showing semi-transparent overlays
 * during agent operations or pending states
 */
export default function GhostOverlay({ visible, children, className = '' }: GhostOverlayProps) {
    return (
        <AnimatePresence>
            {visible && (
                <motion.div
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    transition={{ duration: 0.2 }}
                    className={`absolute inset-0 bg-background/80 backdrop-blur-sm z-50 flex items-center justify-center ${className}`}
                >
                    {children}
                </motion.div>
            )}
        </AnimatePresence>
    );
}
