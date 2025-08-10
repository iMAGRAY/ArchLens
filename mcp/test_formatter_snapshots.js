const assert = (cond, msg) => { if (!cond) throw new Error(msg || 'Assertion failed'); };

const mod = require('./archlens_mcp_server.cjs');

// 1) Export formatter snapshots
{
  const projectPath = process.cwd();
  const big = ('# Head\n```ts\nlet x=1;\n```\npara\n').repeat(3000);
  const s = mod.ResponseFormatter.formatExportResult({ output: big }, projectPath, 'summary');
  assert(!/```/.test(s), 'summary must strip code blocks');
  assert(s.length <= (global.SUMMARY_LIMIT_CHARS + 50), 'summary must be clamped');

  const std = mod.ResponseFormatter.formatExportResult({ output: big }, projectPath, 'standard');
  assert(std.length <= (global.SUMMARY_LIMIT_CHARS * 2 + 50), 'standard must be clamped (2x)');

  const full = mod.ResponseFormatter.formatExportResult({ output: big }, projectPath, 'full');
  assert(full.includes('# Head'), 'full must preserve content');
}

// 2) Diagram formatter snapshots
{
  const projectPath = process.cwd();
  const mermaid = ('graph TD\nA-->B\n').repeat(5000);
  const dSum = mod.ResponseFormatter.formatDiagramResult({ diagram: mermaid, diagram_type: 'mermaid' }, projectPath, 'summary');
  assert(dSum.startsWith('# 📊 DIAGRAM'), 'diagram summary must include header');
  assert(dSum.includes('```mermaid'), 'diagram must be fenced');

  const dStd = mod.ResponseFormatter.formatDiagramResult({ diagram: mermaid, diagram_type: 'mermaid' }, projectPath, 'standard');
  assert(dStd.length >= dSum.length, 'standard should be longer or equal');

  const dFull = mod.ResponseFormatter.formatDiagramResult({ diagram: mermaid, diagram_type: 'mermaid' }, projectPath, 'full');
  assert(dFull.length >= dStd.length, 'full should be longest');
}

console.log('OK: formatter snapshot tests passed');