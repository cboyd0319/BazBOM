package io.bazbom.intellij.toolwindow

import com.intellij.ui.JBColor
import com.intellij.ui.SimpleTextAttributes
import com.intellij.ui.components.JBLabel
import io.bazbom.intellij.model.SecurityStatus
import java.awt.Component
import javax.swing.JTree
import javax.swing.tree.DefaultMutableTreeNode
import javax.swing.tree.DefaultTreeCellRenderer

/**
 * Custom cell renderer for dependency tree.
 * Colors nodes based on vulnerability severity.
 */
class DependencyTreeCellRenderer : DefaultTreeCellRenderer() {
    
    override fun getTreeCellRendererComponent(
        tree: JTree,
        value: Any?,
        selected: Boolean,
        expanded: Boolean,
        leaf: Boolean,
        row: Int,
        hasFocus: Boolean
    ): Component {
        val component = super.getTreeCellRendererComponent(tree, value, selected, expanded, leaf, row, hasFocus)
        
        if (value is DefaultMutableTreeNode) {
            val userObject = value.userObject
            
            when (userObject) {
                is BazBomToolWindowPanel.DependencyNodeData -> {
                    val dependency = userObject.dependency
                    text = userObject.displayText
                    
                    // Set icon and color based on security status
                    when (dependency.securityStatus) {
                        SecurityStatus.CRITICAL -> {
                            foreground = if (selected) JBColor.WHITE else JBColor.RED
                            // Use red circle icon
                        }
                        SecurityStatus.HIGH -> {
                            foreground = if (selected) JBColor.WHITE else JBColor.ORANGE
                            // Use orange circle icon
                        }
                        SecurityStatus.MEDIUM -> {
                            foreground = if (selected) JBColor.WHITE else JBColor.YELLOW.darker()
                            // Use yellow circle icon
                        }
                        SecurityStatus.LOW -> {
                            foreground = if (selected) JBColor.WHITE else JBColor.GRAY
                            // Use gray circle icon
                        }
                        SecurityStatus.SAFE -> {
                            foreground = if (selected) JBColor.WHITE else JBColor.GREEN.darker()
                            // Use green checkmark icon
                        }
                    }
                }
                is String -> {
                    text = userObject
                    foreground = if (selected) JBColor.WHITE else JBColor.GRAY
                }
            }
        }
        
        return component
    }
}
