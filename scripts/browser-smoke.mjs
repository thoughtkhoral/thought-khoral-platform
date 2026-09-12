import { createHash, randomBytes, randomUUID } from 'node:crypto';
import { createConnection } from 'node:net';

const uiOrigin = process.env.THOUGHT_KHORAL_SMOKE_UI_ORIGIN ?? 'http://localhost:8082';
const keycloakOrigin =
  process.env.THOUGHT_KHORAL_SMOKE_KEYCLOAK_ORIGIN ?? 'http://localhost:8081';
const gatewayUrl =
  process.env.THOUGHT_KHORAL_SMOKE_GATEWAY_URL ?? 'ws://localhost:8080/ws';
const realm = process.env.THOUGHT_KHORAL_SMOKE_REALM ?? 'thought-khoral';
const clientId =
  process.env.THOUGHT_KHORAL_SMOKE_CLIENT_ID ?? 'thought-khoral-workspace';
const username = process.env.THOUGHT_KHORAL_SMOKE_USERNAME ?? 'alice';
const password = process.env.THOUGHT_KHORAL_SMOKE_PASSWORD ?? 'alice-dev-only';
const roomId =
  process.env.THOUGHT_KHORAL_SMOKE_ROOM_ID ??
  '10000000-0000-4000-8000-000000000001';
const timeoutMs = Number(process.env.THOUGHT_KHORAL_SMOKE_BROWSER_TIMEOUT_MS ?? 15_000);

function fail(message) {
  throw new Error(message);
}

function base64Url(value) {
  return value.toString('base64').replaceAll('+', '-').replaceAll('/', '_').replaceAll('=', '');
}

function decodeHtmlAttribute(value) {
  return value
    .replaceAll('&amp;', '&')
    .replaceAll('&quot;', '"')
    .replaceAll('&#39;', "'")
    .replaceAll('&#x3D;', '=');
}

class CookieJar {
  constructor() {
    this.cookies = new Map();
  }

  capture(response) {
    for (const header of response.headers.getSetCookie()) {
      const pair = header.split(';', 1)[0];
      const separator = pair.indexOf('=');
      if (separator > 0) this.cookies.set(pair.slice(0, separator), pair.slice(separator + 1));
    }
  }

  header() {
    return [...this.cookies].map(([name, value]) => `${name}=${value}`).join('; ');
  }
}

async function fetchWithCookies(jar, url, options = {}) {
  const headers = new Headers(options.headers);
  if (jar.cookies.size > 0) headers.set('Cookie', jar.header());
  const response = await fetch(url, { ...options, headers, redirect: 'manual' });
  jar.capture(response);
  return response;
}

async function authenticateWithPkce() {
  const jar = new CookieJar();
  const verifier = base64Url(randomBytes(32));
  const challenge = base64Url(createHash('sha256').update(verifier).digest());
  const state = base64Url(randomBytes(32));
  const redirectUri = `${uiOrigin}/`;
  const authorizationUrl = new URL(
    `/realms/${realm}/protocol/openid-connect/auth`,
    keycloakOrigin,
  );
  authorizationUrl.search = new URLSearchParams({
    client_id: clientId,
    code_challenge: challenge,
    code_challenge_method: 'S256',
    redirect_uri: redirectUri,
    response_type: 'code',
    scope: 'openid profile',
    state,
  });

  const authorization = await fetchWithCookies(jar, authorizationUrl);
  if (authorization.status !== 200) {
    fail(`authorization endpoint returned ${authorization.status}`);
  }
  const loginPage = await authorization.text();
  const form = loginPage.match(/<form\b[^>]*\bid="kc-form-login"[^>]*>/)?.[0];
  const action = form?.match(/\baction="([^"]+)"/)?.[1];
  if (!action) fail('Keycloak login form action was not present');

  const login = await fetchWithCookies(jar, decodeHtmlAttribute(action), {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({ credentialId: '', password, username }),
  });
  if (![302, 303].includes(login.status)) {
    fail(`development fixture login returned ${login.status}`);
  }
  const callbackLocation = login.headers.get('location');
  if (!callbackLocation) fail('development fixture login omitted its callback');
  const callback = new URL(callbackLocation);
  if (callback.origin !== uiOrigin || callback.pathname !== '/') {
    fail(`login returned an unexpected callback origin: ${callback.origin}${callback.pathname}`);
  }
  if (callback.searchParams.get('state') !== state) fail('OIDC callback state did not match');
  const code = callback.searchParams.get('code');
  if (!code) fail('OIDC callback did not contain an authorization code');

  const callbackPage = await fetch(callback);
  if (!callbackPage.ok) fail(`authenticated UI callback returned ${callbackPage.status}`);
  const html = await callbackPage.text();
  if (!/<title>[\s]*ThoughtKhoral workspace[\s]*<\/title>/.test(html)) {
    fail('authenticated UI callback did not return the ThoughtKhoral workspace title');
  }

  const tokenEndpoint = new URL(
    `/realms/${realm}/protocol/openid-connect/token`,
    keycloakOrigin,
  );
  const tokenResponse = await fetch(tokenEndpoint, {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({
      client_id: clientId,
      code,
      code_verifier: verifier,
      grant_type: 'authorization_code',
      redirect_uri: redirectUri,
    }),
  });
  if (!tokenResponse.ok) fail(`OIDC token exchange returned ${tokenResponse.status}`);
  const tokens = await tokenResponse.json();
  if (typeof tokens.access_token !== 'string' || tokens.access_token.length === 0) {
    fail('OIDC token exchange did not return an access token');
  }
  return tokens.access_token;
}

