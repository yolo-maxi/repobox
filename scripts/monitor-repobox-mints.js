#!/usr/bin/env node

/**
 * repo.box ENS mint monitor
 *
 * Emits JSON:
 * {
 *   checkedWindow: { fromBlock, toBlock, mode },
 *   totalEvents,
 *   externalEvents: [...],
 *   recentEvents: [...]
 * }
 */

const fs = require('fs');
const path = require('path');

const RPC_URL = process.env.BASE_RPC_URL ||
  process.env.RPC_URL ||
  'https://mainnet.base.org';

const MINT_CONTRACT = (
  process.env.REPOBOX_MINT_CONTRACT ||
  '0x09c4D67e3491EeFBe2a51eaBF1E473e3Ee0B8518'
).toLowerCase();

const NAME_MINTED_TOPIC =
  '0x5368c001f99f452c2722c23b03143d4c3794dbbc4fb7bd340e652f228915b8ba';

const DEFAULT_RESERVE_WALLETS = [
  '0xF053A15C36f1FbCC2A281095e6f1507ea1EFc931',
].map((a) => a.toLowerCase());

const extraReserve = (process.env.REPOBOX_RESERVE_WALLETS || '')
  .split(',')
  .map((s) => s.trim().toLowerCase())
  .filter(Boolean);

const RESERVE_WALLETS = new Set([...DEFAULT_RESERVE_WALLETS, ...extraReserve]);

const STATE_DIR = '/home/xiko/repobox/.state';
const STATE_PATH = path.join(STATE_DIR, 'repobox-mint-monitor.json');

const LOOKBACK_BLOCKS = Number(process.env.REPOBOX_MINT_LOOKBACK_BLOCKS || 22000);

async function rpc(method, params = []) {
  const res = await fetch(RPC_URL, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ jsonrpc: '2.0', id: 1, method, params }),
  });

  if (!res.ok) {
    throw new Error(`RPC HTTP ${res.status} ${res.statusText}`);
  }

  const json = await res.json();
  if (json.error) {
    throw new Error(`RPC ${method} error: ${JSON.stringify(json.error)}`);
  }

  return json.result;
}

function hexToInt(hex) {
  if (!hex) return 0;
  return parseInt(hex, 16);
}

function hexToBigInt(hex) {
  if (!hex) return 0n;
  return BigInt(hex);
}

function decodeAddressFromTopic(topic) {
  if (!topic || topic.length < 42) return null;
  return `0x${topic.slice(-40)}`.toLowerCase();
}

function decodeNameMintedData(dataHex) {
  const raw = (dataHex || '').replace(/^0x/, '');
  if (!raw || raw.length < 64 * 3) {
    throw new Error('Malformed NameMinted data payload');
  }

  const readWord = (wordIndex) => raw.slice(wordIndex * 64, (wordIndex + 1) * 64);

  const nameOffsetBytes = Number(BigInt(`0x${readWord(0)}`));
  const priceWei = BigInt(`0x${readWord(1)}`);
  const mintNumber = BigInt(`0x${readWord(2)}`);

  const dynamicStart = nameOffsetBytes * 2;
  const lenWord = raw.slice(dynamicStart, dynamicStart + 64);
  if (!lenWord) {
    throw new Error('Missing dynamic string length word');
  }

  const nameLen = Number(BigInt(`0x${lenWord}`));
  const nameDataStart = dynamicStart + 64;
  const nameDataHex = raw.slice(nameDataStart, nameDataStart + nameLen * 2);
  const name = Buffer.from(nameDataHex, 'hex').toString('utf8');

  return {
    name,
    priceWei: priceWei.toString(),
    mintNumber: mintNumber.toString(),
  };
}

function loadState() {
  try {
    const raw = fs.readFileSync(STATE_PATH, 'utf8');
    return JSON.parse(raw);
  } catch {
    return {};
  }
}

function saveState(state) {
  fs.mkdirSync(STATE_DIR, { recursive: true });
  fs.writeFileSync(STATE_PATH, JSON.stringify(state, null, 2));
}

