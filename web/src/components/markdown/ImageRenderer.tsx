'use client';

import React, { useState } from 'react';
import Image from 'next/image';

interface ImageRendererProps {
  src?: string | Blob;
  alt?: string;
  title?: string;
}

export default function ImageRenderer({ src, alt = '', title }: ImageRendererProps) {
  const [isZoomed, setIsZoomed] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [hasError, setHasError] = useState(false);

  if (!src || typeof src !== 'string') {
    return (
      <div className="md-image-error">
        <span>❌ Missing or invalid image source</span>
      </div>
    );
  }

  const handleImageClick = () => {
    if (!hasError) {
      setIsZoomed(true);
    }
  };

  const handleCloseZoom = () => {
    setIsZoomed(false);
  };

  if (hasError) {
    return (
      <div className="md-image-error">
        <span>❌ Failed to load image: {alt || src}</span>
      </div>
    );
  }

  return (
    <>
      <div className="md-image-container">
        {isLoading && (
          <div className="md-image-skeleton">
            <div className="md-image-skeleton-content">Loading...</div>
          </div>
        )}
        <img
          src={src}
          alt={alt}
          title={title}
          className={`md-image ${isLoading ? 'md-image-loading' : ''}`}
          onClick={handleImageClick}
          onLoad={() => setIsLoading(false)}
          onError={() => {
            setIsLoading(false);
            setHasError(true);
          }}
          style={{ cursor: hasError ? 'default' : 'pointer' }}
        />
        {alt && !isLoading && (
          <div className="md-image-caption">{alt}</div>
        )}
      </div>

      {/* Zoom Modal */}
      {isZoomed && (
        <div className="md-image-zoom-overlay" onClick={handleCloseZoom}>
          <div className="md-image-zoom-container">
            <button 
              className="md-image-zoom-close"
              onClick={handleCloseZoom}
              aria-label="Close zoomed image"
            >
              ✕
            </button>
            <img
              src={src}
              alt={alt}
              className="md-image-zoomed"
              onClick={(e) => e.stopPropagation()}
            />
            {alt && (
              <div className="md-image-zoom-caption">{alt}</div>
            )}
          </div>
        </div>
      )}
    </>
  );
}