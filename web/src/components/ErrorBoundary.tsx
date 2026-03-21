'use client';

import React, { Component, ReactNode } from 'react';

interface ErrorBoundaryState {
  hasError: boolean;
  error?: Error;
}

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode;
}

export default class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return {
      hasError: true,
      error
    };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('Commit detail error:', error, errorInfo);
  }

  handleReset = () => {
    this.setState({ hasError: false, error: undefined });
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div className="error-boundary">
          <div className="error-boundary-content">
            <h3>Something went wrong</h3>
            <p>Failed to load commit details. This might be due to:</p>
            <ul>
              <li>Very large commit that exceeds display limits</li>
              <li>Corrupted commit data</li>
              <li>Git repository access issues</li>
              <li>Syntax highlighting parsing error</li>
            </ul>
            
            <div className="error-boundary-actions">
              <button 
                onClick={this.handleReset}
                className="error-boundary-retry"
              >
                Try Again
              </button>
              <button 
                onClick={() => window.history.back()}
                className="error-boundary-back"
              >
                Go Back
              </button>
            </div>
            
            {process.env.NODE_ENV === 'development' && this.state.error && (
              <details className="error-boundary-details">
                <summary>Error Details (Development)</summary>
                <pre className="error-boundary-stack">
                  {this.state.error.stack}
                </pre>
              </details>
            )}
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}