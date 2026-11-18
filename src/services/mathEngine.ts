/**
 * Arbitrary-precision Pi calculation using Chudnovsky & Ramanujan formulas
 * Faithful port from bnn-code/src/math.rs with Binary Splitting algorithm
 */

interface BsResult {
  p: bigint;
  q: bigint;
  t: bigint;
}

// 640320^3 / 24 = 262537412640768000 / 24 = 10939058860032000n
const C3_24: bigint = 640320n ** 3n / 24n;
// 396^4 = 24591257856n
const C396_POW4: bigint = 396n ** 4n;

// Reference known digits of Pi to verify precision up to 500+ digits
export const KNOWN_PI_DIGITS = 
  "3.1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679" +
  "8214808651328230664709384460955058223172535940812848111745028410270193852110555964462294895493038196" +
  "4428810975665933446128475648233786783165271201909145648566923460348610454326648213393607260249141273" +
  "7245870066063155881748815209209628292540917153643678925903600113305305488204665213841469519415116094" +
  "3305727036575959195309218611738193261179310511854807446237996274956735188575272489122793818301194912";

function integerSqrt(n: bigint): bigint {
  if (n <= 0n) return 0n;
  if (n === 1n) return 1n;
  // Newton's method with bit-length estimation
  const s = n.toString(16);
  const bitLength = (s.length * 4);
  let x = 1n << BigInt(Math.floor(bitLength / 2) + 1);

  while (true) {
    const y = (x + n / x) >> 1n;
    if (y >= x) {
      return x;
    }
    x = y;
  }
}

function chudnovskyBs(a: bigint, b: bigint): BsResult {
  if (b - a === 1n) {
    if (a === 0n) {
      return {
        p: 1n,
        q: 1n,
        t: 13591409n,
      };
    }
    const p = -(6n * a - 5n) * (2n * a - 1n) * (6n * a - 1n);
    const q = (a ** 3n) * C3_24;
    const t = p * (545140134n * a + 13591409n);
    return { p, q, t };
  } else {
    const m = (a + b) / 2n;
    const left = chudnovskyBs(a, m);
    const right = chudnovskyBs(m, b);
    return {
      p: left.p * right.p,
      q: left.q * right.q,
      t: left.t * right.q + left.p * right.t,
    };
  }
}

function ramanujanBs(a: bigint, b: bigint): BsResult {
  if (b - a === 1n) {
    if (a === 0n) {
      return {
        p: 1n,
        q: 1n,
        t: 1103n,
      };
    }
    const p = (4n * a - 3n) * (4n * a - 2n) * (4n * a - 1n) * (4n * a);
    const q = (a ** 4n) * C396_POW4;
    const t = p * (1103n + 26390n * a);
    return { p, q, t };
  } else {
    const m = (a + b) / 2n;
    const left = ramanujanBs(a, m);
    const right = ramanujanBs(m, b);
    return {
      p: left.p * right.p,
      q: left.q * right.q,
      t: left.t * right.q + left.p * right.t,
    };
  }
}

function formatPi(scaled: bigint, digits: number): string {
  const s = scaled.toString();
  const len = s.length;
  if (len <= digits) {
    const padding = "0".repeat(digits - len + 1);
    return `0.${padding}${s}`;
  } else {
    const intPart = s.slice(0, len - digits);
    const fracPart = s.slice(len - digits, len);
    return `${intPart}.${fracPart}`;
  }
}

export function computeChudnovskyPi(digits: number = 100): { pi: string; durationMs: number; terms: number } {
  const start = performance.now();
  const terms = Math.ceil(digits / 14) + 2;
  const result = chudnovskyBs(0n, BigInt(terms));
  const scale = 10n ** BigInt(digits);
  const sqrtArg = 10005n * scale * scale;
  const sqrt10005 = integerSqrt(sqrtArg);
  const c = 426880n * sqrt10005;
  const piScaled = (c * result.q) / result.t;
  const pi = formatPi(piScaled, digits);
  const durationMs = performance.now() - start;

  return { pi, durationMs, terms };
}

export function computeRamanujanPi(digits: number = 100): { pi: string; durationMs: number; terms: number } {
  const start = performance.now();
  const terms = Math.ceil(digits / 8) + 2;
  const result = ramanujanBs(0n, BigInt(terms));
  const scale = 10n ** BigInt(digits);
  const sqrtArg = 2n * scale * scale;
  const sqrt2 = integerSqrt(sqrtArg);
  const twoRoot2 = 2n * sqrt2;
  const nine801 = 9801n;
  const piScaled = (nine801 * result.q * scale * scale) / (twoRoot2 * result.t);
  const pi = formatPi(piScaled, digits);
  const durationMs = performance.now() - start;

  return { pi, durationMs, terms };
}

export function verifyPiAccuracy(computedPi: string): { matches: boolean; correctDigits: number; mismatchIndex: number } {
  const cleanComputed = computedPi.replace('.', '');
  const cleanKnown = KNOWN_PI_DIGITS.replace('.', '');

  let correctDigits = 0;
  let mismatchIndex = -1;

  for (let i = 0; i < cleanComputed.length; i++) {
    if (i < cleanKnown.length) {
      if (cleanComputed[i] === cleanKnown[i]) {
        correctDigits++;
      } else {
        mismatchIndex = i;
        break;
      }
    } else {
      break;
    }
  }

  return {
    matches: mismatchIndex === -1 && correctDigits > 0,
    correctDigits,
    mismatchIndex,
  };
}