function encodeClientFrame(payload, opcode = 0x1) {
  const content = Buffer.from(payload);
  const mask = randomBytes(4);
  let header;
  if (content.length < 126) {
    header = Buffer.from([0x80 | opcode, 0x80 | content.length]);
  } else if (content.length <= 0xffff) {
    header = Buffer.alloc(4);
    header[0] = 0x80 | opcode;
    header[1] = 0x80 | 126;
    header.writeUInt16BE(content.length, 2);
  } else {
    header = Buffer.alloc(10);
    header[0] = 0x80 | opcode;
    header[1] = 0x80 | 127;
    header.writeBigUInt64BE(BigInt(content.length), 2);
  }
  const masked = Buffer.alloc(content.length);
  for (let index = 0; index < content.length; index += 1) {
    masked[index] = content[index] ^ mask[index % 4];
  }
  return Buffer.concat([header, mask, masked]);
}

class WebSocketConnection {
  constructor(socket, initialData) {
    this.socket = socket;
    this.buffer = initialData;
    this.messages = [];
    this.waiters = [];
    socket.on('data', (data) => {
      this.buffer = Buffer.concat([this.buffer, data]);
      this.parseFrames();
    });
    socket.on('error', (error) => this.rejectWaiters(error));
    socket.on('close', () => this.rejectWaiters(new Error('WebSocket closed unexpectedly')));
    this.parseFrames();
  }

  rejectWaiters(error) {
    while (this.waiters.length > 0) this.waiters.shift().reject(error);
  }

  parseFrames() {
    while (this.buffer.length >= 2) {
      const first = this.buffer[0];
      const second = this.buffer[1];
      const opcode = first & 0x0f;
      const masked = (second & 0x80) !== 0;
      let length = second & 0x7f;
      let offset = 2;
      if (length === 126) {
        if (this.buffer.length < 4) return;
        length = this.buffer.readUInt16BE(2);
        offset = 4;
      } else if (length === 127) {
        if (this.buffer.length < 10) return;
        const largeLength = this.buffer.readBigUInt64BE(2);
        if (largeLength > BigInt(Number.MAX_SAFE_INTEGER)) fail('WebSocket frame is too large');
        length = Number(largeLength);
        offset = 10;
      }
      const maskLength = masked ? 4 : 0;
      if (this.buffer.length < offset + maskLength + length) return;
      const mask = masked ? this.buffer.subarray(offset, offset + 4) : null;
      offset += maskLength;
      const payload = Buffer.from(this.buffer.subarray(offset, offset + length));
      this.buffer = this.buffer.subarray(offset + length);
      if (mask) {
        for (let index = 0; index < payload.length; index += 1) {
          payload[index] ^= mask[index % 4];
        }
      }
      if (opcode === 0x1) this.deliver(JSON.parse(payload.toString('utf8')));
      if (opcode === 0x8) this.rejectWaiters(new Error('WebSocket sent a close frame'));
      if (opcode === 0x9) this.socket.write(encodeClientFrame(payload, 0xa));
    }
  }

  deliver(message) {
    const waiter = this.waiters.shift();
    if (waiter) waiter.resolve(message);
    else this.messages.push(message);
  }

  sendJson(message) {
    this.socket.write(encodeClientFrame(JSON.stringify(message)));
  }

