package io.bazbom.intellij.toolwindow

import com.intellij.openapi.project.DumbAware
import com.intellij.openapi.project.Project
import com.intellij.openapi.wm.ToolWindow
import com.intellij.openapi.wm.ToolWindowFactory
import com.intellij.ui.components.JBLabel
import com.intellij.ui.components.JBPanel
import java.awt.BorderLayout
import javax.swing.SwingConstants

/**
 * Factory for creating the BazBOM tool window.
 * Displays dependency tree with security status.
 */
class BazBomToolWindowFactory : ToolWindowFactory, DumbAware {
    
    override fun createToolWindowContent(project: Project, toolWindow: ToolWindow) {
        val contentManager = toolWindow.contentManager
        val panel = createToolWindowPanel(project)
        val content = contentManager.factory.createContent(panel, "", false)
        contentManager.addContent(content)
    }
    
    private fun createToolWindowPanel(project: Project): JBPanel<*> {
        val panel = JBPanel<JBPanel<*>>(BorderLayout())
        
        // TODO: Replace with actual dependency tree view
        val label = JBLabel("BazBOM Security Scanner", SwingConstants.CENTER)
        panel.add(label, BorderLayout.CENTER)
        
        return panel
    }
    
    override fun shouldBeAvailable(project: Project): Boolean = true
}
