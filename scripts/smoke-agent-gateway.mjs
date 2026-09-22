import { createHash, randomBytes, randomUUID } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { createConnection } from 'node:net';

const uiOrigin = process.env.THOUGHT_KHORAL_SMOKE_UI_ORIGIN ?? 'http://localhost:8082';
const keycloakOrigin =
  process.env.THOUGHT_KHORAL_SMOKE_KEYCLOAK_ORIGIN ?? 'http://localhost:8081';
const gatewayUrl =
  process.env.THOUGHT_KHORAL_SMOKE_GATEWAY_URL ?? 'ws://localhost:8080/ws';
const platformDir = new URL('..', import.meta.url).pathname;
const composeFile = `${platformDir}/compose.yaml`;
const realm = process.env.THOUGHT_KHORAL_SMOKE_REALM ?? 'thought-khoral';
const workspaceClientId =
  process.env.THOUGHT_KHORAL_SMOKE_CLIENT_ID ?? 'thought-khoral-workspace';
const workloadClientId = 'thought-khoral-agent-gateway';
const workloadClientSecret =
  process.env.THOUGHT_KHORAL_SMOKE_AGENT_GATEWAY_SECRET ?? 'agent-gateway-client-dev-only';
const timeoutMs = Number(process.env.THOUGHT_KHORAL_SMOKE_AGENT_TIMEOUT_MS ?? 30_000);
const terminalObservationMs = 2_000;
const referenceAgentId = '74686f75-6768-746b-686f-72616c000003';
const hiddenText = 'hidden targeted smoke packet marker';

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

function delay(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
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

async function authenticateWithPkce(username, password) {
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
    client_id: workspaceClientId,
    code_challenge: challenge,
    code_challenge_method: 'S256',
    redirect_uri: redirectUri,
    response_type: 'code',
    scope: 'openid profile',
    state,
  });

  const authorization = await fetchWithCookies(jar, authorizationUrl);
  if (authorization.status !== 200) fail(`authorization endpoint returned ${authorization.status}`);
  const loginPage = await authorization.text();
  const form = loginPage.match(/<form\b[^>]*\bid="kc-form-login"[^>]*>/)?.[0];
  const action = form?.match(/\baction="([^"]+)"/)?.[1];
  if (!action) fail('Keycloak login form action was not present');

  const login = await fetchWithCookies(jar, decodeHtmlAttribute(action), {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({ credentialId: '', password, username }),
  });
  if (![302, 303].includes(login.status)) fail(`${username} login returned ${login.status}`);
  const callbackLocation = login.headers.get('location');
  if (!callbackLocation) fail(`${username} login omitted its callback`);
  const callback = new URL(callbackLocation);
  if (callback.origin !== uiOrigin || callback.pathname !== '/') {
    fail(`${username} login returned an unexpected callback origin`);
  }
  if (callback.searchParams.get('state') !== state) fail(`${username} OIDC callback state mismatch`);
  const code = callback.searchParams.get('code');
  if (!code) fail(`${username} OIDC callback did not contain an authorization code`);

  const tokenResponse = await fetch(
    new URL(`/realms/${realm}/protocol/openid-connect/token`, keycloakOrigin),
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams({
        client_id: workspaceClientId,
        code,
        code_verifier: verifier,
        grant_type: 'authorization_code',
        redirect_uri: redirectUri,
      }),
    },
  );
  if (!tokenResponse.ok) fail(`${username} OIDC token exchange returned ${tokenResponse.status}`);
  const tokens = await tokenResponse.json();
  if (typeof tokens.access_token !== 'string' || tokens.access_token.length === 0) {
    fail(`${username} token exchange did not return an access token`);
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

  receiveJson(waitMillis = timeoutMs) {
    if (this.messages.length > 0) return Promise.resolve(this.messages.shift());
    return new Promise((resolve, reject) => {
      let timer;
      const waiter = {
        reject: (error) => {
          clearTimeout(timer);
          reject(error);
        },
        resolve: (message) => {
          clearTimeout(timer);
          resolve(message);
        },
      };
      timer = setTimeout(() => {
        const index = this.waiters.indexOf(waiter);
        if (index >= 0) this.waiters.splice(index, 1);
        const error = new Error(`WebSocket response timed out after ${waitMillis}ms`);
        error.code = 'THOUGHT_KHORAL_SMOKE_TIMEOUT';
        reject(error);
      }, waitMillis);
      this.waiters.push(waiter);
    });
  }

  close() {
    this.socket.end(encodeClientFrame('', 0x8));
  }
}

