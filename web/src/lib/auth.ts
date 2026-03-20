const TOKEN = '297b3fb36b7411a091742072e72eaa2ace0fb5d7ba3d3e984ec3254a9ad708ef'

export function verifyAuth(token: string): boolean {
  return token === TOKEN
}

export function getMagicLink(baseUrl: string = 'https://repo.box'): string {
  return `${baseUrl}/dashboard?token=${TOKEN}`
}