  receiveJson() {
    if (this.messages.length > 0) return Promise.resolve(this.messages.shift());
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        const index = this.waiters.findIndex((waiter) => waiter.resolve === resolve);
        if (index >= 0) this.waiters.splice(index, 1);
        reject(new Error(`WebSocket response timed out after ${timeoutMs}ms`));
      }, timeoutMs);
      this.waiters.push({
        reject,
        resolve: (message) => {
          clearTimeout(timer);
          resolve(message);
        },
      });
    });
  }

  close() {
    this.socket.end(encodeClientFrame('', 0x8));
  }
}

async function connectBrowserSocket() {
  const url = new URL(gatewayUrl);
  if (url.protocol !== 'ws:') fail(`unsupported WebSocket protocol: ${url.protocol}`);
  const key = randomBytes(16).toString('base64');
  const expectedAccept = createHash('sha1')
    .update(`${key}258EAFA5-E914-47DA-95CA-C5AB0DC85B11`)
    .digest('base64');

  return new Promise((resolve, reject) => {
    const socket = createConnection({ host: url.hostname, port: Number(url.port || 80) });
    const timer = setTimeout(() => {
      socket.destroy();
      reject(new Error(`WebSocket handshake timed out after ${timeoutMs}ms`));
    }, timeoutMs);
    let response = Buffer.alloc(0);
    const failHandshake = (error) => {
      clearTimeout(timer);
      socket.destroy();
      reject(error);
    };
    socket.once('error', failHandshake);
    socket.once('connect', () => {
      socket.write(
        `GET ${url.pathname}${url.search} HTTP/1.1\r\n` +
          `Host: ${url.host}\r\n` +
          'Connection: Upgrade\r\n' +
          'Upgrade: websocket\r\n' +
          `Origin: ${uiOrigin}\r\n` +
          `Sec-WebSocket-Key: ${key}\r\n` +
          'Sec-WebSocket-Version: 13\r\n\r\n',
      );
    });
    const onData = (data) => {
      response = Buffer.concat([response, data]);
      const headerEnd = response.indexOf('\r\n\r\n');
      if (headerEnd < 0) return;
      clearTimeout(timer);
      socket.off('data', onData);
      socket.off('error', failHandshake);
      const headers = response.subarray(0, headerEnd).toString('utf8');
      if (!headers.startsWith('HTTP/1.1 101 ')) {
        failHandshake(new Error(`WebSocket upgrade failed: ${headers.split('\r\n', 1)[0]}`));
        return;
      }
      const accept = headers.match(/^Sec-WebSocket-Accept:\s*(.+)$/im)?.[1]?.trim();
      if (accept !== expectedAccept) {
        failHandshake(new Error('WebSocket server acceptance key did not match'));
        return;
      }
      resolve(new WebSocketConnection(socket, response.subarray(headerEnd + 4)));
    };
    socket.on('data', onData);
  });
}

async function verifyConnectedRoom(accessToken) {
  const socket = await connectBrowserSocket();
  try {
    socket.sendJson({
      id: 'browser-smoke-authenticate',
      jsonrpc: '2.0',
      method: 'session.authenticate',
      params: { accessToken },
    });
    const authenticated = await socket.receiveJson();
    if (
      authenticated.id !== 'browser-smoke-authenticate' ||
      authenticated.result?.actor?.role !== 'human' ||
      typeof authenticated.result?.actor?.id !== 'string'
    ) {
      fail(`session.authenticate did not bind the human fixture: ${JSON.stringify(authenticated)}`);
    }

    socket.sendJson({
      id: 'browser-smoke-join',
      jsonrpc: '2.0',
      method: 'room.join',
      params: {
        afterSequence: 0,
        contractVersion: 'n2n.room.v1',
        occurredAt: new Date().toISOString(),
        requestId: randomUUID(),
        roomId,
      },
    });
    const joined = await socket.receiveJson();
    if (joined.id !== 'browser-smoke-join' || !Array.isArray(joined.result?.events)) {
      fail(`room.join did not return a room replay: ${JSON.stringify(joined)}`);
    }
  } finally {
    socket.close();
  }
}

try {
  const accessToken = await authenticateWithPkce();
  await verifyConnectedRoom(accessToken);
  console.log('browser-smoke: fresh PKCE session authenticated and joined the room');
} catch (error) {
  console.error(`browser-smoke: ${error.message}`);
  process.exitCode = 1;
}
