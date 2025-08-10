const { spawn } = require('child_process');

function waitForLine(proc, predicate, timeoutMs = 5000) {
  return new Promise((resolve, reject) => {
    let buffer = '';
    const deadline = Date.now() + timeoutMs;
    const onData = (data) => {
      buffer += data.toString('utf8');
      const lines = buffer.split(/\r?\n/);
      // Keep last partial line in buffer
      buffer = lines.pop() || '';
      for (const line of lines) {
        if (!line.trim()) continue;
        try {
          const msg = JSON.parse(line);
          if (predicate(msg)) {
            cleanup();
            return resolve(msg);
          }
        } catch (_) {
          // ignore non-JSON lines
        }
      }
      if (Date.now() > deadline) {
        cleanup();
        reject(new Error('Timeout waiting for matching line'));
      }
    };
    const onErr = () => { /* ignore stderr logs */ };
    const cleanup = () => {
      proc.stdout.off('data', onData);
      proc.stderr.off('data', onErr);
    };
    proc.stdout.on('data', onData);
    proc.stderr.on('data', onErr);
  });
}

(async () => {
  const server = spawn(process.execPath, ['mcp/archlens_mcp_server.cjs'], {
    stdio: ['pipe', 'pipe', 'pipe'],
    env: { ...process.env, ARCHLENS_DEBUG: 'false' }
  });

  const send = (obj) => server.stdin.write(JSON.stringify(obj) + '\n');

  // List tools
  const id1 = 1;
  send({ jsonrpc: '2.0', id: id1, method: 'tools/list' });
  const listResp = await waitForLine(server, m => m.id === id1 && (m.result || m.error));
  if (!listResp.result || !Array.isArray(listResp.result.tools) || listResp.result.tools.length === 0) {
    throw new Error('tools/list returned empty');
  }

  // Call get_project_structure with relative path
  const id2 = 2;
  send({ jsonrpc: '2.0', id: id2, method: 'tools/call', params: { name: 'get_project_structure', arguments: { project_path: '.', detail_level: 'summary' } } });
  const callResp = await waitForLine(server, m => m.id === id2 && (m.result || m.error));
  if (!callResp.result || !Array.isArray(callResp.result.content)) {
    throw new Error('tools/call missing content');
  }
  const textItem = callResp.result.content.find(c => c.type === 'text');
  if (!textItem || !textItem.text.includes('# 📁 STRUCTURE')) {
    throw new Error('STRUCTURE header not found in content');
  }

  console.log('OK: stdio e2e passed');
  server.kill('SIGTERM');
})().catch(err => {
  console.error('FAIL:', err.message);
  process.exit(1);
});