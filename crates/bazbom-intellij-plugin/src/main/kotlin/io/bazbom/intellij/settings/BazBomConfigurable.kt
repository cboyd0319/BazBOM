package io.bazbom.intellij.settings

import com.intellij.openapi.fileChooser.FileChooserDescriptorFactory
import com.intellij.openapi.options.Configurable
import com.intellij.openapi.ui.TextFieldWithBrowseButton
import com.intellij.ui.components.JBCheckBox
import com.intellij.ui.components.JBLabel
import java.awt.GridBagConstraints
import java.awt.GridBagLayout
import java.awt.Insets
import javax.swing.JComponent
import javax.swing.JPanel

/**
 * Settings panel for BazBOM plugin configuration.
 */
class BazBomConfigurable : Configurable {
    
    private var settingsPanel: JPanel? = null
    private var enableRealTimeScanning: JBCheckBox? = null
    private var showInlineWarnings: JBCheckBox? = null
    private var autoScanOnSave: JBCheckBox? = null
    private var autoScanOnOpen: JBCheckBox? = null
    private var showCritical: JBCheckBox? = null
    private var showHigh: JBCheckBox? = null
    private var showMedium: JBCheckBox? = null
    private var showLow: JBCheckBox? = null
    private var policyFilePath: TextFieldWithBrowseButton? = null
    private var cliPath: TextFieldWithBrowseButton? = null
    
    override fun getDisplayName(): String = "BazBOM"
    
    override fun createComponent(): JComponent {
        if (settingsPanel == null) {
            settingsPanel = JPanel(GridBagLayout())
            val gbc = GridBagConstraints()
            gbc.anchor = GridBagConstraints.WEST
            gbc.insets = Insets(5, 5, 5, 5)
            gbc.gridx = 0
            gbc.gridy = 0
            gbc.gridwidth = 2
            gbc.fill = GridBagConstraints.HORIZONTAL
            
            // Scanning options section
            settingsPanel!!.add(JBLabel("<html><b>Scanning Options</b></html>"), gbc)
            gbc.gridy++
            
            enableRealTimeScanning = JBCheckBox("Enable real-time scanning", true)
            settingsPanel!!.add(enableRealTimeScanning!!, gbc)
            gbc.gridy++
            
            showInlineWarnings = JBCheckBox("Show inline warnings", true)
            settingsPanel!!.add(showInlineWarnings!!, gbc)
            gbc.gridy++
            
            autoScanOnSave = JBCheckBox("Auto-scan on file save", true)
            settingsPanel!!.add(autoScanOnSave!!, gbc)
            gbc.gridy++
            
            autoScanOnOpen = JBCheckBox("Auto-scan on project open", false)
            settingsPanel!!.add(autoScanOnOpen!!, gbc)
            gbc.gridy++
            
            // Severity thresholds section
            gbc.insets = Insets(15, 5, 5, 5)
            settingsPanel!!.add(JBLabel("<html><b>Severity Thresholds</b></html>"), gbc)
            gbc.insets = Insets(5, 5, 5, 5)
            gbc.gridy++
            
            showCritical = JBCheckBox("Show CRITICAL (error)", true)
            settingsPanel!!.add(showCritical!!, gbc)
            gbc.gridy++
            
            showHigh = JBCheckBox("Show HIGH (warning)", true)
            settingsPanel!!.add(showHigh!!, gbc)
            gbc.gridy++
            
            showMedium = JBCheckBox("Show MEDIUM (warning)", true)
            settingsPanel!!.add(showMedium!!, gbc)
            gbc.gridy++
            
            showLow = JBCheckBox("Show LOW (info)", false)
            settingsPanel!!.add(showLow!!, gbc)
            gbc.gridy++
            
            // Policy file section
            gbc.insets = Insets(15, 5, 5, 5)
            settingsPanel!!.add(JBLabel("<html><b>Configuration</b></html>"), gbc)
            gbc.insets = Insets(5, 5, 5, 5)
            gbc.gridy++
            
            gbc.gridwidth = 1
            settingsPanel!!.add(JBLabel("Policy File:"), gbc)
            gbc.gridx = 1
            gbc.fill = GridBagConstraints.HORIZONTAL
            gbc.weightx = 1.0
            
            policyFilePath = TextFieldWithBrowseButton()
            policyFilePath!!.text = "bazbom.yml"
            policyFilePath!!.addBrowseFolderListener(
                "Select Policy File",
                "Select the BazBOM policy YAML file",
                null,
                FileChooserDescriptorFactory.createSingleFileDescriptor("yml")
            )
            settingsPanel!!.add(policyFilePath!!, gbc)
            
            gbc.gridx = 0
            gbc.gridy++
            gbc.weightx = 0.0
            settingsPanel!!.add(JBLabel("BazBOM CLI Path:"), gbc)
            gbc.gridx = 1
            gbc.weightx = 1.0
            
            cliPath = TextFieldWithBrowseButton()
            cliPath!!.text = "/usr/local/bin/bazbom"
            cliPath!!.addBrowseFolderListener(
                "Select BazBOM CLI",
                "Select the bazbom executable",
                null,
                FileChooserDescriptorFactory.createSingleFileDescriptor()
            )
            settingsPanel!!.add(cliPath!!, gbc)
        }
        return settingsPanel!!
    }
    
