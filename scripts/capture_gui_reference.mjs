#!/usr/bin/env node
// Read-only Chromium reference capture. No package dependencies or HTML rewriting.
import {spawn} from 'node:child_process';
import {createHash} from 'node:crypto';
import {mkdtemp, mkdir, readdir, readFile, writeFile, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {dirname, isAbsolute, join, resolve} from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const source = join(root, 'docs/gui/prototypes/preferences-window.html');
const browserPath = '/usr/bin/chromium';
const timeoutMs = 15000;
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
const hashSource = async () => createHash('sha256').update(await readFile(source)).digest('hex');

function bounded(promise, label, milliseconds = timeoutMs) {
  let timer;
  return Promise.race([
    promise,
    new Promise((_, reject) => { timer = setTimeout(() => reject(Error(`${label} timed out`)), milliseconds); }),
  ]).finally(() => clearTimeout(timer));
}

async function connect(url) {
  const socket = new WebSocket(url);
  try {
    await bounded(new Promise((resolve, reject) => {
      socket.addEventListener('open', resolve, {once: true});
      socket.addEventListener('error', () => reject(Error('Chromium WebSocket failed')), {once: true});
    }), 'Chromium connection');
  } catch (error) {
    socket.close();
    throw error;
  }
  let serial = 0;
  const pending = new Map();
  const rejectAll = () => {
    for (const item of pending.values()) item.reject(Error('Chromium connection closed'));
    pending.clear();
  };
  socket.addEventListener('close', rejectAll);
  socket.addEventListener('error', rejectAll);
  socket.addEventListener('message', event => {
    const value = JSON.parse(event.data);
    const item = pending.get(value.id);
    if (!item) return;
    pending.delete(value.id);
    if (value.error) item.reject(Error(JSON.stringify(value.error)));
    else item.resolve(value.result);
  });
  return {
    close: () => { rejectAll(); socket.close(); },
    call(method, params = {}, sessionId) {
      const id = ++serial;
      return bounded(new Promise((resolve, reject) => {
        pending.set(id, {resolve, reject});
        socket.send(JSON.stringify({id, method, params, ...(sessionId ? {sessionId} : {})}));
      }), method).finally(() => pending.delete(id));
    },
  };
}

// Selection is deliberately read-only except scrolling: the original ancestor
// layout, siblings, embedded styles and responsive CSS remain in the document.
const selectionExpression = `(() => {
  const headings = [...document.querySelectorAll('.f5ghead')]
    .filter(node => node.textContent.trim().startsWith('C-B · Units'));
  if (headings.length !== 1) throw Error('Expected exactly one C-B Units heading');
  const heading = headings[0];
  const frame = heading.nextElementSibling;
  if (!frame?.classList.contains('f5win')) throw Error('C-B Units frame missing');
  frame.scrollIntoView({block: 'start', inline: 'start', behavior: 'instant'});
  const rect = node => node?.getBoundingClientRect().toJSON() ?? null;
  const box = frame.getBoundingClientRect();
  return {
    heading: heading.textContent.trim(),
    heading_index: [...document.querySelectorAll('.f5ghead')].indexOf(heading),
    selector: {heading: '.f5ghead', text_prefix: 'C-B · Units', frame: 'nextElementSibling.f5win'},
    viewport: {width: innerWidth, height: innerHeight, device_scale_factor: devicePixelRatio},
    scroll: {x: scrollX, y: scrollY},
    frame: rect(frame),
    frame_document: {x: box.x + scrollX, y: box.y + scrollY, width: box.width, height: box.height},
    rail: rect(frame.querySelector('.f5nav')),
    body_direction: getComputedStyle(frame.querySelector('.f5body')).flexDirection,
    controls: frame.querySelectorAll('[data-ctl]').length,
  };
})()`;

export async function capturePreferencesReference(outputDir) {
  if (!isAbsolute(outputDir)) throw Error('Reference output must be an absolute path');
  await mkdir(outputDir, {recursive: true});
  if ((await readdir(outputDir)).length) throw Error('Reference output must be empty');
  const before = await hashSource();
  const profile = await mkdtemp(join(tmpdir(), 'datum-reference-'));
  let browser;
  let browserLog = '';
  let launchError;
  let cdp;
  let interrupted;
  const stop = reason => {
    interrupted = Error(reason);
    cdp?.close();
    browser?.kill('SIGTERM');
  };
  // Keep the entire capture plus the six-second shutdown inside the native
  // caller's 60-second budget, rather than multiplying per-request deadlines.
  const deadlineTimer = setTimeout(() => stop('Reference capture exceeded 45 seconds'), 45000);
  const onTerm = () => stop('Reference capture interrupted by SIGTERM');
  const onInt = () => stop('Reference capture interrupted by SIGINT');
  process.on('SIGTERM', onTerm);
  process.on('SIGINT', onInt);
  try {
    browser = spawn(browserPath, [
      '--headless=new', '--disable-gpu', '--disable-dev-shm-usage', '--no-first-run',
      '--no-default-browser-check', '--disable-background-networking',
      '--disable-extensions', '--password-store=basic',
      '--remote-debugging-port=0', `--user-data-dir=${profile}`, 'about:blank',
    ], {stdio: ['ignore', 'ignore', 'pipe']});
    browser.on('error', error => { launchError = error; });
    browser.stderr.on('data', chunk => { browserLog = (browserLog + chunk).slice(-65536); });
    let portInfo;
    const deadline = Date.now() + timeoutMs;
    while (Date.now() < deadline) {
      if (interrupted) throw interrupted;
      if (launchError) throw launchError;
      if (browser.exitCode !== null || browser.signalCode !== null) throw Error(`Chromium exited: ${browserLog.slice(-1000)}`);
      try { portInfo = await readFile(join(profile, 'DevToolsActivePort'), 'utf8'); break; } catch {}
      await pause(100);
    }
    if (!portInfo) throw Error('Chromium debugging endpoint timed out');
    const [port, endpoint] = portInfo.trim().split('\n');
    cdp = await connect(`ws://127.0.0.1:${port}${endpoint}`);
    const version = await cdp.call('Browser.getVersion');
    const {targetId} = await cdp.call('Target.createTarget', {url: 'about:blank'});
    const {sessionId} = await cdp.call('Target.attachToTarget', {targetId, flatten: true});
    const page = (method, params = {}) => cdp.call(method, params, sessionId);
    const evaluate = async (expression, awaitPromise = false) => {
      const result = await page('Runtime.evaluate', {expression, returnByValue: true, awaitPromise});
      if (result.exceptionDetails) throw Error(JSON.stringify(result.exceptionDetails));
      return result.result.value;
    };
    await page('Page.enable');
    const captures = [];
    for (const [name, width, height] of [['wide', 1160, 720], ['narrow', 720, 720]]) {
      await page('Emulation.setDeviceMetricsOverride', {width, height, deviceScaleFactor: 1, mobile: false});
      const navigation = await page('Page.navigate', {url: pathToFileURL(source).href});
      if (navigation.errorText) throw Error(navigation.errorText);
      const readyDeadline = Date.now() + timeoutMs;
      let ready = false;
      while (Date.now() < readyDeadline) {
        ready = await evaluate(`location.href === ${JSON.stringify(pathToFileURL(source).href)} && document.readyState === 'complete'`);
        if (ready) break;
        await pause(50);
      }
      if (!ready) throw Error('Prototype document load timed out');
      await evaluate('document.fonts.ready.then(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))))', true);
      const geometry = await evaluate(selectionExpression);
      const frame = geometry.frame_document;
      const clip = {x: geometry.scroll.x, y: geometry.scroll.y, width, height, scale: 1};
      if (!(frame.width > 0 && frame.height > 0)) throw Error('Empty prototype frame');
      const capture = await page('Page.captureScreenshot', {format: 'png', captureBeyondViewport: false, fromSurface: true});
      const png = Buffer.from(capture.data, 'base64');
      const filename = `prototype-units-${name}.png`;
      await writeFile(join(outputDir, filename), png, {flag: 'wx'});
      captures.push({filename, geometry, crop_document: clip,
        image_pixels: {width: png.readUInt32BE(16), height: png.readUInt32BE(20)},
        cropped: frame.x < clip.x || frame.y < clip.y || frame.x + frame.width > clip.x + width || frame.y + frame.height > clip.y + height});
    }
    const after = await hashSource();
    if (interrupted) throw interrupted;
    if (before !== after) throw Error('Prototype source changed while capturing');
    const result = {source, source_sha256: before, source_unchanged: true,
      browser: browserPath, browser_version: version, selection_expression: selectionExpression,
      limits: 'Original C-B Units DOM and CSS, scrolled into the viewport without relocating nodes or overriding styles. The image is the viewport crop, including original surrounding margins. Viewport size is not the frame size; actual frame/crop/image dimensions are recorded. Content beyond the crop is not shown. Reference capture supplies no native equivalence or cross-engine PASS.', captures};
    await writeFile(join(outputDir, 'prototype-reference-capture.json'), JSON.stringify(result, null, 2) + '\n', {flag: 'wx'});
    if (interrupted) throw interrupted;
    return result;
  } finally {
    clearTimeout(deadlineTimer);
    cdp?.close();
    try {
      if (browser && browser.exitCode === null && browser.signalCode === null && !launchError) {
        const exited = new Promise(resolve => browser.once('exit', resolve));
        browser.kill('SIGTERM');
        try { await bounded(exited, 'Chromium shutdown', 3000); }
        catch { browser.kill('SIGKILL'); await bounded(exited, 'Chromium forced shutdown', 3000); }
      }
    } finally {
      try {
        try { await writeFile(join(outputDir, 'prototype-browser.log'), browserLog); }
        finally { await rm(profile, {recursive: true, force: true}); }
      } finally {
        process.off('SIGTERM', onTerm);
        process.off('SIGINT', onInt);
      }
    }
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  if (process.argv.length !== 4 || process.argv[2] !== '--output') {
    console.error('Usage: node scripts/capture_gui_reference.mjs --output ABS_EMPTY_OR_NEW_DIR');
    process.exitCode = 2;
  } else {
    capturePreferencesReference(process.argv[3]).then(result => {
      console.log(JSON.stringify({source_unchanged: result.source_unchanged, captures: result.captures}));
    }).catch(error => { console.error(error.message); process.exitCode = 1; });
  }
}
