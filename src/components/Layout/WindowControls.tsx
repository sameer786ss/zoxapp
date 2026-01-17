import { getCurrentWindow } from '@tauri-apps/api/window';
import { Minus, Square, X, Copy } from 'lucide-react';
import { useState, useEffect, useCallback } from 'react';

export default function WindowControls() {
    const [isMaximized, setIsMaximized] = useState(false);

    useEffect(() => {
        // Get window reference inside effect to ensure Tauri is ready
        const appWindow = getCurrentWindow();

        // Check initial maximized state
        const checkMaximized = async () => {
            try {
                const maximized = await appWindow.isMaximized();
                setIsMaximized(maximized);
            } catch (error) {
                console.error('Failed to check maximized state:', error);
            }
        };
        checkMaximized();

        // Listen for resize events to update maximized state
        const unlisten = appWindow.onResized(async () => {
            try {
                const maximized = await appWindow.isMaximized();
                setIsMaximized(maximized);
            } catch (error) {
                console.error('Failed to check maximized state:', error);
            }
        });

        return () => {
            unlisten.then(fn => fn());
        };
    }, []);

    const handleMinimize = useCallback(async () => {
        try {
            const appWindow = getCurrentWindow();
            await appWindow.minimize();
        } catch (error) {
            console.error('Failed to minimize window:', error);
        }
    }, []);

    const handleMaximize = useCallback(async () => {
        try {
            const appWindow = getCurrentWindow();
            await appWindow.toggleMaximize();
        } catch (error) {
            console.error('Failed to toggle maximize:', error);
        }
    }, []);

    const handleClose = useCallback(async () => {
        try {
            const appWindow = getCurrentWindow();
            await appWindow.close();
        } catch (error) {
            console.error('Failed to close window:', error);
        }
    }, []);

    return (
        <div className="flex h-full">
            {/* Minimize Button */}
            <button
                onClick={handleMinimize}
                className="window-control group"
                aria-label="Minimize"
            >
                <Minus className="w-4 h-4 text-foreground/80 group-hover:text-foreground transition-colors duration-faster ease-fluent" />
            </button>

            {/* Maximize/Restore Button */}
            <button
                onClick={handleMaximize}
                className="window-control group"
                aria-label={isMaximized ? 'Restore' : 'Maximize'}
            >
                {isMaximized ? (
                    <Copy className="w-3.5 h-3.5 text-foreground/80 group-hover:text-foreground transition-colors duration-faster ease-fluent rotate-180" />
                ) : (
                    <Square className="w-3 h-3 text-foreground/80 group-hover:text-foreground transition-colors duration-faster ease-fluent" />
                )}
            </button>

            {/* Close Button */}
            <button
                onClick={handleClose}
                className="window-control window-control-close group"
                aria-label="Close"
            >
                <X className="w-4 h-4 text-foreground/80 group-hover:text-white transition-colors duration-faster ease-fluent" />
            </button>
        </div>
    );
}
