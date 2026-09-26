import { readFile, realpath, stat } from 'node:fs/promises';
import { createServer } from 'node:http';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const REPOSITORY = fileURLToPath(new URL('../../', import.meta.url));
const DESIGN_ROOT = path.join(REPOSITORY, 'design-system');
const DEFAULT_PORT = 4265;
const PORT = Number(process.env.PORT ?? DEFAULT_PORT);
const HOST = '127.0.0.1';
const PREVIEW_PATH = '/design-system/preview/';
const CATALOGUES = new Set(['en.json', 'fr.json', 'languages.json']);
const DIRECTORIES = new Set(['assets', 'docs', 'preview', 'tokens']);
const TYPES = new Map([
  ['.html', 'text/html; charset=utf-8'],
  ['.css', 'text/css; charset=utf-8'],
  ['.js', 'text/javascript; charset=utf-8'],
  ['.mjs', 'text/javascript; charset=utf-8'],
  ['.json', 'application/json; charset=utf-8'],
  ['.svg', 'image/svg+xml'],
  ['.ttf', 'font/ttf'],
  ['.woff2', 'font/woff2'],
  ['.md', 'text/plain; charset=utf-8'],
  ['.txt', 'text/plain; charset=utf-8'],
  ['.png', 'image/png'],
  ['.webp', 'image/webp'],
  ['.jpg', 'image/jpeg'],
]);

function isContained(root, filename) {
  const relative = path.relative(root, filename);
  return relative !== '..' && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative);
}

function isAllowed(filename) {
  if (!isContained(REPOSITORY, filename)) return false;
  const relative = path.relative(REPOSITORY, filename).split(path.sep);
  if (relative.some((part) => part.startsWith('.'))) return false;
  if (relative[0] === 'i18n') return relative.length === 2 && CATALOGUES.has(relative[1]);
  if (!isContained(DESIGN_ROOT, filename) || relative[0] !== 'design-system') return false;
  if (relative.length === 2 && relative[1] === 'README.md') return true;
  return DIRECTORIES.has(relative[1]) && TYPES.has(path.extname(filename).toLowerCase());
}

function requestPath(request) {
  const url = new URL(request.url ?? '/', `http://${HOST}:${PORT}`);
  const decoded = decodeURIComponent(url.pathname);
  if (decoded.includes('\\') || decoded.includes('\0')) throw new URIError('Invalid path');
  if (decoded.split('/').some((part) => part === '..' || part.startsWith('.'))) {
    throw new URIError('Invalid path');
  }
  const pathname = decoded.endsWith('/') ? `${decoded}index.html` : decoded;
  return path.resolve(REPOSITORY, `.${pathname}`);
}

async function loadStatic(filename) {
  if (!isAllowed(filename)) return null;
  const canonical = await realpath(filename);
  if (!isAllowed(canonical) || !(await stat(canonical)).isFile()) return null;
  const type = TYPES.get(path.extname(canonical).toLowerCase());
  return { bytes: await readFile(canonical), type };
}

function reply(response, status, message) {
  response.writeHead(status, { 'Content-Type': 'text/plain; charset=utf-8' });
  response.end(message);
}

async function handleRequest(request, response) {
  response.setHeader('Cache-Control', 'no-store');
  response.setHeader('X-Content-Type-Options', 'nosniff');
  if (request.method !== 'GET' && request.method !== 'HEAD') {
    response.setHeader('Allow', 'GET, HEAD');
    return reply(response, 405, 'Method not allowed');
  }
  if (request.url === '/') {
    response.writeHead(302, { Location: PREVIEW_PATH });
    return response.end();
  }
  try {
    const resource = await loadStatic(requestPath(request));
    if (!resource) return reply(response, 404, 'Not found');
    response.writeHead(200, {
      'Content-Type': resource.type,
      'Content-Length': resource.bytes.length,
    });
    response.end(request.method === 'HEAD' ? undefined : resource.bytes);
  } catch (error) {
    reply(response, error instanceof URIError ? 400 : 404, 'Not found');
  }
}

if (!Number.isInteger(PORT) || PORT < 1 || PORT > 65535) {
  throw new Error('PORT must be an integer between 1 and 65535.');
}

const server = createServer(handleRequest);
server.on('error', (error) => {
  console.error(`Preview server could not start: ${error.code ?? 'unknown error'}`);
  process.exitCode = 1;
});
server.listen(PORT, HOST, () => {
  console.log(`Life Pixel design preview: http://${HOST}:${PORT}${PREVIEW_PATH}`);
});
