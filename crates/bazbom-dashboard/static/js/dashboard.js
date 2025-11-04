// BazBOM Dashboard JavaScript

// State
let currentData = {
    summary: null,
    graph: null,
    vulnerabilities: null,
    sbom: null
};

// Initialize dashboard
document.addEventListener('DOMContentLoaded', function() {
    console.log('BazBOM Dashboard initializing...');
    
    // Setup event listeners
    setupEventListeners();
    
    // Load initial data
    loadDashboardData();
    
    // Refresh every 30 seconds
    setInterval(loadDashboardData, 30000);
});

// Setup Event Listeners
function setupEventListeners() {
    // Refresh button
    document.getElementById('refresh-btn').addEventListener('click', loadDashboardData);
    
    // Export button
    document.getElementById('export-btn').addEventListener('click', exportDashboard);
    
    // Tab buttons
    document.querySelectorAll('.tab-button').forEach(button => {
        button.addEventListener('click', function() {
            switchTab(this.dataset.tab);
        });
    });
    
    // Graph controls
    document.getElementById('show-transitive').addEventListener('change', updateGraph);
    document.getElementById('severity-filter').addEventListener('change', updateGraph);
    
    // Search input
    document.getElementById('search-input').addEventListener('input', filterSBOMTable);
    
    // Export SBOM button
    document.getElementById('export-sbom-btn').addEventListener('click', exportSBOM);
}

// Load Dashboard Data
async function loadDashboardData() {
    try {
        // Show loading state
        console.log('Loading dashboard data...');
        
        // Fetch all data in parallel
        const [summary, graph, vulnerabilities, sbom] = await Promise.all([
            fetch('/api/dashboard/summary').then(r => r.json()),
            fetch('/api/dependencies/graph').then(r => r.json()),
            fetch('/api/vulnerabilities').then(r => r.json()),
            fetch('/api/sbom').then(r => r.json())
        ]);
        
        // Store data
        currentData = { summary, graph, vulnerabilities, sbom };
        
        // Update UI
        updateSummaryCards(summary);
        updateDependencyGraph(graph);
        updateVulnerabilityTimeline(vulnerabilities);
        updateSBOMExplorer(sbom);
        
        console.log('Dashboard data loaded successfully');
    } catch (error) {
        console.error('Failed to load dashboard data:', error);
        showError('Failed to load dashboard data. Make sure BazBOM has scanned your project.');
    }
}

// Update Summary Cards
function updateSummaryCards(summary) {
    if (!summary) return;
    
    // Security Score
    const scoreElement = document.getElementById('security-score');
    scoreElement.textContent = summary.security_score || 0;
    scoreElement.style.color = getScoreColor(summary.security_score);
    
    // Vulnerabilities
    const vulns = summary.vulnerabilities || {};
    document.getElementById('critical-count').textContent = vulns.critical || 0;
    document.getElementById('high-count').textContent = vulns.high || 0;
    document.getElementById('medium-count').textContent = vulns.medium || 0;
    document.getElementById('low-count').textContent = vulns.low || 0;
    
    // Dependencies
    document.getElementById('total-deps').textContent = summary.total_dependencies || 0;
    document.getElementById('direct-deps').textContent = summary.direct_dependencies || 0;
    document.getElementById('transitive-deps').textContent = 
        (summary.total_dependencies || 0) - (summary.direct_dependencies || 0);
    
    // Policy
    document.getElementById('policy-violations').textContent = summary.policy_violations || 0;
    document.getElementById('license-issues').textContent = summary.license_issues || 0;
}

