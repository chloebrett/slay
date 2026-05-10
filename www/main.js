import init, { WasmTuiSession } from './pkg/slay_wasm.js';

await init();

const term = new Terminal({
  cols: 120,
  rows: 40,
  theme: {
    background: '#0d0d0d',
    foreground: '#e8e8e8',
    cursor: '#e94560',
  },
  fontFamily: 'Menlo, Monaco, "Courier New", monospace',
  fontSize: 14,
  convertEol: false,
});

term.open(document.getElementById('terminal'));

const session = new WasmTuiSession();

// Write initial render.
term.write(session.send(''));

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
