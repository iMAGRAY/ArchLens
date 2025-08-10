#!/usr/bin/env node
const { spawn } = require('child_process');

function waitForLine(proc, predicate, timeoutMs = 60000) {
  return new Promise((resolve, reject) => {
    let buffer = '';
    const deadline = Date.now() + timeoutMs;
    const onData = (data) => {
      buffer += data.toString('utf8');
      const lines = buffer.split(/\r?\n/);
      buffer = lines.pop() || '';
      for (const line of lines) {
        if (!line.trim()) continue;
        try {
          const msg = JSON.parse(line);
          if (predicate(msg)) {
            cleanup();
            return resolve(msg);
          }
        } catch (_) {}
      }
      if (Date.now() > deadline) {
        cleanup();
        reject(new Error('Timeout waiting for matching line'));
      }
    };
    const onErr = () => {};
    const cleanup = () => {
      proc.stdout.off('data', onData);
      proc.stderr.off('data', onErr);
    };
    proc.stdout.on('data', onData);
    proc.stderr.on('data', onErr);
  });
}

(async () => {
  const serverPath = '/workspace/target/release/archlens-mcp';
  const env = { ...process.env, ARCHLENS_DEBUG: 'false' };
  const server = spawn(serverPath, { stdio: ['pipe', 'pipe', 'pipe'], env });
  const send = (obj) => server.stdin.write(JSON.stringify(obj) + '\n');

  try {
    // 1) tools/list
    const id1 = 1;
    send({ jsonrpc: '2.0', id: id1, method: 'tools/list' });
    const listResp = await waitForLine(server, m => m.id === id1 && (m.result || m.error), 15000);
    if (!listResp.result || !Array.isArray(listResp.result.tools)) throw new Error('tools/list invalid');
    const names = listResp.result.tools.map(t => t.name).sort();
    const expected = ['export_ai_compact','export_ai_summary_json','structure_get','graph_build','analyze_project','ai_recommend','arch_refresh'];
    for (const n of expected) if (!names.includes(n)) throw new Error(`missing tool ${n}`);

    // 2) structure_get (underscore)
    const id2 = 2;
    send({ jsonrpc: '2.0', id: id2, method: 'tools/call', params: { name: 'structure_get', arguments: { project_path: 'tests/fixtures/small_project', detail_level: 'summary' } } });
    const structResp = await waitForLine(server, m => m.id === id2 && (m.result || m.error), 20000);
    if (!structResp.result || !Array.isArray(structResp.result.content)) throw new Error('structure_get missing content');
    const sText = structResp.result.content.find(c => c.type === 'text')?.text || '';
    if (!sText.includes('# 📁 STRUCTURE')) throw new Error('structure header missing');

    // 3) export_ai_summary_json (underscore)
    const id3 = 3;
    send({ jsonrpc: '2.0', id: id3, method: 'tools/call', params: { name: 'export_ai_summary_json', arguments: { project_path: 'tests/fixtures/small_project', top_n: 3 } } });
    const sumResp = await waitForLine(server, m => m.id === id3 && (m.result || m.error), 60000);
    if (!sumResp.result || !sumResp.result.json) throw new Error('ai_summary_json missing json');

    // 4) ai_recommend
    const id4 = 4;
    send({ jsonrpc: '2.0', id: id4, method: 'tools/call', params: { name: 'ai_recommend', arguments: { project_path: 'tests/fixtures/small_project', json: sumResp.result.json } } });
    const recResp = await waitForLine(server, m => m.id === id4 && (m.result || m.error), 15000);
    if (!recResp.result || recResp.result.status !== 'ok') throw new Error('ai_recommend status not ok');
    const recs = Array.isArray(recResp.result.recommendations) ? recResp.result.recommendations : [];
    if (recs.length === 0) throw new Error('no recommendations');

    // 5) alias dotted for export.ai_summary_json
    const id5 = 5;
    send({ jsonrpc: '2.0', id: id5, method: 'tools/call', params: { name: 'export.ai_summary_json', arguments: { project_path: 'tests/fixtures/small_project', top_n: 2 } } });
    const aliasResp = await waitForLine(server, m => m.id === id5 && (m.result || m.error), 30000);
    if (!aliasResp.result || !aliasResp.result.json) throw new Error('dotted alias failed');

    // 6) heavy: export_ai_compact
    const id6 = 6;
    send({ jsonrpc: '2.0', id: id6, method: 'tools/call', params: { name: 'export_ai_compact', arguments: { project_path: 'tests/fixtures/small_project', detail_level: 'summary', top_n: 3, max_output_chars: 10000 } } });
    const compactResp = await waitForLine(server, m => m.id === id6 && (m.result || m.error), 60000);
    if (!compactResp.result || !Array.isArray(compactResp.result.content)) throw new Error('ai_compact missing content');

    console.log('OK: Rust MCP STDIO auto‑run passed');
    server.kill('SIGTERM');
    process.exit(0);
  } catch (e) {
    console.error('FAIL:', e.message);
    server.kill('SIGTERM');
    process.exit(1);
  }
})();