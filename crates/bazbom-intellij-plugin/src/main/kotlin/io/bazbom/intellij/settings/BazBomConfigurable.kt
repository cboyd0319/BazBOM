package io.bazbom.intellij.settings

import com.intellij.openapi.options.Configurable
import javax.swing.JComponent
import javax.swing.JPanel
import javax.swing.JLabel

/**
 * Settings panel for BazBOM plugin configuration.
 */
class BazBomConfigurable : Configurable {
    
    private var settingsPanel: JPanel? = null
    
    override fun getDisplayName(): String = "BazBOM"
    
    override fun createComponent(): JComponent {
        if (settingsPanel == null) {
            settingsPanel = JPanel()
            // TODO: Add actual settings controls
            settingsPanel!!.add(JLabel("BazBOM Settings - Coming Soon"))
        }
        return settingsPanel!!
    }
    
    override fun isModified(): Boolean = false
    
    override fun apply() {
        // TODO: Save settings
    }
    
    override fun reset() {
        // TODO: Load settings
    }
    
    override fun disposeUIResources() {
        settingsPanel = null
    }
}
