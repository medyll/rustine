#!/usr/bin/env node
const args = process.argv.slice(2);

if (args.length === 0 || args[0] !== 'wait') {
  console.error('Usage: node ./scripts/bmad.mjs wait --seconds <seconds>');
  process.exit(1);
}

const secondsIndex = args.indexOf('--seconds');
let seconds = 1;
if (secondsIndex !== -1) {
  const value = Number(args[secondsIndex + 1]);
  if (!Number.isFinite(value) || value < 0) {
    console.error('Invalid --seconds value');
    process.exit(1);
  }
  seconds = value;
}

console.log(`[bmad] wait ${seconds}s`);
setTimeout(() => {
  process.exit(0);
}, seconds * 1000);
