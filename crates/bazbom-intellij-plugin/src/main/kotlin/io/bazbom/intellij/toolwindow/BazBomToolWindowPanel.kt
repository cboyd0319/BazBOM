package io.bazbom.intellij.toolwindow

import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.progress.ProgressIndicator
import com.intellij.openapi.progress.ProgressManager
import com.intellij.openapi.progress.Task
import com.intellij.openapi.project.Project
import com.intellij.ui.JBColor
import com.intellij.ui.components.JBLabel
import com.intellij.ui.components.JBPanel
import com.intellij.ui.components.JBScrollPane
import com.intellij.ui.treeStructure.Tree
import io.bazbom.intellij.model.DependencyNode
import io.bazbom.intellij.model.SecurityStatus
import io.bazbom.intellij.util.BazBomCliRunner
import io.bazbom.intellij.util.SbomParser
import java.awt.BorderLayout
import java.awt.FlowLayout
import javax.swing.JButton
import javax.swing.tree.DefaultMutableTreeNode
import javax.swing.tree.DefaultTreeModel

/**
 * Main panel for BazBOM tool window.
 * Displays dependency tree with security status and provides scan controls.
 */
class BazBomToolWindowPanel(private val project: Project) : JBPanel<BazBomToolWindowPanel>(BorderLayout()) {
    private val log = Logger.getInstance(BazBomToolWindowPanel::class.java)
    
    private val tree: Tree
    private val treeModel: DefaultTreeModel
    private val statusLabel: JBLabel
    private val scanButton: JButton
    private val refreshButton: JButton
    
    init {
        // Create tree
        val rootNode = DefaultMutableTreeNode("Project Dependencies")
        treeModel = DefaultTreeModel(rootNode)
        tree = Tree(treeModel)
        tree.cellRenderer = DependencyTreeCellRenderer()
        tree.isRootVisible = true
        
        // Create toolbar
        val toolbar = JBPanel<JBPanel<*>>(FlowLayout(FlowLayout.LEFT))
        
        scanButton = JButton("Scan Project")
        scanButton.addActionListener { runScan() }
        toolbar.add(scanButton)
        
        refreshButton = JButton("Refresh")
        refreshButton.addActionListener { refreshTree() }
        toolbar.add(refreshButton)
        
        // Create status bar
        statusLabel = JBLabel("Ready")
        statusLabel.foreground = JBColor.GRAY
        
        // Layout
        add(toolbar, BorderLayout.NORTH)
        add(JBScrollPane(tree), BorderLayout.CENTER)
        add(statusLabel, BorderLayout.SOUTH)
        
        // Load initial data
        refreshTree()
    }
    
    private fun runScan() {
        scanButton.isEnabled = false
        statusLabel.text = "Scanning..."
        statusLabel.foreground = JBColor.BLUE
        
        ProgressManager.getInstance().run(object : Task.Backgroundable(project, "Scanning with BazBOM", false) {
            override fun run(indicator: ProgressIndicator) {
                indicator.text = "Running BazBOM scan..."
                
                val cliRunner = BazBomCliRunner(project)
                val result = cliRunner.runScan(fast = true)
                
                ApplicationManager.getApplication().invokeLater {
                    if (result.success) {
                        statusLabel.text = "Scan complete"
                        statusLabel.foreground = JBColor.GREEN
                        refreshTree()
                    } else {
                        statusLabel.text = "Scan failed: ${result.errorMessage ?: "Unknown error"}"
                        statusLabel.foreground = JBColor.RED
                    }
                    scanButton.isEnabled = true
                }
            }
        })
    }
    
    private fun refreshTree() {
        val rootNode = treeModel.root as DefaultMutableTreeNode
        rootNode.removeAllChildren()
        
        // Try to load SBOM from project
        val projectPath = project.basePath ?: return
        val outputDir = java.io.File(projectPath, ".bazbom/scan-output")
        
        if (!outputDir.exists()) {
            rootNode.add(DefaultMutableTreeNode("No scan results found. Click 'Scan Project' to start."))
            treeModel.reload()
            statusLabel.text = "No scan data available"
            statusLabel.foreground = JBColor.GRAY
            return
        }
        
        val sbomFile = SbomParser.findSbomFile(outputDir)
        if (sbomFile == null) {
            rootNode.add(DefaultMutableTreeNode("SBOM file not found in ${outputDir.absolutePath}"))
            treeModel.reload()
            statusLabel.text = "SBOM file not found"
            statusLabel.foreground = JBColor.ORANGE
            return
        }
        
        val dependencyTree = SbomParser.parseSbom(sbomFile)
        if (dependencyTree == null) {
            rootNode.add(DefaultMutableTreeNode("Failed to parse SBOM"))
            treeModel.reload()
            statusLabel.text = "Failed to parse SBOM"
            statusLabel.foreground = JBColor.RED
            return
        }
        
        // Build tree nodes
        buildTreeNodes(rootNode, dependencyTree)
        treeModel.reload()
        
        // Expand first level
        for (i in 0 until tree.rowCount) {
            tree.expandRow(i)
        }
        
        // Update status
        val totalDeps = countDependencies(dependencyTree)
        val vulnCount = countVulnerabilities(dependencyTree)
        statusLabel.text = "Dependencies: $totalDeps | Vulnerabilities: $vulnCount"
        statusLabel.foreground = if (vulnCount > 0) JBColor.ORANGE else JBColor.GREEN
        
        log.info("Refreshed dependency tree: $totalDeps dependencies, $vulnCount vulnerabilities")
    }
    
    private fun buildTreeNodes(parentNode: DefaultMutableTreeNode, dependency: DependencyNode) {
        dependency.children.forEach { child ->
            val nodeText = buildNodeText(child)
            val childNode = DefaultMutableTreeNode(DependencyNodeData(child, nodeText))
            parentNode.add(childNode)
            
            if (child.children.isNotEmpty()) {
                buildTreeNodes(childNode, child)
            }
        }
    }
    
    private fun buildNodeText(node: DependencyNode): String {
        val vulnSummary = if (node.vulnerabilities.isNotEmpty()) {
            val critical = node.vulnerabilities.count { it.severity == "CRITICAL" }
            val high = node.vulnerabilities.count { it.severity == "HIGH" }
            val medium = node.vulnerabilities.count { it.severity == "MEDIUM" }
            val low = node.vulnerabilities.count { it.severity == "LOW" }
            
            val parts = mutableListOf<String>()
            if (critical > 0) parts.add("$critical CRITICAL")
            if (high > 0) parts.add("$high HIGH")
            if (medium > 0) parts.add("$medium MEDIUM")
            if (low > 0) parts.add("$low LOW")
            
            " [${parts.joinToString(", ")}]"
        } else {
            ""
        }
        
        return if (node.version.isNotEmpty()) {
            "${node.name}:${node.version}$vulnSummary"
        } else {
            "${node.name}$vulnSummary"
        }
    }
    
    private fun countDependencies(node: DependencyNode): Int {
        var count = if (node.version.isNotEmpty()) 1 else 0
        node.children.forEach { count += countDependencies(it) }
        return count
    }
    
    private fun countVulnerabilities(node: DependencyNode): Int {
        var count = node.vulnerabilities.size
        node.children.forEach { count += countVulnerabilities(it) }
        return count
    }
    
    /**
     * Data class to hold node information for rendering.
     */
    data class DependencyNodeData(
        val dependency: DependencyNode,
        val displayText: String
    )
}
