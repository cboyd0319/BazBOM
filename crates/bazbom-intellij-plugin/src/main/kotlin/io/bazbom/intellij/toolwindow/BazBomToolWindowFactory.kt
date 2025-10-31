package io.bazbom.intellij.toolwindow

import com.intellij.openapi.project.DumbAware
import com.intellij.openapi.project.Project
import com.intellij.openapi.wm.ToolWindow
import com.intellij.openapi.wm.ToolWindowFactory
import com.intellij.ui.components.JBPanel
import com.intellij.ui.content.ContentFactory

/**
 * Factory for creating the BazBOM tool window.
 * Displays dependency tree with security status.
 */
class BazBomToolWindowFactory : ToolWindowFactory, DumbAware {
    
    override fun createToolWindowContent(project: Project, toolWindow: ToolWindow) {
        val contentManager = toolWindow.contentManager
        val panel = BazBomToolWindowPanel(project)
        val content = ContentFactory.getInstance().createContent(panel, "", false)
        contentManager.addContent(content)
    }
    
    override fun shouldBeAvailable(project: Project): Boolean = true
}
