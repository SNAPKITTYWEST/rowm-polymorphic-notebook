// EmojiCode Parser — Formal Parser for Emoji-based Instructions
// Produces deterministic canonical representation

import crypto from 'crypto';

// Emoji to (Runtime, Verb) mapping
const EMOJI_MAP = {
  '⚡': { runtime: 'holyc', verb: 'Execute' },
  '🦀': { runtime: 'rust', verb: 'LocExecute' },
  '✅': { runtime: 'ada', verb: 'Verify' },
  '🐱': { runtime: 'haskell', verb: 'ConsCell' },
  '🔤': { runtime: 'emoji', verb: 'Encode' },
  '⚙️': { runtime: 'rust', verb: 'Configure' },
  '📝': { runtime: 'python3', verb: 'LogWrite' },
};

/**
 * parse(expression: string): Instruction
 * Parse emoji expression into canonical instruction object
 *
 * Syntax: EMOJI ARG1:value1 ARG2:value2 ...
 * Example: ⚡ fn:FreqAnchor1618 mode:ring0
 */
export function parse(expression) {
  const trimmed = expression.trim();
  if (!trimmed) {
    throw new Error('Empty expression');
  }

  const tokens = trimmed.split(/\s+/);
  const emoji = tokens[0];

  if (!EMOJI_MAP[emoji]) {
    throw new Error(`Unknown emoji: ${emoji}`);
  }

  const { runtime, verb } = EMOJI_MAP[emoji];
  const args = {};

  // Parse key:value arguments
  for (let i = 1; i < tokens.length; i++) {
    const token = tokens[i];
    if (token.includes(':')) {
      const [key, value] = token.split(':', 2);
      args[key] = parseValue(value);
    } else {
      throw new Error(`Invalid token format: ${token} (expected key:value)`);
    }
  }

  // Build canonical instruction
  const instruction = {
    protocol_version: '1.0.0',
    symbol: emoji,
    target_runtime: runtime,
    verb: verb,
    arguments: args,
    timestamp: Math.floor(Date.now() / 1000),
  };

  // Compute deterministic hash
  const canonical = canonicalize(instruction);
  instruction.instruction_hash = hashInstruction(canonical);
  instruction.instruction_id = instruction.instruction_hash;

  return instruction;
}

/**
 * canonicalize(instruction: object): string
 * Produce deterministic JSON-LD representation for hashing
 */
function canonicalize(instruction) {
  // Sort keys and arguments for determinism
  const sorted = {
    protocol_version: instruction.protocol_version,
    symbol: instruction.symbol,
    target_runtime: instruction.target_runtime,
    verb: instruction.verb,
    arguments: sortObjectKeys(instruction.arguments),
    timestamp: instruction.timestamp,
  };

  return JSON.stringify(sorted);
}

/**
 * sortObjectKeys(obj: object): object
 * Recursively sort object keys
 */
function sortObjectKeys(obj) {
  if (typeof obj !== 'object' || obj === null || Array.isArray(obj)) {
    return obj;
  }

  const sorted = {};
  Object.keys(obj)
    .sort()
    .forEach((key) => {
      sorted[key] = sortObjectKeys(obj[key]);
    });

  return sorted;
}

/**
 * hashInstruction(canonical: string): string
 * SHA-256 hash of canonical representation
 */
function hashInstruction(canonical) {
  return crypto.createHash('sha256').update(canonical).digest('hex');
}

/**
 * parseValue(valueStr: string): string | number | boolean
 * Parse value string to appropriate type
 */
function parseValue(valueStr) {
  if (valueStr === 'true') return true;
  if (valueStr === 'false') return false;
  if (/^\d+$/.test(valueStr)) return parseInt(valueStr, 10);
  if (/^\d+\.\d+$/.test(valueStr)) return parseFloat(valueStr);
  return valueStr;
}

/**
 * roundTripTest(expression: string): object
 * Verify that parse and canonicalize produce deterministic results
 */
export function roundTripTest(expression) {
  const instr1 = parse(expression);
  const instr2 = parse(expression);

  return {
    expr: expression,
    hash1: instr1.instruction_hash,
    hash2: instr2.instruction_hash,
    equal: instr1.instruction_hash === instr2.instruction_hash,
    instruction: instr1,
  };
}

// CLI for testing
if (import.meta.url === `file://${process.argv[1]}`) {
  const tests = [
    '⚡ fn:FreqAnchor1618',
    '🦀 mode:release',
    '✅ contract:borrow step:3',
    '🐱 cons:car_cdr',
  ];

  console.log('=== EmojiCode Parser Round-trip Tests ===');
  for (const test of tests) {
    const result = roundTripTest(test);
    console.log(`${test:<30} → hash: ${result.hash1.slice(0, 16)}...`);
    console.log(`  Deterministic: ${result.equal ? 'YES' : 'NO'}`);
  }
}