async function fetchLogsInChunks(fromBlock, toBlock, initialChunkSize = 3000) {
  const all = [];
  let start = fromBlock;
  let chunkSize = initialChunkSize;

  while (start <= toBlock) {
    const end = Math.min(toBlock, start + chunkSize - 1);

    try {
      const chunk = await rpc('eth_getLogs', [{
        address: MINT_CONTRACT,
        fromBlock: `0x${start.toString(16)}`,
        toBlock: `0x${end.toString(16)}`,
        topics: [NAME_MINTED_TOPIC],
      }]);

      all.push(...chunk);
      start = end + 1;

      if (chunkSize < 3000) {
        chunkSize = Math.min(3000, chunkSize * 2);
      }
    } catch (err) {
      const msg = String(err?.message || err).toLowerCase();
      if ((msg.includes('413') || msg.includes('too large')) && chunkSize > 100) {
        chunkSize = Math.max(100, Math.floor(chunkSize / 2));
        continue;
      }
      throw err;
    }
  }

  return all;
}

async function main() {
  const state = loadState();

  const latestHex = await rpc('eth_blockNumber');
  const latestBlock = hexToInt(latestHex);

  const hasState = Number.isInteger(state.lastCheckedBlock) && state.lastCheckedBlock >= 0;
  const fromBlock = hasState
    ? Math.min(state.lastCheckedBlock + 1, latestBlock)
    : Math.max(0, latestBlock - LOOKBACK_BLOCKS);
  const toBlock = latestBlock;

  const mode = hasState ? 'since-last-run' : 'initial-lookback';

  const logs = await fetchLogsInChunks(fromBlock, toBlock);

  const blockTsCache = new Map();
  async function getBlockTimestamp(blockHex) {
    if (blockTsCache.has(blockHex)) return blockTsCache.get(blockHex);
    const block = await rpc('eth_getBlockByNumber', [blockHex, false]);
    const ts = block?.timestamp ? hexToInt(block.timestamp) : null;
    blockTsCache.set(blockHex, ts);
    return ts;
  }

  const events = [];
  for (const log of logs) {
    try {
      const buyer = decodeAddressFromTopic(log.topics?.[1]);
      const decoded = decodeNameMintedData(log.data);
      const ts = await getBlockTimestamp(log.blockNumber);

      events.push({
        timestamp: ts ? new Date(ts * 1000).toISOString() : null,
        blockNumber: hexToInt(log.blockNumber),
        txHash: log.transactionHash,
        buyer,
        name: decoded.name,
        priceWei: decoded.priceWei,
        mintNumber: Number(decoded.mintNumber),
      });
    } catch (err) {
      events.push({
        decodeError: String(err?.message || err),
        blockNumber: hexToInt(log.blockNumber),
        txHash: log.transactionHash,
      });
    }
  }

  events.sort((a, b) => (a.blockNumber || 0) - (b.blockNumber || 0));

  const externalEvents = events.filter((e) => {
    if (!e || !e.buyer || e.decodeError) return false;
    return !RESERVE_WALLETS.has(e.buyer.toLowerCase());
  });

  const output = {
    checkedWindow: {
      fromBlock,
      toBlock,
      mode,
      contract: MINT_CONTRACT,
      rpc: RPC_URL,
    },
    totalEvents: events.filter((e) => !e.decodeError).length,
    externalEvents,
    recentEvents: events.slice(-10),
  };

  saveState({
    lastCheckedBlock: latestBlock,
    lastRunAt: new Date().toISOString(),
    lastResult: {
      totalEvents: output.totalEvents,
      externalEvents: output.externalEvents.length,
      fromBlock,
      toBlock,
    },
  });

  console.log(JSON.stringify(output, null, 2));
}

main().catch((err) => {
  const output = {
    error: String(err?.message || err),
    at: new Date().toISOString(),
    contract: MINT_CONTRACT,
    rpc: RPC_URL,
  };
  console.log(JSON.stringify(output, null, 2));
  process.exit(1);
});
