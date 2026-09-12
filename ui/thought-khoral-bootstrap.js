const clientId = 'thought-khoral-workspace';
const keycloakOrigin = 'http://localhost:8081';
const realm = 'thought-khoral';
const authorizationEndpoint = `${keycloakOrigin}/realms/${realm}/protocol/openid-connect/auth`;
const tokenEndpoint = `${keycloakOrigin}/realms/${realm}/protocol/openid-connect/token`;
const sessionKey = 'thought-khoral.oidc.tokens';
const transactionKey = 'thought-khoral.oidc.transaction';

function encodeBase64Url(bytes) {
  return btoa(String.fromCharCode(...bytes))
    .replaceAll('+', '-')
    .replaceAll('/', '_')
    .replaceAll('=', '');
}

function randomValue() {
  return encodeBase64Url(crypto.getRandomValues(new Uint8Array(32)));
}

function readJwtPayload(token) {
  try {
    const payload = token.split('.')[1].replaceAll('-', '+').replaceAll('_', '/');
    return JSON.parse(atob(payload));
  } catch {
    return {};
  }
}

function readTokens() {
  try {
    return JSON.parse(sessionStorage.getItem(sessionKey) ?? 'null');
  } catch {
    return null;
  }
}

function storeTokens(response) {
  const claims = readJwtPayload(response.access_token);
  const tokens = {
    accessToken: response.access_token,
    refreshToken: response.refresh_token,
    expiresAt: Number(claims.exp ?? 0),
    role: claims.n2n_role === 'agent' ? 'agent' : 'human',
  };
  sessionStorage.setItem(sessionKey, JSON.stringify(tokens));
  return tokens;
}

async function tokenRequest(parameters) {
  const response = await fetch(tokenEndpoint, {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({ client_id: clientId, ...parameters }),
  });
  if (!response.ok) {
    throw new Error(`OIDC token request failed (${response.status})`);
  }
  return storeTokens(await response.json());
}

async function finishAuthorizationCallback() {
  const url = new URL(location.href);
  const code = url.searchParams.get('code');
  if (!code) return;

  const transaction = JSON.parse(sessionStorage.getItem(transactionKey) ?? 'null');
  if (!transaction || url.searchParams.get('state') !== transaction.state) {
    throw new Error('OIDC callback state did not match');
  }
  await tokenRequest({
    grant_type: 'authorization_code',
    code,
    code_verifier: transaction.verifier,
    redirect_uri: transaction.redirectUri,
  });
  sessionStorage.removeItem(transactionKey);
  url.searchParams.delete('code');
  url.searchParams.delete('iss');
  url.searchParams.delete('session_state');
  url.searchParams.delete('state');
  history.replaceState({}, '', url);
}

async function beginAuthorization() {
  const verifier = randomValue();
  const digest = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(verifier));
  const state = randomValue();
  const redirectUri = `${location.origin}${location.pathname}${location.search}`;
  sessionStorage.setItem(transactionKey, JSON.stringify({ redirectUri, state, verifier }));
  const url = new URL(authorizationEndpoint);
  url.search = new URLSearchParams({
    client_id: clientId,
    code_challenge: encodeBase64Url(new Uint8Array(digest)),
    code_challenge_method: 'S256',
    redirect_uri: redirectUri,
    response_type: 'code',
    scope: 'openid profile',
    state,
  });
  location.assign(url);
  return new Promise(() => {});
}

async function getAccessToken() {
  const tokens = readTokens();
  if (!tokens) return beginAuthorization();
  if (tokens.expiresAt > Date.now() / 1000 + 30) return tokens.accessToken;
  if (!tokens.refreshToken) return beginAuthorization();
  try {
    const refreshed = await tokenRequest({
      grant_type: 'refresh_token',
      refresh_token: tokens.refreshToken,
    });
    return refreshed.accessToken;
  } catch {
    sessionStorage.removeItem(sessionKey);
    return beginAuthorization();
  }
}

class AuthenticatedSocket extends EventTarget {
  constructor(url, accessToken) {
    super();
    this.readyState = WebSocket.CONNECTING;
    this.socket = new WebSocket(url);
    const requestId = crypto.randomUUID();
    this.socket.addEventListener('open', () => {
      this.socket.send(JSON.stringify({
        jsonrpc: '2.0',
        id: requestId,
        method: 'session.authenticate',
        params: { accessToken },
      }));
    });
    this.socket.addEventListener('message', (event) => {
      if (this.readyState === WebSocket.CONNECTING) {
        try {
          const response = JSON.parse(event.data);
          if (response.id === requestId && response.result?.actor) {
            this.readyState = WebSocket.OPEN;
            this.dispatchEvent(new Event('open'));
            return;
          }
        } catch {}
        this.dispatchEvent(new Event('error'));
        this.socket.close();
        return;
      }
      this.dispatchEvent(new MessageEvent('message', { data: event.data }));
    });
    this.socket.addEventListener('error', () => this.dispatchEvent(new Event('error')));
    this.socket.addEventListener('close', () => {
      this.readyState = WebSocket.CLOSED;
      this.dispatchEvent(new Event('close'));
    });
  }

  send(data) {
    if (this.readyState !== WebSocket.OPEN) throw new DOMException('Socket is not open');
    this.socket.send(data);
  }

  close() {
    this.socket.close();
  }
}

await finishAuthorizationCallback();
const tokens = readTokens();
const pageUrl = new URL(location.href);
window.thoughtKhoralWorkspace = {
  roomId: pageUrl.searchParams.get('room') ?? '10000000-0000-4000-8000-000000000001',
  participantRole: tokens?.role ?? 'human',
  getAccessToken,
  createSocket: (url, accessToken) => new AuthenticatedSocket(url, accessToken),
};

const appModule = document.querySelector('meta[name="thought-khoral-app-module"]')?.content;
if (!appModule) throw new Error('ThoughtKhoral application module is unavailable');
await import(appModule);
