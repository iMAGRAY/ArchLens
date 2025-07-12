#!/usr/bin/env node

/**
 * ArchLens MCP Integration Example
 * 
 * This example demonstrates how to integrate ArchLens MCP server
 * with your own applications or AI tools.
 */

const { spawn } = require('child_process');
const path = require('path');

class ArchLensMCPClient {
    constructor(serverPath) {
        this.serverPath = serverPath;
        this.requestId = 1;
    }

    async callTool(toolName, args) {
        return new Promise((resolve, reject) => {
            console.log(`🔧 Calling tool: ${toolName}`);
            console.log(`📋 Arguments:`, JSON.stringify(args, null, 2));

            const mcpProcess = spawn('node', [this.serverPath], {
                stdio: ['pipe', 'pipe', 'pipe']
            });

            const request = {
                jsonrpc: "2.0",
                id: this.requestId++,
                method: "tools/call",
                params: {
                    name: toolName,
                    arguments: args
                }
            };

            let stdout = '';
            let stderr = '';

            mcpProcess.stdout.on('data', (data) => {
                stdout += data.toString();
            });

            mcpProcess.stderr.on('data', (data) => {
                stderr += data.toString();
            });

            mcpProcess.on('close', (code) => {
                if (code === 0) {
                    try {
                        const lines = stdout.trim().split('\n');
                        const lastLine = lines[lines.length - 1];
                        const response = JSON.parse(lastLine);
                        
                        if (response.result) {
                            resolve(response.result);
                        } else if (response.error) {
                            reject(new Error(`MCP Error: ${response.error.message}`));
                        } else {
                            reject(new Error('Unexpected response format'));
                        }
                    } catch (error) {
                        reject(new Error(`Failed to parse MCP response: ${error.message}`));
                    }
                } else {
                    reject(new Error(`MCP process failed with code ${code}: ${stderr}`));
                }
            });

            mcpProcess.on('error', (error) => {
                reject(new Error(`Failed to start MCP process: ${error.message}`));
            });

            // Send request
            mcpProcess.stdin.write(JSON.stringify(request) + '\n');
            mcpProcess.stdin.end();

            // Timeout after 30 seconds
            setTimeout(() => {
                mcpProcess.kill();
                reject(new Error('MCP request timed out'));
            }, 30000);
        });
    }
}

async function demonstrateArchLensIntegration() {
    console.log('🏗️ ArchLens MCP Integration Example');
    console.log('====================================\n');

    const serverPath = path.join(__dirname, '..', 'mcp', 'archlens_mcp_server.cjs');
    const client = new ArchLensMCPClient(serverPath);

    try {
        // 1. Quick project analysis
        console.log('📊 Step 1: Quick Project Analysis');
        console.log('----------------------------------');
        
        const analysis = await client.callTool('analyze_project', {
            project_path: '.'
        });
        
        console.log('✅ Analysis completed!');
        console.log('📋 Result preview:', analysis.content[0].text.substring(0, 300) + '...\n');

        // 2. Project structure
        console.log('🏗️ Step 2: Project Structure Analysis');
        console.log('-------------------------------------');
        
        const structure = await client.callTool('get_project_structure', {
            project_path: '.',
            show_metrics: true
        });
        
        console.log('✅ Structure analysis completed!');
        console.log('📋 Result preview:', structure.content[0].text.substring(0, 300) + '...\n');

        // 3. AI-ready comprehensive analysis
        console.log('🤖 Step 3: AI-Ready Comprehensive Analysis');
        console.log('------------------------------------------');
        
        const aiAnalysis = await client.callTool('export_ai_compact', {
            project_path: '.',
            focus_critical_only: false
        });
        
        console.log('✅ AI analysis completed!');
        const aiContent = aiAnalysis.content[0].text;
        console.log(`📊 Generated ${aiContent.length} characters of AI-ready analysis`);
        console.log('📋 Preview:', aiContent.substring(0, 400) + '...\n');

        // 4. Architecture diagram
        console.log('📈 Step 4: Architecture Diagram Generation');
        console.log('------------------------------------------');
        
        const diagram = await client.callTool('generate_diagram', {
            project_path: '.',
            diagram_type: 'mermaid',
            include_metrics: true
        });
        
        console.log('✅ Diagram generated!');
        const diagramContent = diagram.content[0].text;
        console.log('🎨 Mermaid diagram preview:');
        console.log(diagramContent.substring(0, 500) + '...\n');

        // 5. Demonstrate error handling
        console.log('⚠️ Step 5: Error Handling Demo');
        console.log('-------------------------------');
        
        try {
            await client.callTool('analyze_project', {
                project_path: '/nonexistent/path'
            });
        } catch (error) {
            console.log('✅ Error handling works correctly:');
            console.log(`❌ ${error.message}\n`);
        }

        console.log('🎉 Integration example completed successfully!');
        console.log('\n💡 Tips for integration:');
        console.log('  • Use analyze_project for quick overviews');
        console.log('  • Use export_ai_compact for detailed AI analysis');
        console.log('  • Use get_project_structure for understanding organization');
        console.log('  • Use generate_diagram for visual documentation');
        console.log('  • Always handle errors gracefully');
        console.log('  • Consider caching results for better performance');

    } catch (error) {
        console.error('❌ Integration example failed:', error.message);
        process.exit(1);
    }
}

// Example usage patterns
function showUsagePatterns() {
    console.log('\n📚 Common Usage Patterns');
    console.log('========================\n');

    console.log('🔍 Pattern 1: Code Review Automation');
    console.log(`
const reviewResults = await Promise.all([
    client.callTool('analyze_project', { project_path: '.' }),
    client.callTool('export_ai_compact', { 
        project_path: '.', 
        focus_critical_only: true 
    })
]);

// Process results for automated code review
const [overview, criticalIssues] = reviewResults;
`);

    console.log('🤖 Pattern 2: AI Assistant Integration');
    console.log(`
async function getArchitectureContext(projectPath) {
    const [structure, analysis, diagram] = await Promise.all([
        client.callTool('get_project_structure', { 
            project_path: projectPath 
        }),
        client.callTool('export_ai_compact', { 
            project_path: projectPath 
        }),
        client.callTool('generate_diagram', { 
            project_path: projectPath,
            diagram_type: 'mermaid'
        })
    ]);
    
    return {
        structure: structure.content[0].text,
        analysis: analysis.content[0].text,
        diagram: diagram.content[0].text
    };
}
`);

    console.log('📊 Pattern 3: Technical Debt Monitoring');
    console.log(`
async function monitorTechnicalDebt(projectPath) {
    const analysis = await client.callTool('export_ai_compact', {
        project_path: projectPath,
        include_diff_analysis: true
    });
    
    // Extract technical debt metrics
    const content = analysis.content[0].text;
    const debtMatches = content.match(/technical debt.*?(\\d+\\.?\\d*)%/gi);
    
    return debtMatches ? parseFloat(debtMatches[0].match(/\\d+\\.?\\d*/)[0]) : null;
}
`);
}

// Run the example
if (require.main === module) {
    demonstrateArchLensIntegration()
        .then(() => {
            showUsagePatterns();
        })
        .catch(console.error);
}

module.exports = { ArchLensMCPClient }; 