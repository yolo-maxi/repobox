import React from 'react';

interface TableRendererProps {
  children: React.ReactNode;
}

export default function TableRenderer({ children }: TableRendererProps) {
  return (
    <div className="md-table-wrapper">
      <table className="md-table">{children}</table>
    </div>
  );
}