// Update Dependency Graph (D3.js)
function updateDependencyGraph(graphData) {
    if (!graphData || !graphData.nodes || !graphData.edges) {
        console.warn('No graph data available');
        return;
    }
    
    const container = document.getElementById('dependency-graph');
    container.innerHTML = ''; // Clear previous graph
    
    const width = container.clientWidth;
    const height = container.clientHeight;
    
    // Create SVG
    const svg = d3.select('#dependency-graph')
        .append('svg')
        .attr('width', width)
        .attr('height', height);
    
    // Create force simulation
    const simulation = d3.forceSimulation(graphData.nodes)
        .force('link', d3.forceLink(graphData.edges)
            .id(d => d.id)
            .distance(100))
        .force('charge', d3.forceManyBody().strength(-300))
        .force('center', d3.forceCenter(width / 2, height / 2))
        .force('collision', d3.forceCollide().radius(30));
    
    // Create links
    const link = svg.append('g')
        .attr('class', 'links')
        .selectAll('line')
        .data(graphData.edges)
        .enter()
        .append('line')
        .attr('class', 'link')
        .attr('stroke-width', 1.5);
    
    // Create nodes
    const node = svg.append('g')
        .attr('class', 'nodes')
        .selectAll('g')
        .data(graphData.nodes)
        .enter()
        .append('g')
        .attr('class', 'node')
        .call(d3.drag()
            .on('start', dragstarted)
            .on('drag', dragged)
            .on('end', dragended));
    
    // Add circles to nodes
    node.append('circle')
        .attr('r', d => d.direct ? 8 : 5)
        .attr('fill', d => getSeverityColor(d.severity))
        .on('click', showNodeDetails);
    
    // Add labels to direct dependencies only
    node.filter(d => d.direct)
        .append('text')
        .attr('dx', 10)
        .attr('dy', 3)
        .text(d => d.name.split(':')[1] || d.name) // Show artifact name
        .style('font-size', '10px')
        .style('pointer-events', 'none');
    
    // Add tooltip
    node.append('title')
        .text(d => `${d.name}\nSeverity: ${d.severity || 'none'}\n${d.vulnerabilities || 0} vulnerabilities`);
    
    // Update positions on each tick
    simulation.on('tick', () => {
        link
            .attr('x1', d => d.source.x)
            .attr('y1', d => d.source.y)
            .attr('x2', d => d.target.x)
            .attr('y2', d => d.target.y);
        
        node.attr('transform', d => `translate(${d.x},${d.y})`);
    });
    
    // Drag functions
    function dragstarted(event) {
        if (!event.active) simulation.alphaTarget(0.3).restart();
        event.subject.fx = event.subject.x;
        event.subject.fy = event.subject.y;
    }
    
    function dragged(event) {
        event.subject.fx = event.x;
        event.subject.fy = event.y;
    }
    
    function dragended(event) {
        if (!event.active) simulation.alphaTarget(0);
        event.subject.fx = null;
        event.subject.fy = null;
    }
    
    function showNodeDetails(event, d) {
        console.log('Node clicked:', d);
        alert(`Package: ${d.name}\nVulnerabilities: ${d.vulnerabilities || 0}\nSeverity: ${d.severity || 'none'}`);
    }
}

// Update Vulnerability Timeline (Chart.js)
function updateVulnerabilityTimeline(vulnerabilities) {
    const ctx = document.getElementById('vulnerability-timeline');
    if (!ctx) return;
    
    // Sample data - in production, this would come from historical scans
    const data = {
        labels: ['Week 1', 'Week 2', 'Week 3', 'Week 4', 'Current'],
        datasets: [
            {
                label: 'Critical',
                data: [3, 2, 2, 1, vulnerabilities?.vulnerabilities?.filter(v => v.severity === 'critical').length || 0],
                borderColor: '#dc2626',
                backgroundColor: 'rgba(220, 38, 38, 0.1)',
                tension: 0.4
            },
            {
                label: 'High',
                data: [8, 7, 6, 5, vulnerabilities?.vulnerabilities?.filter(v => v.severity === 'high').length || 0],
                borderColor: '#ef4444',
                backgroundColor: 'rgba(239, 68, 68, 0.1)',
                tension: 0.4
            },
            {
                label: 'Medium',
                data: [12, 10, 9, 7, vulnerabilities?.vulnerabilities?.filter(v => v.severity === 'medium').length || 0],
                borderColor: '#f59e0b',
                backgroundColor: 'rgba(245, 158, 11, 0.1)',
                tension: 0.4
            }
        ]
    };
    
    // Destroy previous chart if exists
    if (window.vulnerabilityChart) {
        window.vulnerabilityChart.destroy();
    }
    
    window.vulnerabilityChart = new Chart(ctx, {
        type: 'line',
        data: data,
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    position: 'top',
                },
                title: {
                    display: true,
                    text: 'Vulnerability Trend Over Time'
                }
            },
            scales: {
                y: {
                    beginAtZero: true,
                    ticks: {
                        stepSize: 1
                    }
                }
            }
        }
    });
}

