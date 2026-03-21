const TOKEN = process.env.DASHBOARD_TOKEN || ''

export function verifyAuth(token: string): boolean {
  return TOKEN.length > 0 && token === TOKEN
}

export function getMagicLink(baseUrl: string = 'https://repo.box'): string {
  return `${baseUrl}/dashboard?token=${TOKEN}`
}
