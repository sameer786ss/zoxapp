import { useState, Suspense, lazy, Component, ErrorInfo, ReactNode, useEffect, useRef } from 'react';
import MainLayout from '@/components/Layout/MainLayout';
import ChatPanel from '@/components/Chat/ChatPanel';
import { Toaster } from '@/components/ui/sonner';
import CommandPalette from '@/components/Common/CommandPalette';
import OnboardingScreen from '@/components/Onboarding/OnboardingScreen';
import SplashScreen from '@/components/Onboarding/SplashScreen';
import { useAgentStore, ConnectionMode } from '@/stores/useAgentStore';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { Loader2, AlertTriangle, RefreshCw } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

// Lazy load heavy components to prevent UI freeze
const MonacoWrapper = lazy(() => import('@/components/Editor/MonacoWrapper'));
const ResizableLayout = lazy(() => import('@/components/Layout/ResizableLayout'));
const TerminalPanel = lazy(() => import('@/components/Terminal/TerminalPanel'));

// Loading fallback component
function LoadingFallback() {
  return (
    <div className="h-full w-full flex items-center justify-center bg-card/20">
      <div className="flex items-center gap-3 text-muted-foreground">
        <Loader2 className="w-5 h-5 animate-spin" />
        <span>Loading...</span>
      </div>
    </div>
  );
}

// Error Boundary Component
interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null, errorInfo: null };
  }

  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('Error caught by boundary:', error, errorInfo);
    this.setState({ errorInfo });
  }

  handleReset = () => {
    this.setState({ hasError: false, error: null, errorInfo: null });
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div className="h-full w-full flex items-center justify-center bg-background p-8">
          <Card className="max-w-md w-full bg-destructive/5 border-destructive/30">
            <CardHeader className="text-center pb-2">
              <div className="w-16 h-16 rounded-2xl bg-destructive/20 flex items-center justify-center mx-auto mb-4">
                <AlertTriangle className="w-8 h-8 text-destructive" />
              </div>
              <CardTitle className="text-destructive">Something went wrong</CardTitle>
              <CardDescription>
                {this.state.error?.message || 'An unexpected error occurred'}
              </CardDescription>
            </CardHeader>
            <CardContent className="flex flex-col items-center gap-4">
              <Button onClick={this.handleReset} variant="outline" className="gap-2">
                <RefreshCw className="w-4 h-4" />
                Try Again
              </Button>
              {this.state.errorInfo && (
                <details className="w-full">
                  <summary className="text-xs text-muted-foreground cursor-pointer hover:text-foreground">
                    Error details
                  </summary>
                  <pre className="mt-2 p-2 bg-muted rounded text-xs overflow-auto max-h-40">
                    {this.state.errorInfo.componentStack}
                  </pre>
                </details>
              )}
            </CardContent>
          </Card>
        </div>
      );
    }

    return this.props.children;
  }
}

// Component-specific error fallback
function EditorErrorFallback() {
  return (
    <div className="h-full w-full flex items-center justify-center bg-card/20">
      <div className="flex flex-col items-center gap-3 text-muted-foreground">
        <AlertTriangle className="w-8 h-8 text-destructive/60" />
        <span>Editor failed to load</span>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => window.location.reload()}
        >
          Reload App
        </Button>
      </div>
    </div>
  );
}

function App() {
  const { mode, setConnectionMode } = useAgentStore();
  const [showTerminal, setShowTerminal] = useState(false);
  const [terminalMinimized, setTerminalMinimized] = useState(false);
  const [showOnboarding, setShowOnboarding] = useState(() => {
    return !localStorage.getItem('zox-onboarding-complete');
  });
  const [isInitializing, setIsInitializing] = useState(true);
  const unlistenRef = useRef<UnlistenFn | null>(null);

  // Listen for app-ready event from backend
  useEffect(() => {
    // Notify backend that frontend is mounted and listeners are ready
    import('@tauri-apps/api/event').then(({ emit }) => {
      emit('frontend_loaded', true).catch(err => console.error('Failed to emit loaded:', err));
    });

    listen<boolean>('app-ready', () => {
      // Small delay for smooth transition
      setTimeout(() => setIsInitializing(false), 500);
    }).then(fn => { unlistenRef.current = fn; });

    // Fallback timeout in case backend doesn't emit
    const fallbackTimeout = setTimeout(() => {
      console.warn("Backend didn't respond to frontend_loaded, forcing startup");
      setIsInitializing(false);
    }, 3500);

    return () => {
      unlistenRef.current?.();
      clearTimeout(fallbackTimeout);
    };
  }, []);

  // Load saved connection mode on mount
  useEffect(() => {
    const savedMode = localStorage.getItem('zox-default-connection-mode') as ConnectionMode | null;
    if (savedMode && (savedMode === 'cloud' || savedMode === 'offline')) {
      setConnectionMode(savedMode);
    }
  }, [setConnectionMode]);

  const handleOnboardingComplete = (_selectedMode: 'cloud' | 'offline') => {
    setShowOnboarding(false);
    // If user selected offline, we'll let the ConnectionToggle handle the setup
  };

  // Show splash screen during initialization
  if (isInitializing) {
    return (
      <>
        <SplashScreen onReady={() => setIsInitializing(false)} />
        <Toaster />
      </>
    );
  }

  // Show onboarding for first-time users
  if (showOnboarding) {
    return (
      <>
        <OnboardingScreen onComplete={handleOnboardingComplete} />
        <Toaster />
      </>
    );
  }

  return (
    <ErrorBoundary>
      <>
        <MainLayout>
          <div className="relative w-full h-full overflow-hidden flex bg-background">
            {/* Chat-only mode */}
            {mode === 'chat' && (
              <div className="w-full h-full flex items-center justify-center p-4">
                <div className="w-full max-w-3xl h-full bg-card/40 backdrop-blur-sm rounded-2xl overflow-hidden shadow-elevation-2">
                  <ErrorBoundary fallback={<div className="p-4 text-destructive">Chat failed to load</div>}>
                    <ChatPanel />
                  </ErrorBoundary>
                </div>
              </div>
            )}

            {/* Turbo mode with resizable panels */}
            {mode === 'turbo' && (
              <Suspense fallback={<LoadingFallback />}>
                <ErrorBoundary>
                  <ResizableLayout
                    left={
                      <ErrorBoundary fallback={<div className="p-4 text-destructive">Chat failed to load</div>}>
                        <ChatPanel />
                      </ErrorBoundary>
                    }
                    right={
                      <Suspense fallback={<LoadingFallback />}>
                        <ErrorBoundary fallback={<EditorErrorFallback />}>
                          <MonacoWrapper />
                        </ErrorBoundary>
                      </Suspense>
                    }
                    bottom={
                      showTerminal && (
                        <Suspense fallback={<LoadingFallback />}>
                          <ErrorBoundary>
                            <TerminalPanel
                              onClose={() => setShowTerminal(false)}
                              onMinimize={() => setTerminalMinimized(!terminalMinimized)}
                              isMinimized={terminalMinimized}
                            />
                          </ErrorBoundary>
                        </Suspense>
                      )
                    }
                    showBottom={showTerminal}
                    defaultLeftSize={40}
                    defaultBottomSize={terminalMinimized ? 5 : 25}
                  />
                </ErrorBoundary>
              </Suspense>
            )}
          </div>
        </MainLayout>

        {/* Global Components */}
        <Toaster />
        <CommandPalette />
      </>
    </ErrorBoundary>
  );
}

export default App;