    override fun isModified(): Boolean {
        val settings = BazBomSettings.getInstance()
        return enableRealTimeScanning?.isSelected != settings.enableRealTimeScanning ||
                showInlineWarnings?.isSelected != settings.showInlineWarnings ||
                autoScanOnSave?.isSelected != settings.autoScanOnSave ||
                autoScanOnOpen?.isSelected != settings.autoScanOnOpen ||
                showCritical?.isSelected != settings.showCritical ||
                showHigh?.isSelected != settings.showHigh ||
                showMedium?.isSelected != settings.showMedium ||
                showLow?.isSelected != settings.showLow ||
                policyFilePath?.text != settings.policyFilePath ||
                cliPath?.text != settings.cliPath
    }
    
    override fun apply() {
        val settings = BazBomSettings.getInstance()
        settings.enableRealTimeScanning = enableRealTimeScanning?.isSelected ?: true
        settings.showInlineWarnings = showInlineWarnings?.isSelected ?: true
        settings.autoScanOnSave = autoScanOnSave?.isSelected ?: true
        settings.autoScanOnOpen = autoScanOnOpen?.isSelected ?: false
        settings.showCritical = showCritical?.isSelected ?: true
        settings.showHigh = showHigh?.isSelected ?: true
        settings.showMedium = showMedium?.isSelected ?: true
        settings.showLow = showLow?.isSelected ?: false
        settings.policyFilePath = policyFilePath?.text ?: "bazbom.yml"
        settings.cliPath = cliPath?.text ?: "/usr/local/bin/bazbom"
    }
    
    override fun reset() {
        val settings = BazBomSettings.getInstance()
        enableRealTimeScanning?.isSelected = settings.enableRealTimeScanning
        showInlineWarnings?.isSelected = settings.showInlineWarnings
        autoScanOnSave?.isSelected = settings.autoScanOnSave
        autoScanOnOpen?.isSelected = settings.autoScanOnOpen
        showCritical?.isSelected = settings.showCritical
        showHigh?.isSelected = settings.showHigh
        showMedium?.isSelected = settings.showMedium
        showLow?.isSelected = settings.showLow
        policyFilePath?.text = settings.policyFilePath
        cliPath?.text = settings.cliPath
    }
    
    override fun disposeUIResources() {
        settingsPanel = null
        enableRealTimeScanning = null
        showInlineWarnings = null
        autoScanOnSave = null
        autoScanOnOpen = null
        showCritical = null
        showHigh = null
        showMedium = null
        showLow = null
        policyFilePath = null
        cliPath = null
    }
}