async function assertNoPostTerminalLifecycle(socket, taskId, skillId) {
  const deadline = Date.now() + terminalObservationMs;
  while (Date.now() < deadline) {
    try {
      const event = await socket.receiveJson(Math.max(1, deadline - Date.now()));
      if (event.payload?.taskId === taskId) {
        fail(`${skillId} emitted a post-terminal lifecycle event: ${event.eventType}`);
      }
    } catch (error) {
      if (error?.code === 'THOUGHT_KHORAL_SMOKE_TIMEOUT') return;
      throw error;
    }
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

async function authenticateSocket(token) {
  const socket = await connectBrowserSocket();
  socket.sendJson({
    id: `agent-smoke-auth-${randomUUID()}`,
    jsonrpc: '2.0',
    method: 'session.authenticate',
    params: { accessToken: token },
  });
  const response = await socket.receiveJson();
  const actor = response.result?.actor;
  if (actor?.role !== 'human' || typeof actor.id !== 'string') {
    fail(`session.authenticate did not bind a human fixture: ${JSON.stringify(response)}`);
  }
  return { socket, actor };
}

async function joinRoom(socket, roomId) {
  socket.sendJson({
    id: `agent-smoke-join-${randomUUID()}`,
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
  if (!Array.isArray(joined.result?.events)) {
    fail(`room.join did not return a room replay: ${JSON.stringify(joined)}`);
  }
}

async function receiveUntil(socket, predicate, description) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const message = await socket.receiveJson();
    if (predicate(message)) return message;
  }
  fail(`timed out waiting for ${description}`);
}

function rpcParams(roomId) {
  return {
    contractVersion: 'n2n.room.v1',
    occurredAt: new Date().toISOString(),
    requestId: randomUUID(),
    roomId,
  };
}

async function sendTargetedHiddenMessage(bobSocket, bobId, roomId) {
  const params = rpcParams(roomId);
  params.text = hiddenText;
  params.delivery = 'mentioned';
  params.mentions = [{ id: bobId, token: 'bob', type: 'participant' }];
  bobSocket.sendJson({
    id: `agent-smoke-hidden-${randomUUID()}`,
    jsonrpc: '2.0',
    method: 'chat.send',
    params,
  });
  const hidden = await receiveUntil(
    bobSocket,
    (message) => message.eventType === 'message.created' && message.payload?.text === hiddenText,
    'the hidden targeted message',
  );
  if (!hidden.payload?.audienceIds?.includes(bobId)) {
    fail('targeted hidden message did not retain its only recipient');
  }
  return hidden;
}

async function workloadToken() {
  const credentials = Buffer.from(`${workloadClientId}:${workloadClientSecret}`).toString('base64');
  const response = await fetch(
    new URL(`/realms/${realm}/protocol/openid-connect/token`, keycloakOrigin),
    {
      method: 'POST',
      headers: {
        Authorization: `Basic ${credentials}`,
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({ grant_type: 'client_credentials' }),
    },
  );
  if (!response.ok) fail(`workload token request returned ${response.status}`);
  const body = await response.json();
  if (typeof body.access_token !== 'string' || body.access_token.length === 0) {
    fail('workload token request did not return an access token');
  }
  return body.access_token;
}

async function requireJson(response, description) {
  if (!response.ok) fail(`${description} returned ${response.status}`);
  return response.json();
}

function summaryForPacket(packet) {
  const messageCount = packet.events.filter((event) => event.eventType === 'message.created').length;
  const titles = packet.activeDecisions.length === 0
    ? 'none'
    : packet.activeDecisions.map((decision) => decision.title).join('; ');
  return {
    citations: packet.events.map((event) => event.eventId),
    kind: 'context-summary.v1',
    summary: `Room ${packet.roomId} revision ${packet.contextRevision} has ${messageCount} messages and active decisions: ${titles}.`,
  };
}

function compose(args) {
  execFileSync('podman-compose', ['-f', composeFile, ...args], { stdio: 'inherit' });
}

async function capturePacketWithInternalHarness(aliceSocket, roomId, hiddenEventId) {
  let stopped = false;
  try {
    compose(['stop', 'thought-khoral-agent-gateway']);
    stopped = true;

    const params = rpcParams(roomId);
    params.agentId = referenceAgentId;
    params.skillId = 'summarize-context';
    params.input = 'Capture the authorized packet only.';
    aliceSocket.sendJson({
      id: `agent-smoke-capture-${randomUUID()}`,
      jsonrpc: '2.0',
      method: 'agent.task.start',
      params,
    });
    const requested = await receiveUntil(
      aliceSocket,
      (message) =>
        message.eventType === 'agent.task.requested' &&
        message.payload?.skillId === 'summarize-context',
      'captured packet task request',
    );
    const taskId = requested.payload?.taskId;
    if (typeof taskId !== 'string') fail('captured packet task omitted taskId');

    const token = await workloadToken();
    const claim = await requireJson(
      await fetch(new URL('/internal/v1/agent-tasks/claim', 'http://localhost:8080'), {
        method: 'POST',
        headers: {
          Authorization: `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ leaseOwner: randomUUID() }),
      }),
      'packet capture claim',
    );
    const packet = claim.packet;
    if (packet?.taskId !== taskId || typeof claim.leaseToken !== 'string') {
      fail('packet capture claim did not return the expected live task lease');
    }
    const serializedPacket = JSON.stringify(packet);
    const capturedIds = new Set(packet.events.map((event) => event.eventId));
    if (serializedPacket.includes(hiddenText) || capturedIds.has(hiddenEventId)) {
      fail('hidden targeted message escaped into the captured authorized packet');
    }

    await requireJson(
      await fetch(new URL(`/internal/v1/agent-tasks/${taskId}/updates`, 'http://localhost:8080'), {
        method: 'POST',
        headers: {
          Authorization: `Bearer ${token}`,
          'Content-Type': 'application/json',
          'X-Thought-Khoral-Lease-Token': claim.leaseToken,
        },
        body: JSON.stringify({
          contextRevision: packet.contextRevision,
          update: {
            eventType: 'agent.task.succeeded',
            occurredAt: new Date().toISOString(),
            payload: { result: summaryForPacket(packet) },
          },
          updateId: randomUUID(),
        }),
      }),
      'packet capture terminal update',
    );
    console.log('agent-smoke: hidden targeted message is absent from the captured packet harness');
  } finally {
    if (stopped) {
      compose(['up', '-d', 'thought-khoral-agent-gateway']);
      await delay(1_000);
    }
  }
}

async function invokeSkill(aliceSocket, roomId, skillId, input, hiddenEventId) {
  const params = rpcParams(roomId);
  params.agentId = referenceAgentId;
  params.skillId = skillId;
  params.input = input;
  aliceSocket.sendJson({
    id: `agent-smoke-${skillId}-${randomUUID()}`,
    jsonrpc: '2.0',
    method: 'agent.task.start',
    params,
  });

  const requested = await receiveUntil(
    aliceSocket,
    (message) =>
      message.eventType === 'agent.task.requested' && message.payload?.skillId === skillId,
    `${skillId} task request`,
  );
  const taskId = requested.payload?.taskId;
  if (typeof taskId !== 'string') fail(`${skillId} requested event omitted taskId`);

  const taskEvents = [];
  while (!taskEvents.some((event) => event.eventType === 'agent.task.succeeded')) {
    const event = await receiveUntil(
      aliceSocket,
      (message) => message.payload?.taskId === taskId,
      `${skillId} lifecycle event`,
    );
    taskEvents.push(event);
    if (event.eventType === 'agent.task.failed') {
      fail(`${skillId} emitted a terminal failure: ${JSON.stringify(event.payload)}`);
    }
  }

  const eventTypes = taskEvents.map((event) => event.eventType);
  const expectedEventTypes = [
    'agent.task.progressed',
    'agent.task.progressed',
    'agent.task.progressed',
    'agent.task.succeeded',
  ];
  if (JSON.stringify(eventTypes) !== JSON.stringify(expectedEventTypes)) {
    fail(`${skillId} durable event order differed: ${JSON.stringify(eventTypes)}`);
  }
  const progress = taskEvents.slice(0, 3).map((event) => event.payload);
  const expectedProgress = [
    ['accepted', 'Task submitted'],
    ['working', 'Reading authorized room context'],
    ['working', 'Preparing cited result'],
  ];
  for (let index = 0; index < expectedProgress.length; index += 1) {
    if (
      progress[index].phase !== expectedProgress[index][0] ||
      progress[index].text !== expectedProgress[index][1]
    ) {
      fail(`${skillId} progress ${index} differed from the fixed A2A lifecycle`);
    }
  }
  const terminal = taskEvents.at(-1);
  const citations = terminal.payload?.result?.citations;
  if (!Array.isArray(citations) || citations.length !== new Set(citations).size || citations.length === 0) {
    fail(`${skillId} terminal result did not contain one non-empty unique citation set`);
  }
  if (citations.includes(hiddenEventId)) {
    fail(`${skillId} cited the hidden targeted message`);
  }
  if (skillId === 'extract-action-items') {
    const actionItems = terminal.payload?.result?.actionItems;
    if (!Array.isArray(actionItems) || actionItems.length !== 1 || citations.length !== 1) {
      fail('extract-action-items did not return exactly the requested cited action item');
    }
  }
  if (taskEvents.filter((event) => event.eventType === 'agent.task.succeeded').length !== 1) {
    fail(`${skillId} emitted more than one terminal result`);
  }
  await assertNoPostTerminalLifecycle(aliceSocket, taskId, skillId);
  return taskId;
}

try {
  const [aliceToken, bobToken] = await Promise.all([
    authenticateWithPkce('alice', 'alice-dev-only'),
    authenticateWithPkce('bob', 'bob-dev-only'),
  ]);
  const [{ socket: aliceSocket }, { socket: bobSocket, actor: bobActor }] = await Promise.all([
    authenticateSocket(aliceToken),
    authenticateSocket(bobToken),
  ]);
  try {
    const roomId = randomUUID();
    await joinRoom(aliceSocket, roomId);
    await joinRoom(bobSocket, roomId);
    const hidden = await sendTargetedHiddenMessage(bobSocket, bobActor.id, roomId);
    await capturePacketWithInternalHarness(aliceSocket, roomId, hidden.eventId);
    await invokeSkill(
      aliceSocket,
      roomId,
      'summarize-context',
      'Summarize the authorized room context.',
      hidden.eventId,
    );
    await invokeSkill(
      aliceSocket,
      roomId,
      'extract-action-items',
      '- Prepare A2A verification | owner: Alice | due: Friday',
      hidden.eventId,
    );
    console.log('agent-smoke: both human-invoked A2A skills emitted ordered progress and one cited result');
  } finally {
    aliceSocket.close();
    bobSocket.close();
  }
} catch (error) {
  console.error(`agent-smoke: ${error.message}`);
  process.exitCode = 1;
}
