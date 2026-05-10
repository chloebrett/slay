import init, { WasmSession } from './pkg/slay_wasm.js';

await init();

const term = new Terminal({
  cols: 100,
  rows: 40,
  theme: {
    background: '#0d0d0d',
    foreground: '#e8e8e8',
    cursor: '#e94560',
  },
  fontFamily: 'Menlo, Monaco, "Courier New", monospace',
  fontSize: 14,
  convertEol: true,
});

term.open(document.getElementById('terminal'));

const session = new WasmSession();

function writeOutput(text) {
  // Normalise LF → CRLF so xterm renders correctly.
  term.write(text.replace(/\n/g, '\r\n'));
}

// Write initial state.
writeOutput(session.send(''));

// Line-buffer for keyboard input.
let line = '';

term.onKey(({ key, domEvent }) => {
  if (session.is_over()) return;

  const code = domEvent.keyCode;

  if (code === 13) {
    // Enter — submit line.
    term.write('\r\n');
    if (line.trim().length > 0) {
      writeOutput('> ' + line + '\r\n');
      const response = session.send(line);
      writeOutput(response);
    }
    line = '';
  } else if (code === 8) {
    // Backspace.
    if (line.length > 0) {
      line = line.slice(0, -1);
      term.write('\b \b');
    }
  } else if (key.length === 1) {
    // Printable character.
    line += key;
    term.write(key);
  }
});
