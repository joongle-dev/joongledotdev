import init, { run } from './yahtzee.js';

const canvas = document.getElementById('canvas');

async function run_yahtzee() {
    await init();
    await run(canvas);
}
await run_yahtzee();