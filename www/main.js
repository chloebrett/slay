import init, { WasmTuiSession } from './pkg/slay_wasm.js';

const isMobile = /Mobi|Android|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent)
  || window.matchMedia('(pointer: coarse) and (max-width: 1024px)').matches;
if (isMobile) {
  const container = document.getElementById('terminal');
  container.style.cssText =
    'display:flex;align-items:center;justify-content:center;' +
    'color:#e8e8e8;font-size:1.1rem;text-align:center;padding:2rem;line-height:1.6';
  container.textContent = 'This game works best on a desktop browser 💻';
  throw new Error('mobile');
}

await init();

const term = new Terminal({
  theme: {
    background: '#0a0a0a',
    foreground: '#c9a96e',
    cursor: '#c0392b',
    cursorAccent: '#0a0a0a',
    selectionBackground: '#c9a96e33',
  },
  fontFamily: '"Courier Prime", "Courier New", monospace',
  fontSize: 14,
  convertEol: false,
});

const container = document.getElementById('terminal');
term.open(container);
term.focus();

// Measure a single character cell using a temporary off-screen element that
// matches the terminal's font exactly. Returns { cellW, cellH } in pixels.
function measureCell() {
  const probe = document.createElement('span');
  probe.style.cssText =
    `position:absolute;visibility:hidden;white-space:pre;` +
    `font-family:${term.options.fontFamily};font-size:${term.options.fontSize}px`;
  probe.textContent = 'W';
  document.body.appendChild(probe);
  const rect = probe.getBoundingClientRect();
  document.body.removeChild(probe);
  return { cellW: rect.width, cellH: rect.height };
}

const session = new WasmTuiSession();

function fit() {
  const { cellW, cellH } = measureCell();
  // Leave a small gutter so scroll bars never appear.
  const cols = Math.max(80, Math.floor((window.innerWidth  - 32) / cellW));
  const rows = Math.max(24, Math.floor((window.innerHeight - 96) / cellH));

  if (cols !== term.cols || rows !== term.rows) {
    term.resize(cols, rows);
    const output = session.resize(cols, rows);
    term.write(output);
  }
}

fit();

// Write initial render after first fit, then reveal now that cols/rows are correct.
term.write(session.send(''));
container.style.visibility = 'visible';

window.addEventListener('resize', fit);

term.onKey(({ key, domEvent }) => {
  if (session.is_over()) return;

  const code = domEvent.keyCode;
  let output = '';

  if (code === 13) {
    output = session.send_key('Enter');
  } else if (code === 8) {
    output = session.send_key('Backspace');
  } else if (code === 27) {
    output = session.send_key('Esc');
  } else if (code === 38) {
    output = session.send_key('Up');
  } else if (code === 40) {
    output = session.send_key('Down');
  } else if (key.length === 1) {
    output = session.send(key);
  }

  if (output) {
    term.write(output);
  }
});
