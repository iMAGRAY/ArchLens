/* Lightweight integration tests for archlens_mcp_server.cjs
   Runs without Rust binary; focuses on path resolution, formatting and manual structure. */

const assert = (cond, msg) => { if (!cond) throw new Error(msg || 'Assertion failed'); };
const equal = (a, b, msg) => assert(a === b, msg || `Expected ${a} === ${b}`);

(async () => {
  const mod = require('./archlens_mcp_server.cjs');

  // Speed limits for tests
  mod.CONFIG.limits.scanTimeout = 1500;
  mod.CONFIG.limits.maxFiles = 150;
  mod.CONFIG.limits.scanDepth = 6;

  // 1) resolveProjectPath accepts '.' and returns absolute cwd
  const cwd = process.cwd();
  const resolved = mod.resolveProjectPath('.');
  equal(resolved, cwd, 'resolveProjectPath(".") must equal process.cwd()');

  // 2) stripCodeBlocks removes fenced blocks
  const withCode = 'header\n```js\nconsole.log(1)\n```\nfooter';
  const stripped = mod.stripCodeBlocks(withCode);
  assert(!/```/.test(stripped), 'stripCodeBlocks must remove fenced blocks');

  // 3) clampText truncates
  const long = 'x'.repeat(1000);
  const clamped = mod.clampText(long, 100);
  assert(clamped.length <= 125, 'clampText should truncate and append suffix');
  assert(clamped.endsWith('(truncated)'), 'clampText should indicate truncation');

  // 4) ResponseFormatter.formatExportResult strips code and clamps for summary
  const md = '# Title\n```ts\nconst a = 1;\n```\nparagraph '.repeat(300);
  const formatted = mod.ResponseFormatter.formatExportResult({ output: md }, resolved, 'summary');
  assert(formatted.length <= (global.SUMMARY_LIMIT_CHARS + 50), 'summary must be clamped near SUMMARY_LIMIT_CHARS');
  assert(!/```/.test(formatted), 'summary must not contain code fences');

  // 5) Manual structure builder returns something useful quickly
  const manual = mod.createManualStructure(resolved);
  equal(manual.status, 'success', 'manual structure status should be success');
  assert(Array.isArray(manual.structure.files), 'manual structure files must be an array');

  // 6) Handler get_project_structure supports relative path and falls back to manual scan (no binary required)
  const resp = await mod.handleGetProjectStructure({ project_path: '.', detail_level: 'summary' });
  assert(resp && resp.content && resp.content[0] && typeof resp.content[0].text === 'string', 'handler must return text content');
  assert(resp.content[0].text.includes('# 📁 STRUCTURE'), 'structure header must be present');

  // 7) export_ai_compact should return error response when binary missing (but not throw)
  const exportResp = await mod.handleExportAICompact({ project_path: '.', detail_level: 'summary' });
  assert(exportResp && exportResp.isError === true, 'export_ai_compact should return error when binary missing');

  console.log('OK: all MCP tests passed');
})().catch(err => {
  console.error('FAIL:', err.message);
  process.exit(1);
});