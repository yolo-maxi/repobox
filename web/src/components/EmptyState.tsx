'use client';

import React from 'react';
import Link from 'next/link';

interface EmptyStateProps {
  illustration: React.ComponentType<{ className?: string }>;
  title: string;
  description?: string;
  action?: {
    label: string;
    href?: string;
    onClick?: () => void;
  };
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

export default function EmptyState({
  illustration: Illustration,
  title,
  description,
  action,
  size = 'md',
  className = ''
}: EmptyStateProps) {
  const sizeClasses = {
    sm: 'empty-state-sm',
    md: 'empty-state-md', 
    lg: 'empty-state-lg'
  };

  const ActionComponent = action ? (
    action.href ? (
      <Link href={action.href} className="empty-state-action">
        {action.label}
      </Link>
    ) : (
      <button onClick={action.onClick} className="empty-state-action">
        {action.label}
      </button>
    )
  ) : null;

  return (
    <div className={`empty-state ${sizeClasses[size]} ${className}`}>
      <div className="empty-state-illustration">
        <Illustration className="empty-state-icon" />
      </div>
      <div className="empty-state-content">
        <h3 className="empty-state-title">{title}</h3>
        {description && (
          <p className="empty-state-description">{description}</p>
        )}
        {ActionComponent}
      </div>
    </div>
  );
}