// Update SBOM Explorer
function updateSBOMExplorer(sbom) {
    const tbody = document.getElementById('sbom-tbody');
    if (!tbody) return;
    
    tbody.innerHTML = ''; // Clear existing rows
    
    if (!sbom || !sbom.packages || sbom.packages.length === 0) {
        tbody.innerHTML = '<tr><td colspan="5" class="loading">No SBOM data available</td></tr>';
        return;
    }
    
    sbom.packages.forEach(pkg => {
        const row = document.createElement('tr');
        row.innerHTML = `
            <td>${pkg.name || 'Unknown'}</td>
            <td>${pkg.version || 'Unknown'}</td>
            <td>${pkg.license || 'Unknown'}</td>
            <td>${getVulnerabilityBadge(pkg.vulnerabilities, pkg.severity)}</td>
            <td>${pkg.scope || 'compile'}</td>
        `;
        tbody.appendChild(row);
    });
}

// Helper Functions
function getScoreColor(score) {
    if (score >= 90) return '#10b981';
    if (score >= 70) return '#f59e0b';
    return '#ef4444';
}

function getSeverityColor(severity) {
    const colors = {
        'critical': '#dc2626',
        'high': '#ef4444',
        'medium': '#f59e0b',
        'low': '#64748b',
        'none': '#10b981'
    };
    return colors[severity] || colors.none;
}

function getVulnerabilityBadge(count, severity) {
    if (!count || count === 0) {
        return '<span class="vuln-badge none">None</span>';
    }
    
    const severityClass = severity || 'low';
    return `<span class="vuln-badge ${severityClass}">${count} ${severity || 'low'}</span>`;
}

function switchTab(tabName) {
    // Update tab buttons
    document.querySelectorAll('.tab-button').forEach(btn => {
        btn.classList.toggle('active', btn.dataset.tab === tabName);
    });
    
    // Update tab content
    document.querySelectorAll('.tab-content').forEach(content => {
        content.classList.toggle('active', content.id === `${tabName}-tab`);
    });
}

function updateGraph() {
    // Re-render graph with current filters
    if (currentData.graph) {
        const showTransitive = document.getElementById('show-transitive').checked;
        const severityFilter = document.getElementById('severity-filter').value;
        
        let filteredGraph = { ...currentData.graph };
        
        // Apply filters
        if (!showTransitive) {
            filteredGraph.nodes = filteredGraph.nodes.filter(n => n.direct);
        }
        
        if (severityFilter !== 'all') {
            filteredGraph.nodes = filteredGraph.nodes.filter(n => 
                n.severity === severityFilter || n.direct
            );
        }
        
        updateDependencyGraph(filteredGraph);
    }
}

function filterSBOMTable() {
    const searchTerm = document.getElementById('search-input').value.toLowerCase();
    const rows = document.querySelectorAll('#sbom-tbody tr');
    
    rows.forEach(row => {
        const text = row.textContent.toLowerCase();
        row.style.display = text.includes(searchTerm) ? '' : 'none';
    });
}

function exportDashboard() {
    alert('Export functionality coming soon! You can use browser print to PDF for now.');
}

function exportSBOM() {
    if (!currentData.sbom) {
        alert('No SBOM data available to export');
        return;
    }
    
    // Convert to JSON and download
    const dataStr = JSON.stringify(currentData.sbom, null, 2);
    const dataBlob = new Blob([dataStr], { type: 'application/json' });
    const url = URL.createObjectURL(dataBlob);
    const link = document.createElement('a');
    link.href = url;
    link.download = 'bazbom-sbom.json';
    link.click();
    URL.revokeObjectURL(url);
}

function showError(message) {
    // Simple error display - could be enhanced with a proper notification system
    const errorDiv = document.createElement('div');
    errorDiv.style.cssText = 'position: fixed; top: 20px; right: 20px; background: #fee2e2; color: #991b1b; padding: 1rem; border-radius: 0.5rem; box-shadow: 0 4px 6px rgba(0,0,0,0.1); z-index: 1000;';
    errorDiv.textContent = message;
    document.body.appendChild(errorDiv);
    setTimeout(() => errorDiv.remove(), 5000);
}

// Export for potential external use
window.BazBOMDashboard = {
    loadDashboardData,
    currentData,
    updateGraph
};
