import Link from 'next/link';

interface ServiceCheck {
  name: string;
  url: string;
  status: 'online' | 'offline' | 'slow';
  responseTime: number | null;
  error?: string;
}

async function checkService(url: string, name: string): Promise<ServiceCheck> {
  const startTime = Date.now();
  
  try {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 3000); // 3s timeout
    
    const response = await fetch(url, {
      signal: controller.signal,
      headers: {
        'User-Agent': 'repo.box-status-checker',
      },
    });
    
    clearTimeout(timeout);
    const responseTime = Date.now() - startTime;
    
    if (response.ok) {
      return {
        name,
        url,
        status: responseTime > 2000 ? 'slow' : 'online',
        responseTime,
      };
    } else {
      return {
        name,
        url,
        status: 'offline',
        responseTime,
        error: `HTTP ${response.status}`,
      };
    }
  } catch (error: any) {
    const responseTime = Date.now() - startTime;
    return {
      name,
      url,
      status: 'offline',
      responseTime,
      error: error.name === 'AbortError' ? 'Timeout' : 'Connection failed',
    };
  }
}

async function getServiceStatuses(): Promise<ServiceCheck[]> {
  const services = [
    { name: 'repo.box', url: 'https://repo.box' },
    { name: 'SUPStrategy', url: 'https://supstrategy.repo.box' },
    { name: 'TradeStrategy (Railway)', url: 'https://tradestrategy-production.up.railway.app/api/game-state' },
    { name: 'Agentation', url: 'https://agentation.repo.box' },
    { name: 'Oceangram', url: 'https://oceangram.repo.box' },
  ];

  // Check all services in parallel
  const checks = await Promise.all(
    services.map(service => checkService(service.url, service.name))
  );

  return checks;
}

function getStatusIcon(status: ServiceCheck['status']): string {
  switch (status) {
    case 'online': return '🟢';
    case 'slow': return '🟡';
    case 'offline': return '🔴';
    default: return '⚪';
  }
}

function getOverallStatus(services: ServiceCheck[]): { text: string; degraded: boolean } {
  const offlineCount = services.filter(s => s.status === 'offline').length;
  const slowCount = services.filter(s => s.status === 'slow').length;
  
  if (offlineCount > 0) {
    return {
      text: `${offlineCount} service${offlineCount === 1 ? '' : 's'} offline`,
      degraded: true,
    };
  } else if (slowCount > 0) {
    return {
      text: `${slowCount} service${slowCount === 1 ? '' : 's'} slow`,
      degraded: true,
    };
  } else {
    return {
      text: 'All systems operational',
      degraded: false,
    };
  }
}

export default async function StatusPage() {
  const services = await getServiceStatuses();
  const overallStatus = getOverallStatus(services);
  const lastChecked = new Date().toISOString();

  return (
    <>
      <head>
        <meta httpEquiv="refresh" content="60" />
        <title>Service Status - repo.box</title>
      </head>
      
      <div style={{
        maxWidth: 800,
        margin: '0 auto',
        padding: '60px 40px',
        fontFamily: 'var(--font-mono), "JetBrains Mono", monospace',
      }}>
        {/* Header */}
        <header style={{ marginBottom: 40 }}>
          <Link href="/" style={{ 
            color: 'var(--bp-accent)', 
            textDecoration: 'none',
            fontSize: '0.9rem',
            marginBottom: 20,
            display: 'inline-block'
          }}>
            ← repo.box
          </Link>
          
          <h1 style={{ 
            fontSize: '2rem', 
            color: 'var(--bp-heading)', 
            margin: 0,
            marginBottom: 12,
          }}>
            Service Status
          </h1>
          
          {/* Overall Status */}
          <div style={{
            display: 'flex',
            alignItems: 'center',
            gap: 12,
            marginBottom: 20,
          }}>
            <span style={{
              fontSize: '1.2rem',
              color: overallStatus.degraded ? '#f85149' : '#4ade80',
              fontWeight: 600,
            }}>
              {overallStatus.text}
            </span>
          </div>
          
          <p style={{ 
            color: 'var(--bp-dim)', 
            fontSize: '0.9rem',
            margin: 0,
          }}>
            Live health checks for repo.box services. Auto-refresh every 60 seconds.
          </p>
        </header>

        {/* Services List */}
        <main>
          <div style={{
            background: 'var(--bp-surface)',
            border: '1px solid var(--bp-border)',
            borderRadius: 8,
            overflow: 'hidden',
          }}>
            {services.map((service, index) => (
              <div
                key={service.name}
                style={{
                  padding: '16px 20px',
                  borderBottom: index < services.length - 1 ? '1px solid var(--bp-border)' : 'none',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
                  <span style={{ fontSize: '1.2rem' }}>
                    {getStatusIcon(service.status)}
                  </span>
                  <div>
                    <div style={{
                      fontSize: '1rem',
                      fontWeight: 600,
                      color: 'var(--bp-heading)',
                      marginBottom: 2,
                    }}>
                      {service.name}
                    </div>
                    <div style={{
                      fontSize: '0.8rem',
                      color: 'var(--bp-dim)',
                      fontFamily: 'var(--font-mono), monospace',
                    }}>
                      {service.url}
                    </div>
                  </div>
                </div>
                
                <div style={{ textAlign: 'right' }}>
                  <div style={{
                    fontSize: '0.9rem',
                    fontWeight: 500,
                    color: service.status === 'offline' ? '#f85149' : 
                           service.status === 'slow' ? '#f59e0b' : '#4ade80',
                    marginBottom: 2,
                  }}>
                    {service.status === 'offline' ? 'Offline' : 
                     service.status === 'slow' ? 'Slow' : 'Online'}
                  </div>
                  {service.responseTime !== null && (
                    <div style={{
                      fontSize: '0.8rem',
                      color: 'var(--bp-dim)',
                    }}>
                      {service.responseTime}ms
                    </div>
                  )}
                  {service.error && (
                    <div style={{
                      fontSize: '0.8rem',
                      color: '#f85149',
                    }}>
                      {service.error}
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        </main>

        {/* Footer */}
        <footer style={{
          marginTop: 32,
          padding: '16px 0',
          borderTop: '1px solid var(--bp-border)',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          fontSize: '0.8rem',
          color: 'var(--bp-dim)',
        }}>
          <span>
            Last checked: {new Date(lastChecked).toLocaleString()}
          </span>
          <span>
            Auto-refresh: 60s
          </span>
        </footer>
      </div>
    </>
  );
}

export const revalidate = 0; // Always fresh data